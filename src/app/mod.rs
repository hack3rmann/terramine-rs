pub mod utils;

use {
    crate::{
        prelude::*,
        graphics::{
            Graphics,
            camera::Camera,
            RenderDescriptor,
            debug_visuals,
        },
    },

    winit::{
        event::{Event, WindowEvent, StartCause},
        event_loop::{ControlFlow, EventLoopWindowTarget, EventLoop},
        window::WindowId,
    },
};

/// Struct that handles application stuff.
#[derive(Debug)]
pub struct App {
    graphics: Graphics,
    camera: Camera,
    draw_timer: Timer,
    update_timer: Timer,

    event_loop: Option<EventLoop<()>>,
    imgui_window_builders: Vec<fn(&imgui::Ui)>,
}

impl App {
    /// Constructs [`App`].
    pub async fn new() -> Self {
        let _work_guard = logger::work("app", "initialize");

        let event_loop = default();

        let graphics = Graphics::new(&event_loop)
            .await
            .expect("failed to create graphics");

        let camera = Camera::new()
            .with_position(0.0, 16.0, 2.0)
            .with_rotation(0.0, 0.0, std::f32::consts::PI);

        let imgui_window_builders = vec![
            logger::spawn_window,
            loading::spawn_info_window,
            crate::terrain::voxel::generator::spawn_control_window,
        ];

        Self {
            graphics,
            camera,
            event_loop: Some(event_loop),
            draw_timer: default(),
            update_timer: default(),
            imgui_window_builders,
        }
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) -> ! {
        let event_loop = self.event_loop.take().expect("failed to take event_loop twice");
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

        self.graphics.imgui.platform.handle_event(
            self.graphics.imgui.context.io_mut(),
            &self.graphics.window,
            &event
        );
        user_io::handle_event(&event, &self.graphics.window);

        match event {
            Event::WindowEvent { event, window_id }
                if window_id == self.graphics.window.id() => match event
            {
                WindowEvent::CloseRequested =>
                    *control_flow = ControlFlow::Exit,

                WindowEvent::Resized(new_size) => {
                    let (width, height) = (new_size.width, new_size.height);
                    self.graphics.on_window_resize(UInt2::new(width, height));
                },

                _ => (),
            },

            Event::MainEventsCleared =>
                self.main_events_cleared(control_flow).await,

            Event::RedrawRequested(window_id) =>
                self.redraw_requested(window_id).await,

            Event::NewEvents(start_cause) =>
                self.new_events(start_cause).await,

            _ => (),
        }
    }

    /// Main events cleared.
    async fn main_events_cleared(&mut self, control_flow: &mut ControlFlow) {
        // ImGui can capture keyboard, if needed.
        keyboard::set_input_capture(self.graphics.imgui.context.io().want_capture_keyboard);
        
        // Close window if `escape` pressed
        if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Control camera cursor grab.
        if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            if self.camera.grabbes_cursor {
                mouse::release_cursor(&self.graphics.window);
            } else {
                mouse::grab_cursor(&self.graphics.window);
            }
            self.camera.grabbes_cursor = !self.camera.grabbes_cursor;
        }

        if keyboard::just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            self.graphics.refresh_test_shader().await;
        }

        // Display FPS
        self.graphics.window.set_title(&format!("Terramine: {0:.0} FPS", self.draw_timer.fps));

        // Prepare ImGui to render a frame.
        self.graphics.imgui.platform
            .prepare_frame(self.graphics.imgui.context.io_mut(), &self.graphics.window)
            .expect("failed to prepare frame");

        // Moves to `RedrawRequested` stage
        self.graphics.window.request_redraw();
    }

    /// Renders an image.
    async fn redraw_requested(&mut self, window_id: WindowId) {
        if window_id != self.graphics.window.id() { return }

        // InGui draw data.
        let use_ui = |ui: &mut imgui::Ui| {
            // Camera window
            self.camera.spawn_control_window(ui);

            // Profiler window.
            profiler::update_and_build_window(ui, &self.draw_timer);

            // Draw all windows by callbacks.
            for builder in self.imgui_window_builders.iter() {
                builder(ui)
            }
        };

        self.graphics.render(
            RenderDescriptor {
                use_imgui_ui: use_ui,
                time: self.draw_timer.time,
            }
        ).expect("failed to render graphics");

        self.draw_timer.update();
        self.graphics.imgui.context
            .io_mut()
            .update_delta_time(self.draw_timer.duration());
    }

    /// Updates things.
    async fn new_events(&mut self, _start_cause: StartCause) {
        self.update_timer.update();

        // Rotating camera.
        self.camera.update(self.update_timer.dt);

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
        mouse::update(&self.graphics.window).await
            .log_error("app", "failed to update mouse input");
    }
}
