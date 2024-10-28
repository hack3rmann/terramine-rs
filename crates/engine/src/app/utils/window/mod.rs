// FIXME(hack3rmann): support unix
// pub mod message_box;

use {
    // glium::glutin::{
    //     // dpi::PhysicalSize,
    //     event_loop::EventLoop,
    //     // window::WindowBuilder,
    //     ContextBuilder,
    //     ContextWrapper,
    //     GlRequest,
    //     PossiblyCurrent,
    // },
    math_linear::prelude::*,
};

use glium::backend::glutin::{SimpleWindowBuilder, Display};
use glium::glutin::surface::WindowSurface;
use glium::winit::event_loop::EventLoop;
use glium::winit::window::{Icon, Window as GWindow};

/// Temporary holds glutin window.
#[derive(Debug)]
pub struct Window {
    // pub window: Option<ContextWrapper<PossiblyCurrent, GWindow>>,
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

        // let window = WindowBuilder::new()
        //     .with_title("Terramine")
        //     .with_resizable(true)
        //     .with_inner_size(PhysicalSize::new(sizes.x as u32, sizes.y as u32))
        //     .with_window_icon(Some(Self::load_icon()));

        // let window = ContextBuilder::new()
        //     .with_gl(GlRequest::Latest)
        //     .with_depth_buffer(24)
        //     .with_stencil_buffer(8)
        //     .with_vsync(false)
        //     .build_windowed(window, event_loop)
        //     .expect("failed to build the window");

        // let window = unsafe {
        //     window
        //         .make_current()
        //         .expect("failed to make window as current context")
        // };

        Self { window, display }
    }

    fn load_icon() -> Icon {
        /* Bytes vector from bmp file */
        /* File formatted in BGRA */
        let raw_data = include_bytes!("../../../../../../assets/image/terramine_icon_32p.bmp");
        let mut raw_data = *raw_data;

        /* Bytemap pointer load from 4 bytes of file */
        /* This pointer is 4 bytes large and can be found on 10th byte position from file begin */
        let start_bytes = (raw_data[13] as usize) << 24
            | (raw_data[12] as usize) << 16
            | (raw_data[11] as usize) << 8
            | (raw_data[10] as usize);

        /* Trim useless information */
        let raw_data = raw_data[start_bytes..].as_mut();

        /* Converting BGRA into RGBA formats */
        let mut current: usize = 0;
        while current <= raw_data.len() - 3 {
            raw_data.swap(current, current + 2);
            current += 4;
        }

        /* Upload data */
        let mut data = Vec::with_capacity(raw_data.len());
        data.extend_from_slice(raw_data);
        Icon::from_rgba(data, 32, 32).expect(
            "length of data should be divisible by 4, \
                     and width * height must equal data.len() / 4",
        )
    }

    // /// Gives window and removes it from graphics struct
    // pub fn take_window(&mut self) -> ContextWrapper<PossiblyCurrent, GWindow> {
    //     self.window.take().expect("cannot take window twice")
    // }
}
