mod utils;

/* Glium includes */
use glium::{
	glutin::event::{
		Event,
		WindowEvent,
	},
	Surface,
	uniform
};

use std::io::prelude::*;

/* Other files */
use utils::{
	*,
	user_io::{InputManager, KeyCode},
	graphics::{
		Graphics,
		camera::Camera,
		shader::Shader,
		texture::Texture,
		vertex_buffer::VertexBuffer,
	},
};

fn main() {
	/* Keyboard init */
	let mut input_manager = InputManager::new();

	/* Graphics initialization */
	let mut graphics = Graphics::initialize().unwrap();

	/* Camera handle */
	let mut camera = Camera::new();

	/* Texture loading */
	let texture = Texture::from("src/image/grass_top_separ.png", &graphics.display).unwrap();

	/* Vertex buffer loading */
	let vertex_buffer = VertexBuffer::default(&graphics);
	vertex_buffer.bind(&mut graphics);

	/* Shader program */
	let shaders = Shader::new("vertex_shader", "fragment_shader", &graphics.display);
	graphics.upload_shaders(shaders);

	/* Temporary moves */
	let vertex_buffer = graphics.take_vertex_buffer();
	let indices = graphics.take_privitive_type();
	let shaders = graphics.take_shaders();

	/* Camera preposition */
	camera.set_position(0.0, 0.0, 2.0);

	/* Time stuff */
	let mut time = 0.0;
	let mut last_frame = std::time::Instant::now();
	let mut _dt: f64 = 0.0;

	/* Event/game loop */
	graphics.take_event_loop().run(move |event, _, control_flow| {
		/* Aliasing */
		let window = graphics.display.gl_window();

		/* Event handlers */
		graphics.imguiw.handle_event(graphics.imguic.io_mut(), window.window(), &event);
		input_manager.handle_event(&event, &graphics);

		/* This event handler */
		match event {
			/* Window events */
	        Event::WindowEvent { event, .. } => match event {
	 			/* Close event */
	            WindowEvent::CloseRequested => {
					*control_flow = glium::glutin::event_loop::ControlFlow::Exit;
					let mut buf: String = Default::default();
					graphics.imguic.save_ini_settings(&mut buf);

					let mut file = std::fs::File::create("src/imgui_settings.ini").unwrap();
					file.write_all(buf.as_bytes()).unwrap();
				},
				WindowEvent::Resized(new_size) => {
					camera.aspect_ratio = new_size.height as f32 / new_size.width as f32;
				},
	            _ => (),
	        },
	 		Event::MainEventsCleared => {
	 			/* Close window is `escape` pressed */
	 			if input_manager.keyboard.just_pressed(KeyCode::Escape) {
					*control_flow = glium::glutin::event_loop::ControlFlow::Exit;
					let mut buf: String = Default::default();
					graphics.imguic.save_ini_settings(&mut buf);

					let mut file = std::fs::File::create("src/imgui_settings.ini").unwrap();
					file.write_all(buf.as_bytes()).unwrap();
	 			}

	 			/* Control camera by user input */
				if input_manager.keyboard.just_pressed(KeyCode::T) {
					if camera.grabbes_cursor {
						camera.grabbes_cursor = false;
						input_manager.mouse.release_cursor(&graphics);
					} else {
						camera.grabbes_cursor = true;
						input_manager.mouse.grab_cursor(&graphics);
					}
				}

	 			/* Update ImGui stuff */
				graphics.imguiw
					.prepare_frame(graphics.imguic.io_mut(), window.window())
					.unwrap();
 
				/* Moves to `RedrawRequested` stage */
				window.window().request_redraw();
	 		},
			glium::glutin::event::Event::RedrawRequested(_) => {
				let draw_data = {
					let ui = graphics.imguic.frame();
					imgui::Window::new("Camera")
						.resizable(false)
						.movable(false)
						.collapsible(false)
						.build(&ui, || {
							ui.text("Position");
							ui.text(format!("x: {0:.3}, y: {1:.3}, z: {2:.3}", camera.get_x(), camera.get_y(), camera.get_z()));
							ui.separator();
							ui.text("Speed factor");
							imgui::Slider::new("Camera speed", 5.0, 50.0)
								.display_format("%.1f")
								.build(&ui, &mut camera.speed);
							imgui::Slider::new("Camera fov", 0.0, std::f32::consts::PI * 4.0)
								.display_format("%.2f")
								.build(&ui, &mut camera.fov);
						});
	
					graphics.imguiw.prepare_render(&ui, graphics.display.gl_window().window());
					ui.render()
				};

				/* Uniforms set */
				let uniforms = uniform! {
					/* Texture uniform with filtering */
					tex: texture.with_mips(),
					time: time,
					proj: camera.get_proj(),
					view: camera.get_view()
				};

				let params = glium::DrawParameters {
					depth: glium::Depth {
						test: glium::DepthTest::Overwrite,
						write: true,
						.. Default::default()
					},
					backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
					.. Default::default()
				};

				/* Actual drawing */
				let mut target = graphics.display.draw(); 
				target.clear_color(0.01, 0.01, 0.01, 1.0);
				target.clear_depth(0.0); {
					target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &params).unwrap();

					graphics.imguir
						.render(&mut target, draw_data)
						.expect("error rendering imgui");

				} target.finish().unwrap();
			},
			glium::glutin::event::Event::NewEvents(_) => {
				/* Update time */
				let now = std::time::Instant::now();
				graphics.imguic
					.io_mut()
					.update_delta_time(now.duration_since(last_frame));
				_dt = now.duration_since(last_frame).as_secs_f64();
				last_frame = now;
				time += _dt;
				
				/* Rotating camera */
				camera.update(&mut input_manager, _dt);

				/* Input update */
				input_manager.update(&graphics);				
			},
			_ => ()
		}
	});
}