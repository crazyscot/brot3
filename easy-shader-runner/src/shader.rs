use std::path::{Path, PathBuf};

use spirv_builder::{CompileResult, ModuleResult, SpirvBuilder, SpirvMetadata};
#[cfg(feature = "hot-reload-shader")]
use {
    crate::{controller::ControllerTrait, user_event::CustomEvent},
    egui_winit::winit::event_loop::EventLoopProxy,
};

use crate::{Error as ESRError, RuntimeCompilationShader};

pub(crate) struct CompiledShader {
    pub(crate) key: &'static str,
    pub(crate) path: PathBuf,
}

/// Compile the shader with a standard path (absolute, or relative to the current working directory)
///
/// If `relative_to_manifest` is true, `shader_crate_path` is relative to `CARGO_MANIFEST_DIR`.
/// If not, it is a standard path (may be absolute or relative).
#[allow(unsafe_code, clippy::disallowed_methods)]
pub(crate) fn compile_shaders<#[cfg(feature = "hot-reload-shader")] C: ControllerTrait + Send>(
    #[cfg(feature = "hot-reload-shader")] event_proxy: &EventLoopProxy<CustomEvent<C>>,
    shaders: &'static [RuntimeCompilationShader],
    initial_shader_key: &'static str,
    crate_path: impl AsRef<Path>,
    relative_to_manifest: bool,
    rustc_codegen_spirv_location: Option<&PathBuf>,
) -> Result<Vec<CompiledShader>, ESRError> {
    // Hack: spirv_builder builds into a custom directory if running under cargo, to not
    // deadlock, and the default target directory if not. However, packages like `proc-macro2`
    // have different configurations when being built here vs. when building
    // rustc_codegen_spirv normally, so we *want* to build into a separate target directory, to
    // not have to rebuild half the crate graph every time we run. So, pretend we're running
    // under cargo by setting these environment variables.
    let crate_path = if relative_to_manifest {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").map_err(|_| ESRError::MissingCargoManifest)?;
        let buf = [Path::new(&manifest_dir), crate_path.as_ref()]
            .iter()
            .collect::<PathBuf>();
        if !matches!(std::fs::exists(&buf), Ok(true)) {
            return Err(ESRError::ShaderDirectoryNotFound(buf));
        }
        // It's a PathBuf
        buf
    } else {
        crate_path.as_ref().to_path_buf()
    };

    #[cfg(feature = "hot-reload-shader")]
    {
        compile_shaders_with_background_warmup(
            event_proxy,
            shaders,
            initial_shader_key,
            &crate_path,
            rustc_codegen_spirv_location,
        )
    }

    #[cfg(not(feature = "hot-reload-shader"))]
    {
        let _initial_shader_key = initial_shader_key;
        shaders
            .iter()
            .map(|shader| {
                compile_shader(
                    shader,
                    "(runtime compilation)",
                    &crate_path,
                    rustc_codegen_spirv_location,
                )
            })
            .collect()
    }
}

#[cfg(not(feature = "hot-reload-shader"))]
#[allow(unsafe_code, clippy::disallowed_methods)]
fn compile_shader(
    shader: &RuntimeCompilationShader,
    reason: &str,
    crate_path: &Path,
    rustc_codegen_spirv_location: Option<&PathBuf>,
) -> Result<CompiledShader, ESRError> {
    let out_dir = shader_out_dir(shader.key);
    std::fs::create_dir_all(&out_dir)?;
    unsafe {
        std::env::set_var("OUT_DIR", &out_dir);
        std::env::set_var("PROFILE", env!("PROFILE"));
    }

    #[allow(unused_mut)]
    let mut shader_features = shader
        .crate_features
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    #[cfg(feature = "emulate_constants")]
    shader_features.push("emulate_constants".to_string());
    let start = std::time::Instant::now();
    log_shader_build_start(shader.key, reason, &shader_features);

    let mut builder = SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
        .spirv_metadata(SpirvMetadata::None)
        .shader_crate_features(shader_features)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    builder.build_script.defaults = true;
    builder.build_script.dependency_info = Some(false); // Don't emit env vars for cargo

    let builder = if let Some(p) = rustc_codegen_spirv_location {
        builder.rustc_codegen_spirv_location(p)
    } else {
        builder
    };

    let initial_result = builder.build().map_err(ESRError::BuildFailed)?;
    let compiled_shader = CompiledShader {
        key: shader.key,
        path: stage_compile_result(shader.key, initial_result)?,
    };
    let duration = start.elapsed();
    log_shader_build_end(shader.key, duration);
    Ok(compiled_shader)
}

#[cfg(feature = "hot-reload-shader")]
fn compile_shaders_with_background_warmup<C: ControllerTrait + Send>(
    event_proxy: &EventLoopProxy<CustomEvent<C>>,
    shaders: &'static [RuntimeCompilationShader],
    initial_shader_key: &'static str,
    crate_path: &Path,
    rustc_codegen_spirv_location: Option<&PathBuf>,
) -> Result<Vec<CompiledShader>, ESRError> {
    let Some((initial_shader, remaining_shaders)) = shaders.split_first().map(|(first, _rest)| {
        let initial = shaders
            .iter()
            .find(|shader| shader.key == initial_shader_key)
            .unwrap_or(first);
        let remaining = shaders
            .iter()
            .filter(|shader| shader.key != initial.key)
            .copied()
            .collect::<Vec<_>>();
        (initial, remaining)
    }) else {
        return Ok(Vec::new());
    };

    let initial_compiled_shader = compile_shader_with_watch(
        event_proxy.clone(),
        initial_shader,
        "as the startup shader",
        crate_path,
        rustc_codegen_spirv_location,
    )?;

    for shader in remaining_shaders {
        let event_proxy = event_proxy.clone();
        let crate_path = crate_path.to_path_buf();
        let rustc_codegen_spirv_location = rustc_codegen_spirv_location.cloned();
        let _jh = std::thread::spawn(move || {
            let compiled_shader = match compile_shader_with_watch(
                event_proxy.clone(),
                &shader,
                "in the background",
                &crate_path,
                rustc_codegen_spirv_location.as_ref(),
            ) {
                Ok(compiled_shader) => compiled_shader,
                Err(error) => {
                    log::error!("Failed to warm up shader {}: {error}", shader.key);
                    return;
                }
            };

            if event_proxy
                .send_event(CustomEvent::NewModule {
                    shader_key: compiled_shader.key,
                    shader_path: compiled_shader.path,
                })
                .is_err()
            {
                log::warn!(
                    "Shader warm-up completed for {}, but the event loop is no longer running",
                    shader.key
                );
            }
        });
    }

    let mut compiled_shaders = Vec::with_capacity(shaders.len());
    compiled_shaders.push(initial_compiled_shader);
    compiled_shaders.extend(
        shaders
            .iter()
            .filter(|shader| shader.key != initial_shader.key)
            .map(|shader| CompiledShader {
                key: shader.key,
                path: shader_out_dir(shader.key).join("shader.spv"),
            }),
    );
    Ok(compiled_shaders)
}

#[cfg(feature = "hot-reload-shader")]
#[allow(unsafe_code, clippy::disallowed_methods)]
fn compile_shader_with_watch<C: ControllerTrait + Send>(
    event_proxy: EventLoopProxy<CustomEvent<C>>,
    shader: &RuntimeCompilationShader,
    initial_reason: &str,
    crate_path: &Path,
    rustc_codegen_spirv_location: Option<&PathBuf>,
) -> Result<CompiledShader, ESRError> {
    let out_dir = shader_out_dir(shader.key);
    std::fs::create_dir_all(&out_dir)?;
    unsafe {
        std::env::set_var("OUT_DIR", &out_dir);
        std::env::set_var("PROFILE", env!("PROFILE"));
    }

    #[allow(unused_mut)]
    let mut shader_features = shader
        .crate_features
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    #[cfg(feature = "emulate_constants")]
    shader_features.push("emulate_constants".to_string());
    log_shader_build_start(shader.key, initial_reason, &shader_features);
    let start = std::time::Instant::now();

    let mut builder = SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
        .spirv_metadata(SpirvMetadata::None)
        .shader_crate_features(shader_features)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    builder.build_script.defaults = true;
    builder.build_script.dependency_info = Some(false); // Don't emit env vars for cargo

    let builder = if let Some(p) = rustc_codegen_spirv_location {
        builder.rustc_codegen_spirv_location(p)
    } else {
        builder
    };

    let mut watcher = builder
        .watch()
        .expect("Configuration is incorrect for watching");
    let first_compile = watcher.recv().map_err(ESRError::BuildFailed)?;
    let shader_key = shader.key;
    let _jh = std::thread::spawn(move || {
        loop {
            let compile_result = match watcher.recv() {
                Ok(compile_result) => compile_result,
                Err(e) => {
                    log::error!("Shader build failed: {e:?}");
                    continue;
                }
            };
            let shader_path = match stage_compile_result(shader_key, compile_result) {
                Ok(shader_path) => shader_path,
                Err(e) => {
                    log::error!("Failed to stage shader build for {shader_key}: {e}");
                    continue;
                }
            };
            if event_proxy
                .send_event(CustomEvent::NewModule {
                    shader_key,
                    shader_path,
                })
                .is_err()
            {
                break;
            }
            log::info!("Rebuilt {shader_key} because the shader source changed");
        }
    });

    let compiled_shader = CompiledShader {
        key: shader.key,
        path: stage_compile_result(shader.key, first_compile)?,
    };
    let duration = start.elapsed();
    log_shader_build_end(shader.key, duration);
    Ok(compiled_shader)
}

fn handle_compile_result(compile_result: CompileResult) -> PathBuf {
    log::debug!("Shader compile result: {compile_result:?}");
    match compile_result.module {
        ModuleResult::SingleModule(result) => result,
        ModuleResult::MultiModule(_) => {
            panic!("expected `ModuleResult::SingleModule")
        }
    }
}

fn stage_compile_result(
    shader_key: &str,
    compile_result: CompileResult,
) -> Result<PathBuf, ESRError> {
    let compiled_shader_path = handle_compile_result(compile_result);
    let staged_shader_path = shader_out_dir(shader_key).join("shader.spv");
    let _copied = std::fs::copy(&compiled_shader_path, &staged_shader_path)?;
    Ok(staged_shader_path)
}

fn shader_out_dir(shader_key: &str) -> PathBuf {
    PathBuf::from(option_env!("SHADERS_TARGET_DIR").unwrap_or(env!("OUT_DIR"))).join(shader_key)
}

fn log_shader_build_start(shader_key: &str, reason: &str, shader_features: &[String]) {
    log::info!(
        "Starting shader build for {shader_key} {reason}, with features {shader_features:?}"
    );
}

fn log_shader_build_end(shader_key: &str, duration: std::time::Duration) {
    log::info!("Built {shader_key} in {duration:?}");
}
