pub mod utils;

use {
    /* Other files */
    crate::app::utils::{
        cfg,
        user_io::{InputManager, KeyCode},
        graphics::{
            Graphics,
            camera::Camera,
            texture::Texture,
            debug_visuals::{self, DebugVisualized},
        },
        terrain::chunk::{
            chunk_array::{
                MeshlessChunkArray,
                MeshedChunkArray,
            },
            DetailedVertexVec,
        },
        time::timer::Timer,
        profiler,
        concurrency::promise::Promise,
        runtime::prelude::*,
        werror::prelude::*,
        concurrency::loading::Loading,
    },

    /* Glium includes */
    glium::{
        glutin::{
            event::{
                Event,
                WindowEvent,
            },
            event_loop::ControlFlow, dpi::PhysicalSize,
        },
        Surface,
        uniform
    },
};

/// Struct that handles everything.
pub struct App {
    /* Important stuff */
    input_manager: InputManager,
    graphics: Graphics,
    camera: DebugVisualized<Camera>,
    timer: Timer,
    window_size: PhysicalSize<u32>,

    /* Temp voxel */
    chunk_arr: Option<MeshedChunkArray<'static>>,

    /* Second layer temporary stuff */
    texture: Texture,
}

impl App {
    /// Constructs app struct.
    pub fn new() -> Self {
        /* Graphics initialization */
        let graphics = Graphics::initialize().wunwrap();
    
        /* Camera handle */
        let camera = DebugVisualized::new_camera(Camera::new().with_position(0.0, 0.0, 2.0), &graphics.display);
    
        /* Texture loading */
        let texture = Texture::from("src/image/texture_atlas.png", &graphics.display).wunwrap();

        App {
            chunk_arr: None,
            graphics,
            camera,
            texture,
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
        self.graphics.take_event_loop().run(move |event, _, control_flow| {
            runtime().block_on(async {
                self.run_frame_loop(event, control_flow).await
            })
        });
    }

    /// Event loop run function
    pub async fn run_frame_loop<'e>(&mut self, event: Event<'e, ()>, control_flow: &mut ControlFlow) {
        /* Event handlers */
        self.graphics.imguiw.handle_event(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window(), &event);
        self.input_manager.handle_event(&event, &self.graphics);

        /* This event handler */
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

            Event::MainEventsCleared => {
                self.main_events_cleared(control_flow).await;
            },

            Event::RedrawRequested(_) => {
                self.redraw_requested().await;
            },

            Event::NewEvents(_) => {
                self.new_events().await;
            },

            _ => ()
        }
    }

    /// Main events cleared.
    async fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        /* Close window is `escape` pressed */
        if self.input_manager.keyboard.just_pressed(KeyCode::Escape) {
            *control_flow = ControlFlow::Exit;
        }

        /* Control camera by user input */
        if self.input_manager.keyboard.just_pressed(KeyCode::T) {
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
        self.graphics.display.gl_window().window().set_title(format!("Terramine: {0:.0} FPS", self.timer.fps()).as_str());

        /* Update ImGui stuff */
        self.graphics.imguiw
            .prepare_frame(self.graphics.imguic.io_mut(), self.graphics.display.gl_window().window())
            .wunwrap();

        /* Moves to `RedrawRequested` stage */
        self.graphics.display.gl_window().window().request_redraw();
    }

    /// Prepares the frame.
    async fn redraw_requested(&mut self) {
        /* Chunk generation flag */
        // FIXME:
        let mut generate_chunks = false;
        static mut SIZES: [i32; 3] = cfg::terrain::default::WORLD_SIZES_IN_CHUNKS;
        static mut GENERATION_PERCENTAGE: Loading = Loading::none();

        /* InGui draw data */
        let draw_data = {
            /* Get UI frame renderer */
            let ui = self.graphics.imguic.frame();

            /* Camera window */
            self.camera.inner.spawn_control_window(&ui, &mut self.input_manager);

            /* Profiler window */
            profiler::update_and_build_window(&ui, &self.timer, &self.input_manager);

            /* Chunk generation window */
            Self::spawn_chunk_generation_window(
                &ui, self.chunk_arr.is_some(), self.window_size.width as f32, self.window_size.height as f32,
                &mut generate_chunks, unsafe { &mut SIZES }, unsafe { GENERATION_PERCENTAGE }
            );
            if unsafe { GENERATION_PERCENTAGE } == Loading::none() {
                if self.input_manager.keyboard.just_pressed_combo(&[KeyCode::LControl, KeyCode::G]) {
                    generate_chunks = true;
                }

                if self.input_manager.keyboard.just_pressed_combo(&[KeyCode::LControl, KeyCode::H]) {
                    unsafe { SIZES = [16, 8, 16] };
                    generate_chunks = true;
                }
            }

            /* Render UI */
            self.graphics.imguiw.prepare_render(&ui, self.graphics.display.gl_window().window());

            ui.render()
        };

        /* Uniforms set */
        let uniforms = uniform! {
            /* Texture uniform with filtering */
            tex: self.texture.with_mips(),
            time: self.timer.time(),
            proj: self.camera.inner.get_proj(),
            view: self.camera.inner.get_view()
        };

        /* Actual drawing */
        let mut target = self.graphics.display.draw(); 
        target.clear_all(cfg::shader::CLEAR_COLOR, cfg::shader::CLEAR_DEPTH, cfg::shader::CLEAR_STENCIL); {
            if let Some(chunk_arr) = self.chunk_arr.as_mut() {
                chunk_arr.render(&mut target, &uniforms, &self.camera.inner).wunwrap();
            }

            self.camera.render_camera(&self.graphics.display, &mut target, &uniforms).wunwrap();

            self.graphics.imguir
                .render(&mut target, draw_data)
                .wexpect("Error rendering imgui");

        } target.finish().wunwrap();

        /* Chunk reciever */
        // FIXME:
        static mut CHUNKS_PROMISE: Option<Promise<(MeshlessChunkArray, Vec<DetailedVertexVec>)>> = None;
        static mut PERCENTAGE_PROMISE: Option<Promise<Loading>> = None;
        if generate_chunks {
            /* Dimensions shortcut */
            let (width, height, depth) = {
                let [width, height, depth] = unsafe { SIZES };
                (width as usize, height as usize, depth as usize)
            };

            /* Get receivers */
            let (array, percentage) = MeshlessChunkArray::generate(width, height, depth);

            /* Write to statics */
            unsafe {
                (CHUNKS_PROMISE, PERCENTAGE_PROMISE) = (Some(array), Some(percentage))
            };
        }

        /* If array recieved then store it in self */
        if let Some(promise) = unsafe { CHUNKS_PROMISE.as_ref() } {
            promise.poll_do_cleanup(|array| {
                /* Apply meshes to chunks */
                let array = {
                    let (array, meshes) = array;
                    array.to_meshed(&self.graphics, meshes)
                };

                /* Move result */
                self.chunk_arr = Some(array);
            }, || unsafe { CHUNKS_PROMISE = None });
        }

        /* Receive percentage */
        if let Some(promise) = unsafe { PERCENTAGE_PROMISE.as_ref() } {
            if let Some(percent) = promise.iter().last() {
                unsafe { GENERATION_PERCENTAGE = percent }
            }
        }

        // FIXME:
        unsafe {
            static mut TEMP: bool = false;
            if self.input_manager.keyboard.just_pressed(KeyCode::R) { TEMP = !TEMP; }
            if let Some(ref mut chunk_array) = self.chunk_arr {
                if !TEMP {
                    chunk_array.update_chunks_details(&self.graphics.display, &self.camera.inner);
                    //TEMP = true;
                }
            }
        }
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
        if self.input_manager.keyboard.just_pressed(KeyCode::F3) {
            debug_visuals::switch_enable();
        }

        /* Input update */
        self.input_manager.update(&self.graphics);		
    }

    /// Spawns chunk generation window.
    pub fn spawn_chunk_generation_window(ui: &imgui::Ui, inited: bool, width: f32, height: f32, generate_chunks: &mut bool, sizes: &mut [i32; 3], gen_percent: Loading) {
        if !inited {
            imgui::Window::new("Chunk generator")
                .position_pivot([0.5, 0.5])
                .position([width * 0.5, height as f32 * 0.5], imgui::Condition::Always)
                .movable(false)
                .size_constraints([150.0, 100.0], [300.0, 200.0])
                .always_auto_resize(true)
                .save_settings(false)
                .build(&ui, || {
                    ui.text("How many?");
                    ui.input_int3("Sizes", sizes)
                        .auto_select_all(true)
                        .enter_returns_true(true)
                        .build();
                    *generate_chunks = ui.button("Generate");

                    if gen_percent != Loading::none() {
                        imgui::ProgressBar::new(gen_percent.percent as f32)
                            .overlay_text(format!(
                                "{state} ({percent:.1}%)",
                                percent = gen_percent.percent * 100.0,
                                state = gen_percent.state
                            ))
                            .build(&ui);
                    }
                });
        }
    }
}
