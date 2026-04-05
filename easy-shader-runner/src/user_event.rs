use crate::app::Graphics;
use crate::controller::ControllerTrait;

pub enum CustomEvent<C: ControllerTrait> {
    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    NewModule(std::path::PathBuf),
    CreateWindow(Box<Graphics<C>>),
}
