#![allow(clippy::single_match)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{self, prelude::*};

mod controller;

#[derive(Clone, Copy, Default)]
pub(crate) struct Options {}

const TITLE: &str = "brot3";

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    let controller = controller::Controller::new(&Options {});
    cfg_if::cfg_if! {
        if #[cfg(all(
            any(feature = "hot-reload-shader", feature = "runtime-compilation"),
            not(target_arch = "wasm32")
        ))] {
            // CAUTION: Hard-wired path !
            easy_shader_runner::run_with_runtime_compilation(controller, "../shader", TITLE);
        } else {
            easy_shader_runner::run_with_prebuilt_shader(controller, include_bytes!(env!("shader.spv")), TITLE);
        }
    }
}
