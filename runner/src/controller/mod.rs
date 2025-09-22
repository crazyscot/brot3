use crate::Options;
use easy_shader_runner::{ControllerTrait, GraphicsContext};
use glam::*;
use shared::push_constants::shader::*;

pub(crate) struct Controller {
    /// viewport pixel size
    size: UVec2,
}

impl Controller {
    pub fn new(_options: &Options) -> Self {
        Self { size: UVec2::ZERO }
    }
}

impl ControllerTrait for Controller {
    fn resize(&mut self, size: UVec2) {
        self.size = size;
    }

    fn prepare_render(
        &mut self,
        _gfx_ctx: &GraphicsContext,
        _offset: Vec2,
    ) -> impl bytemuck::NoUninit {
        FragmentConstants {
            size: self.size.into(),
        }
    }
}
