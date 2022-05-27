/**
 *  Adds container to window stuff
 */

/* Glium stuff */
use glium::{
	glutin::{
		event_loop::ControlFlow,
		window::{
			WindowBuilder,
			Icon,
		},
		dpi::LogicalSize,
	}
};

/* Window struct */
pub struct Window {
	pub window_builder: Option<WindowBuilder>,
    pub width: i32,
    pub height: i32
}

impl Window {
	/// Constructs window.
	pub fn from(width: i32, height: i32, resizable: bool) -> Self {
		let window_builder = WindowBuilder::new()
			.with_title("Terramine")
			.with_resizable(resizable)
            .with_inner_size(LogicalSize::new(width, height))
			.with_window_icon(Some(Self::load_icon()));

		Window {
            window_builder: Some(window_builder),
            width: width,
            height: height
        }
	}

	fn load_icon() -> Icon {
		/* Bytes vector from bmp file */
		/* File formatted in BGRA */
		let raw_data = include_bytes!("../image/TerramineIcon32p.bmp");
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
		Icon::from_rgba(data, 32, 32).unwrap()
	}

	/// Window close function.
    pub fn exit(control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Exit;
    }

	/// Gives window_builder and removes it from graphics struct
	pub fn take_window_builder(&mut self) -> WindowBuilder {
		/* Swaps struct variable with returned */
		if let None = self.window_builder {
			panic!("Window.window_builder is None!")
		} else {
			let mut window_builder = None;
			std::mem::swap(&mut window_builder, &mut self.window_builder);
			window_builder.unwrap()
		}
	}
}