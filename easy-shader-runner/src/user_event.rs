use crate::{app::Graphics, controller::ControllerTrait};

pub(crate) enum CustomEvent<C: ControllerTrait> {
    #[cfg(all(feature = "hot-reload-shader", not(target_arch = "wasm32")))]
    NewModule(std::path::PathBuf),
    CreateWindow(Box<Graphics<C>>),
}
