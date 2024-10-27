pub mod utils;

use {
    /* Other files */
    crate::{
        graphics::{
            self,
            camera::Camera,
            debug_visuals::{self, DebugVisualizedStatic},
            light::DirectionalLight,
            texture::Texture,
            Graphics,
        },
        prelude::*,
        terrain::chunk::{chunk_array::ChunkArray, ChunkDrawBundle},
    },
    /* Glium includes */
    glium::{
        glutin::{
            event::{Event, StartCause, WindowEvent},
            event_loop::{ControlFlow, EventLoopWindowTarget},
            window::WindowId,
        },
        Surface,
    },
};

/// Struct that handles everything.
pub struct App {
    graphics: Graphics,
    camera: DebugVisualizedStatic<Camera>,
    lights: [DirectionalLight; 5],
    render_shadows: bool,
    draw_timer: Timer,
    update_timer: Timer,

    chunk_arr: DebugVisualizedStatic<ChunkArray>,
    chunk_draw_bundle: ChunkDrawBundle<'static>,

    texture_atlas: Texture,
    normal_atlas: Texture,

    imgui_window_builders: Vec<fn(&imgui::Ui)>,
}

impl App {
    /// Constructs [`App`].
    pub async fn new() -> Self {
        let _work_guard = logger::work("app", "initialize");

        let graphics = Graphics::new().expect("failed to create graphics");

        let camera = DebugVisualizedStatic::new_camera(
            Camera::new().with_position(0.0, 16.0, 2.0).with_rotation(
                0.0,
                0.0,
                std::f32::consts::PI,
            ),
            graphics.display.as_ref().get_ref(),
        );

        let texture_atlas = Texture::from_path(
            "src/image/texture_atlas.png",
            graphics.display.as_ref().get_ref(),
        )
        .expect("path should be valid and file is readable");

        let normal_atlas = Texture::from_path(
            "src/image/normal_atlas.png",
            graphics.display.as_ref().get_ref(),
        )
        .expect("path should be valid and file is readable");

        let chunk_draw_bundle = ChunkDrawBundle::new(graphics.display.as_ref().get_ref());
        let chunk_arr = DebugVisualizedStatic::new_chunk_array(
            ChunkArray::new_empty(),
            graphics.display.as_ref().get_ref(),
        )
        .await;

        let imgui_window_builders = vec![
            logger::spawn_window,
            loading::spawn_info_window,
            crate::terrain::voxel::generator::spawn_control_window,
        ];

        Self {
            chunk_arr,
            chunk_draw_bundle,
            graphics,
            camera,
            lights: Default::default(),
            render_shadows: false,
            texture_atlas,
            normal_atlas,
            draw_timer: Timer::new(),
            update_timer: Timer::new(),
            imgui_window_builders,
        }
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) -> ! {
        let event_loop = self.graphics.take_event_loop();
        event_loop.run(move |event, elw_target, control_flow| {
            RUNTIME.block_on(self.run_frame_loop(event, elw_target, control_flow))
        })
    }

    /// Event loop run function.
    pub async fn run_frame_loop(
        &mut self,
        event: Event<'_, ()>,
        _elw_target: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        self.graphics.imguip.handle_event(
            self.graphics.imguic.io_mut(),
            self.graphics.display.gl_window().window(),
            &event,
        );
        user_io::handle_event(&event, self.graphics.display.gl_window().window());

        match event {
            /* Window events */
            Event::WindowEvent { event, .. } => match event {
                /* Close event */
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    self.chunk_arr.drop_tasks();
                }

                WindowEvent::Resized(new_size) => {
                    let (width, height) = (new_size.width, new_size.height);
                    self.camera.aspect_ratio = height as f32 / width as f32;

                    for light in self.lights.iter_mut() {
                        light.cam.aspect_ratio = 1.0;
                    }

                    self.graphics
                        .on_window_resize(UInt2::new(width, height))
                        .expect("failed to update graphics with new window size");
                }

                _ => (),
            },

            Event::MainEventsCleared => self.main_events_cleared(control_flow).await,

            Event::RedrawRequested(window_id) => self.redraw_requested(window_id).await,

            Event::NewEvents(start_cause) => self.new_events(start_cause).await,

            _ => (),
        }
    }

    /// Main events cleared.
    async fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        // ImGui can capture keyboard, if needed.
        keyboard::set_input_capture(self.graphics.imguic.io().want_text_input);

        // Close window if `escape` pressed
        if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
            *control_flow = ControlFlow::Exit;
            self.chunk_arr.drop_tasks();
            return;
        }

        if keyboard::just_pressed(Key::Y) {
            self.chunk_arr.drop_tasks();
        }

        // Control camera by user input
        if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            if self.camera.grabbes_cursor {
                mouse::release_cursor(self.graphics.display.gl_window().window());
            } else {
                mouse::grab_cursor(self.graphics.display.gl_window().window());
            }

            self.camera.grabbes_cursor = !self.camera.grabbes_cursor;
        }

        if keyboard::just_pressed(cfg::key_bindings::SWITCH_RENDER_SHADOWS) {
            self.render_shadows = !self.render_shadows;
        }

        if keyboard::just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            self.chunk_draw_bundle = ChunkDrawBundle::new(self.graphics.display.as_ref().get_ref());

            self.graphics
                .refresh_postprocessing_shaders()
                .log_error("app", "failed to reload postprocessing shaders");

            match Texture::from_path(
                "src/image/normal_atlas.png",
                self.graphics.display.as_ref().get_ref(),
            ) {
                Ok(normals) => self.normal_atlas = normals,
                Err(err) => {
                    logger::log!(Error, from = "app", "failed to reload normal atlas: {err}")
                }
            }
        }

        // Update save/load tasks of `ChunkArray`
        self.chunk_arr
            .update(self.graphics.display.as_ref().get_ref(), &self.camera)
            .await
            .log_error("app", "failed to update chunk array");

        let window = self.graphics.display.gl_window();
        let window = window.window();

        // Display FPS
        window.set_title(&format!("Terramine: {0:.0} FPS", self.draw_timer.fps));

        // Update ImGui stuff
        self.graphics
            .imguip
            .prepare_frame(self.graphics.imguic.io_mut(), window)
            .expect("failed to prepare frame");

        // Moves to `RedrawRequested` stage
        window.request_redraw();
    }

    /// Prepares the frame.
    async fn redraw_requested(&mut self, _window_id: WindowId) {
        // InGui draw data
        let draw_data = {
            // Get UI frame renderer
            let ui = self.graphics.imguic.new_frame();

            // Camera window
            self.camera.spawn_control_window(ui);

            // Profiler window
            profiler::update_and_build_window(ui, &self.draw_timer);

            // Render UI
            self.graphics
                .imguip
                .prepare_render(ui, self.graphics.display.gl_window().window());

            // Chunk array control window
            self.chunk_arr.spawn_control_window(ui);

            // Draw all windows by callbacks
            for builder in self.imgui_window_builders.iter() {
                builder(ui)
            }

            // Light control window
            for light in self.lights.iter_mut().take(1) {
                light.spawn_control_window(ui);
            }

            self.graphics.imguic.render()
        };

        let resolution = vec2::from(UInt2::from(
            self.graphics.display.get_framebuffer_dimensions(),
        ));

        graphics::draw! {
            render_shadows: self.render_shadows,
            self.graphics,
            self.graphics.display.draw(),

            let uniforms = {
                screen_resolution: resolution.as_array(),

                texture_atlas: self.texture_atlas.get_sampler(),
                normal_atlas:  self.normal_atlas.get_sampler(),

                light_proj0: self.lights[0].cam.get_ortho(64.0, 64.0),
                light_view0: self.lights[0].cam.get_view(),
                light_dir0:  self.lights[0].cam.front.as_array(),
                light_pose0: self.lights[0].cam.pos.as_array(),

                light_proj1: self.lights[1].cam.get_ortho(512.0, 512.0),
                light_view1: self.lights[1].cam.get_view(),
                light_dir1:  self.lights[1].cam.front.as_array(),
                light_pose1: self.lights[1].cam.pos.as_array(),

                time: self.draw_timer.time,
                cam_pos: self.camera.pos.as_array(),
                proj: self.camera.get_proj(),
                view: self.camera.get_view(),
            },

            |&mut frame_buffer| {
                let display = self.graphics.display.as_ref().get_ref();

                self.chunk_arr.render(frame_buffer, &self.chunk_draw_bundle, &uniforms, display, &mut self.camera)
                    .await
                    .log_error("app", "failed to render chunk array");

                self.chunk_arr.render_chunk_debug(display, frame_buffer, &uniforms)
                    .await
                    .log_error("app", "failed to render chunk array debug visuals");

                self.camera.render_camera_debug_visuals(display, frame_buffer, &uniforms)
                    .log_error("app", "failed to render camera");
            },

            |mut target| {
                self.graphics.imguir.render(&mut target, draw_data)
                    .log_error("app", "failed to render imgui");
            },
        };

        self.draw_timer.update();
        self.graphics
            .imguic
            .io_mut()
            .update_delta_time(self.draw_timer.duration());
    }

    /// Updates things.
    async fn new_events(&mut self, _start_cause: StartCause) {
        self.update_timer.update();

        // Rotating camera.
        self.camera.update(self.update_timer.dt);
        for light in self.lights.iter_mut() {
            light.update(self.camera.pos);
        }
        // Debug visuals switcher.
        if keyboard::just_pressed(cfg::key_bindings::DEBUG_VISUALS_SWITCH) {
            debug_visuals::switch_enable();
        }

        // Loading recieve.
        loading::recv_all().log_error("app", "failed to receive all loadings");

        // Log messages receive.
        logger::recv_all();

        // Update keyboard inputs.
        keyboard::update_input();
        mouse::update(self.graphics.display.gl_window().window())
            .log_error("app", "failed to update mouse input");
    }
}
