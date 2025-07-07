use glium::{
    backend::glutin::{Display, SimpleWindowBuilder},
    glutin::surface::WindowSurface,
    winit::{event_loop::EventLoop, window::Window as GWindow},
};
use math_linear::prelude::*;

/// Temporary holds glutin window.
#[derive(Debug)]
pub struct Window {
    pub window: GWindow,
    pub display: Display<WindowSurface>,
}

impl Window {
    /// Constructs window.
    pub fn new(event_loop: &EventLoop<()>, sizes: USize2) -> Self {
        let (window, display) = SimpleWindowBuilder::new()
            .with_title("Terramine")
            .with_inner_size(sizes.x as u32, sizes.y as u32)
            .build(event_loop);

        Self { window, display }
    }
}
