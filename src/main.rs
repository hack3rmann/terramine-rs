mod utils;

/* Glium includes */
use glium::{
	glutin::event::{
		Event,
		WindowEvent,
		ElementState,
	},
	Surface,
	uniform
};

/* Other files */
use utils::{
	*,
	user_io::{InputManager, KeyCode},
	window::Window,
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
	let texture = Texture::from("src/image/testSprite.png", &graphics.display).unwrap();

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

	/* Time stuff */
	let time_start = std::time::Instant::now();
	let mut _time = time_start.elapsed().as_secs_f32();
	
	/* Camera rotation */
	let mut roll: f32 = Default::default();
	let mut pitch: f32 = Default::default();

	camera.set_position(0.0, 0.0, 2.0);

	/* Event loop run */
	graphics.take_event_loop().run(move |event, _, control_flow| {
		/* Exit if window have that message */
		match event {
			/* Window events */
            Event::WindowEvent { event, .. } => match event {
				/* Close event */
                WindowEvent::CloseRequested => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
				/* Keyboard input event */
				WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
					/* Key matching */
					Some(key) => match key {
						_ => {
							/* If key is pressed then press it on virtual keyboard, if not then release it. */
							match input.state {
								ElementState::Pressed => {
									input_manager.keyboard.press(key);
									return
								},
								ElementState::Released => {
									input_manager.keyboard.release(key);
									return
								}
							}
						}
					}
					_ => return
				},
				/* Mouse buttons match. */
				WindowEvent::MouseInput { button, state, .. } => match state {
					/* If button is pressed then press it on virtual mouse, if not then release it. */
					ElementState::Pressed => {
						input_manager.mouse.press(button);
						return
					},
					ElementState::Released => {
						input_manager.mouse.release(button);
						return
					}
				},
				/* Cursor entered the window event. */
				WindowEvent::CursorEntered { .. } => {
					input_manager.mouse.on_window = true;
					return
				},
				/* Cursor left the window. */
				WindowEvent::CursorLeft { .. } => {
					input_manager.mouse.on_window = false;
					return
				},
				/* Cursor moved to new position. */
				WindowEvent::CursorMoved { position, .. } => {
					input_manager.mouse.move_cursor(position.x as f32, position.y as f32);
					return
				}
                _ => return,
            },
			Event::MainEventsCleared => {
				/* Close window is `escape` pressed */
				if input_manager.keyboard.just_pressed(KeyCode::Escape) {
					Window::exit(control_flow);
				}

				/* Control camera by user input */
				if input_manager.keyboard.is_pressed(KeyCode::W)		{ camera.move_pos( 0.001,  0.0,    0.0); }
				if input_manager.keyboard.is_pressed(KeyCode::S)		{ camera.move_pos(-0.001,  0.0,    0.0); }
				if input_manager.keyboard.is_pressed(KeyCode::D)		{ camera.move_pos( 0.0,    0.0,   -0.001); }
				if input_manager.keyboard.is_pressed(KeyCode::A)		{ camera.move_pos( 0.0,    0.0,    0.001); }
				if input_manager.keyboard.is_pressed(KeyCode::LShift)	{ camera.move_pos( 0.0,   -0.001,  0.0); }
				if input_manager.keyboard.is_pressed(KeyCode::Space)	{ camera.move_pos( 0.0,    0.001,  0.0); }
				if input_manager.mouse.just_left_pressed() {
					camera.set_position(0.0, 0.0, 2.0);
					camera.reset_rotation();
					roll = 0.0;
					pitch = 0.0;
				}

				/* Camera rotation */
				pitch += input_manager.mouse.dx / 100.0;
				roll -= input_manager.mouse.dy / 100.0;

				/* Time refresh */
				_time = time_start.elapsed().as_secs_f32();

				/* Rotating camera */
				camera.set_rotation(roll, pitch, 0.0);
				input_manager.mouse.update();
			},
			_ => return,
		}
		/* Uniforms set */
		let uniforms = uniform! {
			/* Texture uniform with filtering */
			tex: texture.with_mips(),
			time: _time,
			proj: camera.get_proj(),
			view: camera.get_view()
		};

		/* Drawing process */
		let mut target = graphics.display.draw();
		target.clear_color(0.1, 0.1, 0.1, 1.0); {
			target.draw(&vertex_buffer, &indices, &shaders.program, &uniforms, &Default::default()).unwrap();
		} target.finish().unwrap();
	});
}