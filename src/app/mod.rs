pub mod utils;

use {
    /* Other files */
    crate::app::utils::{
        cfg,
        user_io::InputManager,
        graphics::{
            Graphics,
            camera::Camera,
            texture::Texture,
            debug_visuals::{self, DebugVisualized},
        },
        terrain::chunk::{
            chunk_array::ChunkArray,
            ChunkDrawBundle,
        },
        time::timer::Timer,
        profiler,
        runtime::prelude::*,
    },

    /* Glium includes */
    glium::{
        glutin::{
            event::{Event, WindowEvent},
            event_loop::ControlFlow,
            dpi::PhysicalSize,
        },
        Surface,
        uniform,
    },
};

/// Struct that handles everything.
#[derive(Debug)]
pub struct App {
    /* Important stuff */
    input_manager: InputManager,
    graphics: Graphics,
    camera: DebugVisualized<Camera>,
    timer: Timer,
    window_size: PhysicalSize<u32>,

    /* Temp voxel */
    chunk_arr: ChunkArray,
    chunk_draw_bundle: ChunkDrawBundle<'static>,

    /* Second layer temporary stuff */
    texture_atlas: Texture,
}

impl App {
    /// Constructs app struct.
    pub fn new() -> Self {
        let graphics = Graphics::initialize()
            .expect("graphics should be initialized once");

        let camera = DebugVisualized::new_camera(
            Camera::new()
                .with_position(0.0, 16.0, 2.0)
                .with_rotation(0.0, 0.0, 3.14),
            &graphics.display,
        );
    
        let texture_atlas = Texture::from_path("src/image/texture_atlas.png", &graphics.display)
            .expect("path should be valid and file is readable");

        let chunk_draw_bundle = ChunkDrawBundle::new(&graphics.display);
        let chunk_arr = ChunkArray::new(vecs!(16, 2, 8));

        App {
            chunk_arr,
            chunk_draw_bundle,
            graphics,
            camera,
            texture_atlas,
            timer: Timer::new(),
            input_manager: InputManager::new(),
            window_size: PhysicalSize::new(
                cfg::window::default::WIDTH  as u32,
                cfg::window::default::HEIGHT as u32,
            ),
        }
    }

    /// Runs app.
    pub fn run(mut self) {
        /* Event/game loop */
        // TODO: rewrite `loop { async {} }` into `async { loop {} }`.
        self.graphics.take_event_loop().run(move |event, _, control_flow| {
            runtime().block_on(async {
                self.run_frame_loop(event, control_flow).await
            })
        });
    }

    /// Event loop run function.
    pub async fn run_frame_loop(&mut self, event: Event<'_, ()>, control_flow: &mut ControlFlow) {
        self.graphics.imguiw.handle_event(
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
                },

                WindowEvent::Resized(new_size) => {
                    self.camera.inner.aspect_ratio = new_size.height as f32 / new_size.width as f32;
                    self.window_size = new_size;
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

    /// Main events cleared.
    async fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        /* Close window is `escape` pressed */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::APP_EXIT) {
            *control_flow = ControlFlow::Exit;
        }

        /* Control camera by user input */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            if self.camera.inner.grabbes_cursor {
                self.camera.inner.grabbes_cursor = false;
                self.input_manager.mouse.release_cursor(&self.graphics);
            }
            
            else {
                self.camera.inner.grabbes_cursor = true;
                self.input_manager.mouse.grab_cursor(&self.graphics);
            }
        }

        /* Display FPS */
        self.graphics.display.gl_window().window()
            .set_title(&format!("Terramine: {0:.0} FPS", self.timer.fps()));

        /* Update ImGui stuff */
        self.graphics.imguiw
            .prepare_frame(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window())
            .expect("failed to prepare frame");

        /* Moves to `RedrawRequested` stage */
        self.graphics.display.gl_window().window()
            .request_redraw();
    }

    /// Prepares the frame.
    async fn redraw_requested(&mut self) {
        /* InGui draw data */
        let draw_data = {
            /* Get UI frame renderer */
            let ui = self.graphics.imguic.frame();

            /* Camera window */
            self.camera.inner.spawn_control_window(&ui, &mut self.input_manager);

            /* Profiler window */
            profiler::update_and_build_window(&ui, &self.timer, &mut self.input_manager);

            /* Render UI */
            self.graphics.imguiw.prepare_render(&ui, self.graphics.display.gl_window().window());

            ui.render()
        };

        /* Uniforms set */
        let uniforms = uniform! {
            tex:  self.texture_atlas.with_mips(),
            time: self.timer.time(),
            proj: self.camera.inner.get_proj(),
            view: self.camera.inner.get_view(),
        };

        /* Actual drawing */
        let mut target = self.graphics.display.draw(); 
        target.clear_all(cfg::shader::CLEAR_COLOR, cfg::shader::CLEAR_DEPTH, cfg::shader::CLEAR_STENCIL);
        {
            self.chunk_arr.render(&mut target, &self.chunk_draw_bundle, &uniforms, &self.graphics.display, self.camera.inner.pos)
                .await
                .expect("failed to render chunk array");

            self.camera.render_camera(&self.graphics.display, &mut target, &uniforms)
                .expect("failed to render camera");

            // TODO: deal with this unique borrow.
            self.graphics.imguir.0.render(&mut target, draw_data)
                .expect("failed to render imgui");
        }
        target.finish().expect("failed to finish target");
    }

    /// Updates things.
    async fn new_events(&mut self) {
        /* Update time */
        self.timer.update();
        self.graphics.imguic
            .io_mut()
            .update_delta_time(self.timer.duration());
        
        /* Rotating camera */
        self.camera.inner.update(&mut self.input_manager, self.timer.dt_as_f64());

        /* Debug visuals switcher */
        if self.input_manager.keyboard.just_pressed(cfg::key_bindings::DEBUG_VISUALS_SWITCH) {
            debug_visuals::switch_enable();
        }

        /* Input update */
        self.input_manager.update(&self.graphics);		
    }

    // Spawns chunk generation window.
    // pub fn spawn_chunk_generation_window(
    //     ui: &imgui::Ui, inited: bool, width: f32, height: f32,
    //     generate_chunks: &mut bool, sizes: &mut [i32; 3], gen_percent: Loading
    // ) {
    //     return;
    //     if inited { return }

    //     imgui::Window::new("Chunk generator")
    //         .position_pivot([0.5, 0.5])
    //         .position([width * 0.5, height * 0.5], imgui::Condition::Always)
    //         .movable(false)
    //         .size_constraints([150.0, 100.0], [300.0, 200.0])
    //         .always_auto_resize(true)
    //         .save_settings(false)
    //         .build(&ui, || {
    //             ui.text("How many?");
    //             ui.input_int3("Sizes", sizes)
    //                 .auto_select_all(true)
    //                 .enter_returns_true(true)
    //                 .build();
    //             *generate_chunks = ui.button("Generate");

    //             if gen_percent != Loading::none() {
    //                 imgui::ProgressBar::new(gen_percent.percent as f32)
    //                     .overlay_text(format!(
    //                         "{state} ({percent:.1}%)",
    //                         percent = gen_percent.percent * 100.0,
    //                         state = gen_percent.state
    //                     ))
    //                     .build(&ui);
    //             }
    //         });
    // }
}
