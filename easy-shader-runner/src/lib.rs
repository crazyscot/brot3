use egui_winit::winit::event_loop::EventLoop;
use std::borrow::Cow;
use std::path::PathBuf;
use user_event::CustomEvent;

pub use context::GraphicsContext;
pub use controller::ControllerTrait;
pub use ui::UiState;

pub use egui_wgpu::wgpu;
pub use egui_winit::egui;
pub use egui_winit::winit;

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
    #[error("Missing CARGO_MANIFEST_DIR")]
    MissingCargoManifest,
    #[error("Shader directory {0} not found")]
    ShaderDirectoryNotFound(PathBuf),
    #[cfg(all(
        any(feature = "runtime-compilation", feature = "hot-reload-shader"),
        not(target_arch = "wasm32")
    ))]
    #[error(transparent)]
    BuildFailed(spirv_builder::SpirvBuilderError),
    #[error("Build failed")]
    BuildFailedQuietly,
}

/// Common parameters and options for all shader runs.
///
/// There is no `Default` implementation as `controller` and `title` must always be provided.
#[non_exhaustive]
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
            options: Default::default(),
        }
    }

    pub fn esc_key_exits(mut self, enable: bool) -> Self {
        self.options.escape_exits = enable;
        self
    }
}

/// Run with runtime compilation
///
/// If `relative_to_manifest` is true, `shader_crate_path` is relative to CARGO_MANIFEST_DIR.
/// If not, it is a standard path (may be absolute or relative).
#[cfg(all(
    any(feature = "runtime-compilation", feature = "hot-reload-shader"),
    not(target_arch = "wasm32")
))]
pub fn run_with_runtime_compilation<C: ControllerTrait + Send>(
    params: Parameters<C>,
    // Path of shader crate (see `relative_to_manifest`!)
    shader_crate_path: impl AsRef<std::path::Path>,
    // If true, shader_crate_path is relative to CARGO_MANIFEST_DIR
    relative_to_manifest: bool,
    // Location of librustc_codegen_spirv.so, if it's not on SHARED_LIBRARY_PATH
    rustc_codegen_spirv_location: Option<PathBuf>,
) -> Result<(), Error> {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build()?;
    // Build the shader before we pop open a window, since it might take a while.
    let shader_path = shader::compile_shader(
        #[cfg(feature = "hot-reload-shader")]
        event_loop.create_proxy(),
        shader_crate_path,
        relative_to_manifest,
        rustc_codegen_spirv_location,
    )?;
    let shader_bytes = std::fs::read(shader_path)?;
    start(event_loop, shader_bytes, params)
}

pub fn run_with_prebuilt_shader<C: ControllerTrait + Send>(
    params: Parameters<C>,
    shader_bytes: &'static [u8],
) -> Result<(), Error> {
    setup_logging();
    let event_loop = EventLoop::with_user_event().build()?;
    start(event_loop, shader_bytes, params)
}

fn start<C: ControllerTrait + Send>(
    event_loop: EventLoop<CustomEvent<C>>,
    shader_bytes: impl Into<Cow<'static, [u8]>>,
    params: Parameters<C>,
) -> Result<(), Error> {
    let mut app = app::App::new(event_loop.create_proxy(), shader_bytes.into(), params);
    Ok(event_loop.run_app(&mut app)?)
}

pub fn setup_logging() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            let _ = console_log::init();
        } else {
            let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
            for loud_crate in ["naga", "wgpu_core", "wgpu_hal"] {
                if !rust_log.contains(&format!("{loud_crate}=")) {
                    rust_log += &format!(",{loud_crate}=warn");
                }
            }
            unsafe {
                std::env::set_var("RUST_LOG", rust_log);
            }
            let _ = env_logger::try_init();
        }
    }
}
