pub mod utils;

use {
    /* Other files */
    crate::app::utils::{
        cfg,
        concurrency::loading,
        user_io::InputManager,
        graphics::{
            light::DirectionalLight,
            self,
            Graphics,
            camera::Camera,
            texture::Texture,
            debug_visuals::{self, DebugVisualizedStatic},
        },
        terrain::chunk::{
            chunk_array::ChunkArray,
            ChunkDrawBundle,
        },
        time::timer::Timer,
        profiler,
        runtime::RUNTIME,
    },

    /* Glium includes */
    glium::{
        Surface,
        glutin::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
        },
    },

    math_linear::prelude::*,
};

/// Struct that handles everything.
pub struct App {
    input_manager: InputManager,
    graphics: Graphics,
    camera: DebugVisualizedStatic<Camera>,
    lights: [DirectionalLight; 3],
    render_shadows: bool,
    timer: Timer,

    chunk_arr: DebugVisualizedStatic<ChunkArray>,
    chunk_draw_bundle: ChunkDrawBundle<'static>,

    texture_atlas: Texture,
    normal_atlas: Texture,
}

impl App where Self: 'static {
    /// Constructs app struct.
    pub fn new() -> Self {
        let graphics = Graphics::new()
            .expect("failed to create graphics");

        let camera = DebugVisualizedStatic::new_camera(
            Camera::new()
                .with_position(0.0, 16.0, 2.0)
                .with_rotation(0.0, 0.0, std::f64::consts::PI),
            graphics.display.as_ref().get_ref(),
        );

        let texture_atlas = Texture::from_path("src/image/texture_atlas.png", graphics.display.as_ref().get_ref())
            .expect("path should be valid and file is readable");

        let normal_atlas = Texture::from_path("src/image/normal_atlas.png", graphics.display.as_ref().get_ref())
            .expect("path should be valid and file is readable");

        let chunk_draw_bundle = ChunkDrawBundle::new(graphics.display.as_ref().get_ref());
        let chunk_arr = DebugVisualizedStatic::new_chunk_array(
            ChunkArray::new_empty(),
            graphics.display.as_ref().get_ref(),
        );

        Self {
            chunk_arr,
            chunk_draw_bundle,
            graphics,
            camera,
            lights: Default::default(),
            render_shadows: false,
            texture_atlas,
            normal_atlas,
            timer: Timer::new(),
            input_manager: InputManager::new(),
        }
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) -> ! {
        // TODO: rewrite `loop { async {} }` into `async { loop {} }`.
        self.graphics.take_event_loop().run(move |event, _, control_flow|
            RUNTIME.block_on(
                self.run_frame_loop(event, control_flow)
            )
        )
    }

    /// Event loop run function.
    pub async fn run_frame_loop(&mut self, event: Event<'_, ()>, control_flow: &mut ControlFlow) {
        self.graphics.imguip.handle_event(
            self.graphics.imguic.io_mut(),
            self.graphics.display.gl_window().window(),
            &event
        );
        self.input_manager.handle_event(&event, &self.graphics);

        match event {
            /* Window events */
            Event::WindowEvent { event, .. } => match event {
                /* Close event */
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    self.chunk_arr.drop_tasks();
                },

                WindowEvent::Resized(new_size) => {
                    let (width, height) = (new_size.width, new_size.height);
                    self.camera.aspect_ratio = height as f32
                                             / width  as f32;
                    
                    for light in self.lights.iter_mut() {
                        light.cam.aspect_ratio = 1.0;//self.camera.aspect_ratio;
                    }

                    self.graphics.on_window_resize(UInt2::new(width, height));
                },

                _ => (),
            },

            Event::MainEventsCleared =>
                self.main_events_cleared(control_flow).await,

            Event::RedrawRequested(_) =>
                self.redraw_requested().await,

            Event::NewEvents(_) =>
                self.new_events().await,

            _ => ()
        }
    }

    /// FIXME: truncate large chunk generation task list.
    /// Main events cleared.
    async fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        use glium::glutin::event::VirtualKeyCode as Key;
        
        /* Close window is `escape` pressed */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::APP_EXIT) {
            *control_flow = ControlFlow::Exit;
            self.chunk_arr.drop_tasks();
        }

        if self.input_manager.keyboard.just_pressed(Key::Y) {
            self.chunk_arr.drop_tasks();
        }

        /* Control camera by user input */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            if self.camera.grabbes_cursor {
                self.input_manager.mouse.release_cursor(&self.graphics);
            } else {
                self.input_manager.mouse.grab_cursor(&self.graphics);
            }

            self.camera.grabbes_cursor = !self.camera.grabbes_cursor;
        }

        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::SWITCH_RENDER_SHADOWS) {
            self.render_shadows = !self.render_shadows;
        }

        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            self.chunk_draw_bundle = ChunkDrawBundle::new(self.graphics.display.as_ref().get_ref());
            self.graphics.refresh_postprocessing_shaders()
                .expect("failed to refresh postprocessing shaders");

            // FIXME:
            self.normal_atlas = Texture::from_path("src/image/normal_atlas.png", self.graphics.display.as_ref().get_ref())
                .expect("path should be valid and file is readable");
        }

        /* Update save/load tasks of `ChunkArray` */
        self.chunk_arr.update(&mut self.input_manager).await
            .expect("failed to update chunk array");

        let window = self.graphics.display.gl_window();
        let window = window.window();

        /* Display FPS */
        window.set_title(&format!("Terramine: {0:.0} FPS", self.timer.fps()));

        /* Update ImGui stuff */
        self.graphics.imguip
            .prepare_frame(self.graphics.imguic.io_mut(), window)
            .expect("failed to prepare frame");

        /* Moves to `RedrawRequested` stage */
        window.request_redraw();
    }

    /// Prepares the frame.
    async fn redraw_requested(&mut self) {
        /* InGui draw data */
        let draw_data = {
            /* Get UI frame renderer */
            let ui = self.graphics.imguic.new_frame();
            let keyboard = &mut self.input_manager.keyboard;

            /* Camera window */
            self.camera.spawn_control_window(ui, keyboard);

            /* Profiler window */
            profiler::update_and_build_window(ui, &self.timer, keyboard);

            /* Render UI */
            self.graphics.imguip.prepare_render(ui, self.graphics.display.gl_window().window());

            /* Chunk array control window */
            self.chunk_arr.spawn_control_window(ui, keyboard);

            /* Loadings window */
            loading::spawn_info_window(ui, keyboard);

            /* Light control window */
            for light in self.lights.iter_mut() {
                light.spawn_control_window(ui, keyboard);
            }

            self.graphics.imguic.render()
        };

        graphics::draw! {
            render_shadows: self.render_shadows,
            self.graphics,
            self.graphics.display.draw(),
            let uniforms = {
                texture_atlas: self.texture_atlas.with_mips(),
                normal_atlas:  self.normal_atlas.with_mips(),

                light_proj: self.lights[0].cam.get_ortho(64.0),
                light_view: self.lights[0].cam.get_view(),
                light_dir: self.lights[0].cam.front.as_array(),
                light_pos: self.lights[0].cam.pos.as_array(),

                time: self.timer.time(),
                cam_pos: self.camera.pos.as_array(),
                proj: self.camera.get_proj(),
                view: self.camera.get_view(),
            },

            |&mut frame_buffer| {
                let display = self.graphics.display.as_ref().get_ref();

                self.chunk_arr.render(frame_buffer, &self.chunk_draw_bundle, &uniforms, display, &self.camera)
                    .await
                    .expect("failed to render chunk array");
        
                self.camera.render_camera(display, frame_buffer, &uniforms)
                    .expect("failed to render camera");
            },

            |mut target| {
                self.graphics.imguir.render(&mut target, draw_data)
                    .expect("failed to render imgui");
            },
        };

        self.timer.update();
        self.graphics.imguic
            .io_mut()
            .update_delta_time(self.timer.duration());
    }

    /// Updates things.
    async fn new_events(&mut self) {
        /* Rotating camera */
        self.camera.update(&mut self.input_manager, self.timer.dt_as_f64());
        self.lights[0].update(self.camera.pos);

        /* Debug visuals switcher */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::DEBUG_VISUALS_SWITCH) {
            debug_visuals::switch_enable();
        }

        /* Input update */
        self.input_manager.update(&self.graphics);		

        /* Loading recieve */
        loading::recv_all()
            .expect("failed to receive all loadings");
    }
}
