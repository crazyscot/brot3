#![allow(missing_docs)]

use std::path::PathBuf;

pub use context::GraphicsContext;
pub use controller::ControllerTrait;
pub use egui_wgpu::wgpu;
use egui_winit::winit::event_loop::EventLoop;
pub use egui_winit::{egui, winit};
pub use ui::UiState;
use user_event::CustomEvent;

use crate::ui::Options;

mod app;
mod context;
mod controller;
mod fps_counter;
mod render_pass;
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
mod shader;
mod ui;
mod user_event;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    EventLoopError(#[from] egui_winit::winit::error::EventLoopError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("No shaders were provided")]
    EmptyShaderSet,
    #[error("Duplicate shader key {0}")]
    DuplicateShaderKey(&'static str),
    #[error("Missing CARGO_MANIFEST_DIR")]
    MissingCargoManifest,
    #[error("Shader directory {0} not found")]
    ShaderDirectoryNotFound(PathBuf),
    #[error("Shader key {0} not found")]
    UnknownShaderKey(&'static str),
    #[cfg(all(
        any(feature = "runtime-compilation", feature = "hot-reload-shader"),
        not(target_arch = "wasm32")
    ))]
    #[error(transparent)]
    BuildFailed(spirv_builder::SpirvBuilderError),
}

#[derive(Clone, Copy, Debug)]
pub struct PrebuiltShader {
    pub key: &'static str,
    pub bytes: &'static [u8],
}

impl PrebuiltShader {
    #[must_use]
    pub const fn new(key: &'static str, bytes: &'static [u8]) -> Self {
        Self { key, bytes }
    }
}

#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
#[derive(Clone, Copy, Debug)]
pub struct RuntimeCompilationShader {
    pub key: &'static str,
    pub crate_features: &'static [&'static str],
}

#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
impl RuntimeCompilationShader {
    #[must_use]
    pub const fn new(key: &'static str, crate_features: &'static [&'static str]) -> Self {
        Self {
            key,
            crate_features,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShaderDescriptor {
    pub(crate) key: &'static str,
    pub(crate) source: ShaderSource,
}

#[derive(Clone, Debug)]
pub(crate) enum ShaderSource {
    Prebuilt(&'static [u8]),
    #[cfg(all(
        any(feature = "runtime-compilation", feature = "hot-reload-shader"),
        not(target_arch = "wasm32")
    ))]
    RuntimePath(PathBuf),
}

/// Common parameters and options for all shader runs.
///
/// There is no `Default` implementation as `controller` and `title` must always be provided.
#[non_exhaustive]
#[allow(missing_debug_implementations)]
pub struct Parameters<C: ControllerTrait + Send> {
    /// UI controller
    pub controller: C,
    /// Window title
    pub title: String,
    options: ui::Options,
}

impl<C: ControllerTrait + Send> Parameters<C> {
    /// Constructor for the mandatory fields.
    /// Optional fields are set to their defaults.
    pub fn new<S: Into<String>>(controller: C, title: S) -> Self {
        Self {
            controller,
            title: title.into(),
            options: Options::default(),
        }
    }

    #[must_use]
    pub fn esc_key_exits(mut self, enable: bool) -> Self {
        self.options.escape_exits = enable;
        self
    }

    #[must_use]
    pub fn default_shader_key(mut self, key: &'static str) -> Self {
        self.options.default_shader_key = Some(key);
        self
    }
}

/// Run with runtime compilation
///
/// If `relative_to_manifest` is true, `shader_crate_path` is relative to `CARGO_MANIFEST_DIR`.
/// If not, it is a standard path (may be absolute or relative).
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
pub fn run_with_runtime_compilation<C: ControllerTrait + Send>(
    params: Parameters<C>,
    shaders: &'static [RuntimeCompilationShader],
    // Path of shader crate (see `relative_to_manifest`!)
    shader_crate_path: impl AsRef<std::path::Path>,
    // If true, shader_crate_path is relative to CARGO_MANIFEST_DIR
    relative_to_manifest: bool,
    // Location of librustc_codegen_spirv.so, if it's not on SHARED_LIBRARY_PATH
    rustc_codegen_spirv_location: Option<&PathBuf>,
) -> Result<(), Error> {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build()?;
    // Build the shaders before we pop open a window, since it might take a while.
    let shaders = shader::compile_shaders(
        #[cfg(feature = "hot-reload-shader")]
        &event_loop.create_proxy(),
        shaders,
        shader_crate_path,
        relative_to_manifest,
        rustc_codegen_spirv_location,
    )?;
    start(
        event_loop,
        shaders
            .into_iter()
            .map(|shader| ShaderDescriptor {
                key: shader.key,
                source: ShaderSource::RuntimePath(shader.path),
            })
            .collect(),
        params,
    )
}

pub fn run_with_prebuilt_shaders<C: ControllerTrait + Send>(
    params: Parameters<C>,
    shaders: &'static [PrebuiltShader],
) -> Result<(), Error> {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build()?;
    start(
        event_loop,
        shaders
            .iter()
            .map(|shader| ShaderDescriptor {
                key: shader.key,
                source: ShaderSource::Prebuilt(shader.bytes),
            })
            .collect(),
        params,
    )
}

fn start<C: ControllerTrait + Send>(
    event_loop: EventLoop<CustomEvent<C>>,
    shaders: Vec<ShaderDescriptor>,
    params: Parameters<C>,
) -> Result<(), Error> {
    validate_shaders(
        &shaders,
        params
            .options
            .default_shader_key
            .unwrap_or_else(|| shaders.first().map_or("", |shader| shader.key)),
    )?;
    let mut app = app::App::new(event_loop.create_proxy(), shaders, params);
    Ok(event_loop.run_app(&mut app)?)
}

fn validate_shaders(
    shaders: &[ShaderDescriptor],
    default_shader_key: &'static str,
) -> Result<(), Error> {
    if shaders.is_empty() {
        return Err(Error::EmptyShaderSet);
    }

    for (index, shader) in shaders.iter().enumerate() {
        if shaders[..index].iter().any(|other| other.key == shader.key) {
            return Err(Error::DuplicateShaderKey(shader.key));
        }
    }

    if shaders
        .iter()
        .all(|shader| shader.key != default_shader_key)
    {
        return Err(Error::UnknownShaderKey(default_shader_key));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Error, ShaderDescriptor, ShaderSource, validate_shaders};

    fn shader(key: &'static str) -> ShaderDescriptor {
        ShaderDescriptor {
            key,
            source: ShaderSource::Prebuilt(&[]),
        }
    }

    #[test]
    fn rejects_empty_shader_sets() {
        assert!(matches!(
            validate_shaders(&[], "default"),
            Err(Error::EmptyShaderSet)
        ));
    }

    #[test]
    fn rejects_duplicate_shader_keys() {
        let shaders = [shader("first"), shader("first")];
        assert!(matches!(
            validate_shaders(&shaders, "first"),
            Err(Error::DuplicateShaderKey("first"))
        ));
    }

    #[test]
    fn rejects_missing_default_shader_key() {
        let shaders = [shader("first"), shader("second")];
        assert!(matches!(
            validate_shaders(&shaders, "missing"),
            Err(Error::UnknownShaderKey("missing"))
        ));
    }

    #[test]
    fn accepts_unique_shader_keys_with_present_default() {
        let shaders = [shader("first"), shader("second")];
        assert!(validate_shaders(&shaders, "first").is_ok());
    }
}

#[allow(unsafe_code, clippy::disallowed_methods)]
pub fn setup_logging() {
    use std::fmt::Write;
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            let _ = console_log::init();
        } else {
            let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
            for loud_crate in ["naga", "wgpu_core", "wgpu_hal"] {
                if !rust_log.contains(&format!("{loud_crate}=")) {
                    let _ = write!(&mut rust_log, ",{loud_crate}=warn");
                }
            }
            unsafe {
                std::env::set_var("RUST_LOG", rust_log);
            }
            let _ = env_logger::try_init();
        }
    }
}
