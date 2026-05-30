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

    shaders
        .iter()
        .map(|shader| {
            compile_shader(
                #[cfg(feature = "hot-reload-shader")]
                event_proxy.clone(),
                shader,
                &crate_path,
                rustc_codegen_spirv_location,
            )
        })
        .collect()
}

#[allow(unsafe_code, clippy::disallowed_methods)]
fn compile_shader<#[cfg(feature = "hot-reload-shader")] C: ControllerTrait + Send>(
    #[cfg(feature = "hot-reload-shader")] event_proxy: EventLoopProxy<CustomEvent<C>>,
    shader: &RuntimeCompilationShader,
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
    log::debug!(
        "Compiling shader {} with features {:?}",
        shader.key,
        shader_features
    );

    let mut builder = SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
        .spirv_metadata(SpirvMetadata::None)
        .shader_crate_features(shader_features)
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    builder.build_script.defaults = true;
    builder.build_script.dependency_info = Some(true);

    let builder = if let Some(p) = rustc_codegen_spirv_location {
        builder.rustc_codegen_spirv_location(p)
    } else {
        builder
    };

    #[cfg(feature = "hot-reload-shader")]
    let initial_result = {
        let mut watcher = builder
            .watch()
            .expect("Configuration is incorrect for watching");
        let first_compile = watcher.recv().map_err(ESRError::BuildFailed)?;
        let shader_key = shader.key;
        //let mut thread_watcher = watcher.forget_lifetime();
        let _jh = std::thread::spawn(move || {
            loop {
                let compile_result = match watcher.recv() {
                    Ok(compile_result) => compile_result,
                    Err(e) => {
                        log::error!("Shader build failed: {e:?}");
                        continue;
                    }
                };
                std::assert!(
                    event_proxy
                        .send_event(CustomEvent::NewModule {
                            shader_key,
                            shader_path: handle_compile_result(compile_result),
                        })
                        .is_ok()
                );
            }
        });
        first_compile
    };

    #[cfg(not(feature = "hot-reload-shader"))]
    let initial_result = builder.build().map_err(ESRError::BuildFailed)?;
    Ok(CompiledShader {
        key: shader.key,
        path: handle_compile_result(initial_result),
    })
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

fn shader_out_dir(shader_key: &str) -> PathBuf {
    PathBuf::from(option_env!("SHADERS_TARGET_DIR").unwrap_or(env!("OUT_DIR"))).join(shader_key)
}
