use spirv_builder::{CompileResult, MetadataPrintout, ModuleResult, SpirvBuilder};
use std::path::{Path, PathBuf};
#[cfg(feature = "hot-reload-shader")]
use {
    crate::{controller::ControllerTrait, user_event::CustomEvent},
    egui_winit::winit::event_loop::EventLoopProxy,
};

use crate::Error as ESRError;

/// Compile the shader with a standard path (absolute, or relative to the current working directory)
///
/// If `relative_to_manifest` is true, `shader_crate_path` is relative to CARGO_MANIFEST_DIR.
/// If not, it is a standard path (may be absolute or relative).
pub(crate) fn compile_shader<#[cfg(feature = "hot-reload-shader")] C: ControllerTrait + Send>(
    #[cfg(feature = "hot-reload-shader")] event_proxy: EventLoopProxy<CustomEvent<C>>,
    crate_path: impl AsRef<Path>,
    relative_to_manifest: bool,
    rustc_codegen_spirv_location: Option<PathBuf>,
) -> Result<PathBuf, ESRError> {
    // Hack: spirv_builder builds into a custom directory if running under cargo, to not
    // deadlock, and the default target directory if not. However, packages like `proc-macro2`
    // have different configurations when being built here vs. when building
    // rustc_codegen_spirv normally, so we *want* to build into a separate target directory, to
    // not have to rebuild half the crate graph every time we run. So, pretend we're running
    // under cargo by setting these environment variables.
    unsafe {
        std::env::set_var(
            "OUT_DIR",
            option_env!("SHADERS_TARGET_DIR").unwrap_or(env!("OUT_DIR")),
        );
        std::env::set_var("PROFILE", env!("PROFILE"));
    }

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

    let builder = SpirvBuilder::new(crate_path, "spirv-unknown-vulkan1.1")
        .print_metadata(MetadataPrintout::None)
        .shader_crate_features([
            #[cfg(feature = "emulate_constants")]
            "emulate_constants".into(),
        ])
        .shader_panic_strategy(spirv_builder::ShaderPanicStrategy::SilentExit);
    let builder = if let Some(p) = rustc_codegen_spirv_location {
        builder.rustc_codegen_spirv_location(p)
    } else {
        builder
    };
    fn handle_compile_result(compile_result: CompileResult) -> PathBuf {
        match compile_result.module {
            ModuleResult::SingleModule(result) => result,
            ModuleResult::MultiModule(_) => {
                panic!("expected `ModuleResult::SingleModule")
            }
        }
    }
    #[cfg(feature = "hot-reload-shader")]
    let initial_result = builder
        .watch(move |compile_result, first| {
            if let Some(first) = first {
                first.submit(compile_result);
            } else {
                std::assert!(
                    event_proxy
                        .send_event(CustomEvent::NewModule(handle_compile_result(
                            compile_result
                        )))
                        .is_ok()
                )
            }
        })
        .expect("Configuration is incorrect for watching")
        .first_compile
        .ok_or(ESRError::BuildFailedQuietly)?;
    #[cfg(not(feature = "hot-reload-shader"))]
    let initial_result = builder.build().map_err(ESRError::BuildFailed)?;
    Ok(handle_compile_result(initial_result))
}
