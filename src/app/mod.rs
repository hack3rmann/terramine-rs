pub mod utils;

use {
    crate::{
        prelude::*,
        graphics::{
            Graphics,
            RenderDescriptor,
            debug_visuals,
        },
        camera::*,
    },

    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoopWindowTarget, EventLoop},
        window::WindowId,
    },
};

/// Struct that handles application stuff.
pub struct App {
    world: World,
    camera: Camera,

    imgui_window_builders: Vec<fn(&imgui::Ui)>,
    event_loop: Option<EventLoop<()>>,
}

impl App {
    /// Constructs [`App`].
    pub async fn new() -> Self {
        let _work_guard = logger::work("app", "initialize");

        let mut world = World::new();

        world.init_resource::<Timer>();

        let event_loop = EventLoop::default();
        world.insert_resource(
            Graphics::new(&event_loop).await
                .expect("failed to create graphics")
        );

        let camera = Camera::spawn_default(&mut world);
        
        Camera::spawn_default(&mut world);
        Camera::spawn_default(&mut world);
        Camera::spawn_default(&mut world);
        Camera::spawn_default(&mut world);

        let imgui_window_builders = vec![
            logger::spawn_window,
            loading::spawn_info_window,
            crate::terrain::voxel::generator::spawn_control_window,
        ];

        Self {
            camera,
            world,
            event_loop: Some(event_loop),
            imgui_window_builders,
        }
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) -> ! {
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, elw_target, control_flow| RUNTIME.block_on(
            self.run_frame_loop(event, elw_target, control_flow)
        ))
    }

    /// Event loop run function.
    pub async fn run_frame_loop(
        &mut self, event: Event<'_, ()>,
        _elw_target: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow,
    ) {
        *control_flow = ControlFlow::Poll;
        let mut graphics = self.world.resource::<&mut Graphics>().unwrap();

        graphics.handle_event(&event);
        user_io::handle_event(&event, &graphics.window);

        match event {
            Event::WindowEvent { event, window_id }
                if window_id == graphics.window.id() => match event
            {
                WindowEvent::CloseRequested =>
                    *control_flow = ControlFlow::Exit,

                WindowEvent::Resized(new_size) => {
                    let (width, height) = (new_size.width, new_size.height);
                    graphics.on_window_resize(UInt2::new(width, height));
                },

                _ => (),
            },

            Event::MainEventsCleared => {
                if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
                    control_flow.set_exit();
                }
                graphics.window.request_redraw();
            },

            Event::RedrawRequested(window_id) => {
                drop(graphics);
                self.do_frame(window_id).await
            },

            _ => (),
        }
    }

    async fn update_systems(&mut self, _window_id: WindowId) {
        let duration = {
            let mut timer = self.world.resource::<&mut Timer>().unwrap();
            timer.update();
            timer.duration()
        };

        Camera::update_all(&self.world);

        {
            let mut graphics = self.world.resource::<&mut Graphics>().unwrap();

            mouse::update(&graphics.window).await
                .log_error("app", "failed to update mouse input");

            graphics.imgui.context
                .io_mut()
                .update_delta_time(duration);
        }

        // Debug visuals switcher.
        if keyboard::just_pressed(cfg::key_bindings::DEBUG_VISUALS_SWITCH) {
            debug_visuals::switch_enable();
        }

        // Loading recieve.
        loading::recv_all()
            .log_error("app", "failed to receive all loadings");

        // Log messages receive.
        logger::recv_all();

        // Update keyboard inputs.
        keyboard::update_input();
    }

    async fn prepare_frame(&mut self, _window_id: WindowId) {
        let fps = self.world.resource::<&Timer>().unwrap().fps;
        let mut graphics = self.world.resource::<&mut Graphics>().unwrap();

        graphics.window.set_title(&format!("Terramine: {fps:.0} FPS"));

        keyboard::set_input_capture(graphics.imgui.context.io().want_capture_keyboard);

        if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            if self.camera.switch_mouse_capture(&self.world) {
                mouse::grab_cursor(&graphics.window);
            } else {
                mouse::release_cursor(&graphics.window);
            }
        }

        if keyboard::just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            graphics.refresh_test_shader().await;
        }

        // Prepare frame to render.
        graphics.prepare_frame().expect("failed to prepare a frame");
    }

    async fn draw_frame(&mut self, _window_id: WindowId) {
        let mut graphics = self.world.resource::<&mut Graphics>().unwrap();
        let timer = self.world.resource::<&Timer>().unwrap();

        // InGui draw data.
        let use_ui = |ui: &mut imgui::Ui| {
            // Camera window.
            Camera::spawn_control_windows(&self.world, ui);

            // Profiler window.
            profiler::update_and_build_window(ui, timer.dt);

            // Draw all windows by callbacks.
            for builder in self.imgui_window_builders.iter() {
                builder(ui)
            }
        };

        graphics.render(
            RenderDescriptor {
                use_imgui_ui: use_ui,
                time: timer.time,
            }
        ).expect("failed to render graphics");
    }

    // Does a frame.
    async fn do_frame(&mut self, window_id: WindowId) {
        {
            let graphics = self.world.resource::<&Graphics>().unwrap();
            if window_id != graphics.window.id() { return }
        }

        self.update_systems(window_id).await;
        self.prepare_frame(window_id).await;
        self.draw_frame(window_id).await;
    }
}
