pub mod message_box;

/**
 *  Adds container to window stuff
 */

use {
	crate::app::utils::werror::prelude::*,
	glium::glutin::{
		event_loop::EventLoop,
		window::{
			WindowBuilder,
			Icon,
			Window as GWindow
		},
		dpi::PhysicalSize,
		ContextWrapper,
		PossiblyCurrent,
		ContextBuilder,
		GlRequest,
	},
};

/// Temporary holds glutin window.
pub struct Window {
	pub window: Option<ContextWrapper<PossiblyCurrent, GWindow>>,
}

impl Window {
	/// Constructs window.
	pub fn from(event_loop: &EventLoop<()>, width: i32, height: i32) -> Self {
		let window = WindowBuilder::new()
			.with_title("Terramine")
			.with_resizable(true)
            .with_inner_size(PhysicalSize::new(width, height))
			.with_window_icon(Some(Self::load_icon()));
		let window = ContextBuilder::new()
			.with_gl(GlRequest::Latest)
			.with_depth_buffer(24)
			.with_stencil_buffer(8)
			.with_vsync(true)
			.build_windowed(window, event_loop)
			.wunwrap();
		let window = unsafe {
			window.make_current().wunwrap()
		};

		Window { window: Some(window) }
	}

	fn load_icon() -> Icon {
		/* Bytes vector from bmp file */
		/* File formatted in BGRA */
		let raw_data = include_bytes!("../../../image/TerramineIcon32p.bmp");
		let mut raw_data = *raw_data;

		/* Bytemap pointer load from 4 bytes of file */
		/* This pointer is 4 bytes large and can be found on 10th byte position from file begin */
		let start_bytes: usize =	(raw_data[13] as usize) << 24 |
									(raw_data[12] as usize) << 16 |
									(raw_data[11] as usize) << 8  |
									(raw_data[10] as usize);

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
		Icon::from_rgba(data, 32, 32).wunwrap()
	}

	/// Gives window and removes it from graphics struct
	pub fn take_window(&mut self) -> ContextWrapper<PossiblyCurrent, GWindow> {
		/* Swaps struct variable with returned */
		if let None = self.window {
			panic!("Window.window is None!")
		} else {
			let mut window = None;
			std::mem::swap(&mut window, &mut self.window);
			window.wunwrap()
		}
	}
}