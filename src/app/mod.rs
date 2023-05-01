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

    bevy_ecs::prelude::*,
};

#[derive(Debug, Resource, Deref, Default)]
pub struct DrawTimer(pub Timer);

#[derive(Debug, Resource, Deref, Default)]
pub struct UpdateTimer(pub Timer);

/// Struct that handles application stuff.
#[derive(Debug)]
pub struct App {
    world: World,

    imgui_window_builders: Vec<fn(&imgui::Ui)>,
}

impl App {
    /// Constructs [`App`].
    pub async fn new() -> Self {
        let _work_guard = logger::work("app", "initialize");

        let mut world = World::new();
        world.init_resource::<DrawTimer>();
        world.init_resource::<UpdateTimer>();

        let event_loop = EventLoop::default();
        world.insert_resource(
            Graphics::new(&event_loop).await
            .expect("failed to create graphics")
        );
        world.insert_non_send_resource(event_loop);
        
        world.insert_resource(
            Camera::new()
                .with_position(0.0, 16.0, 2.0)
                .with_rotation(0.0, 0.0, std::f32::consts::PI)
        );

        let imgui_window_builders = vec![
            logger::spawn_window,
            loading::spawn_info_window,
            crate::terrain::voxel::generator::spawn_control_window,
        ];

        Self {
            world,
            imgui_window_builders,
        }
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) -> ! {
        let event_loop = self.world.remove_non_send_resource::<EventLoop<()>>().unwrap();
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
        let mut graphics = self.world.get_resource_mut::<Graphics>().unwrap();

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
        let world = self.world.cell();
        let mut graphics = world.get_resource_mut::<Graphics>().unwrap();

        // ImGui can capture keyboard, if needed.
        keyboard::set_input_capture(graphics.imgui.context.io().want_capture_keyboard);
        
        // Close window if `escape` pressed
        if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Control camera cursor grab.
        if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
            let mut camera = world.get_resource_mut::<Camera>().unwrap();

            if camera.grabbes_cursor {
                mouse::release_cursor(&graphics.window);
            } else {
                mouse::grab_cursor(&graphics.window);
            }
            camera.grabbes_cursor = !camera.grabbes_cursor;
        }

        if keyboard::just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            graphics.refresh_test_shader().await;
        }

        // Display FPS.
        {
            let draw_timer = world.get_resource::<DrawTimer>().unwrap();
            graphics.window.set_title(&format!("Terramine: {0:.0} FPS", draw_timer.fps));
        }

        // Prepare frame to render.
        graphics.prepare_frame().expect("failed to prepare a frame");

        // Moves to `RedrawRequested` stage.
        graphics.window.request_redraw();
    }

    /// Renders an image.
    async fn redraw_requested(&mut self, window_id: WindowId) {
        use std::ops::DerefMut;

        let world = self.world.cell();
        let mut graphics = world.get_resource_mut::<Graphics>().unwrap();
        let mut draw_timer = world.get_resource_mut::<DrawTimer>().unwrap();

        if window_id != graphics.window.id() { return }

        // InGui draw data.
        let use_ui = |ui: &mut imgui::Ui| {
            // Camera window.
            world.get_resource_mut::<Camera>()
                .unwrap()
                .deref_mut()
                .spawn_control_window(ui);

            // Profiler window.
            profiler::update_and_build_window(ui, &draw_timer);

            // Draw all windows by callbacks.
            for builder in self.imgui_window_builders.iter() {
                builder(ui)
            }
        };

        graphics.render(
            RenderDescriptor {
                use_imgui_ui: use_ui,
                time: draw_timer.time,
            }
        ).expect("failed to render graphics");

        draw_timer.update();
        graphics.imgui.context
            .io_mut()
            .update_delta_time(draw_timer.duration());
    }

    /// Updates things.
    async fn new_events(&mut self, _start_cause: StartCause) {
        {
            let world = self.world.cell();
            let mut update_timer = world.get_resource_mut::<UpdateTimer>().unwrap();
            let mut camera = world.get_resource_mut::<Camera>().unwrap();

            update_timer.update();
            camera.update(update_timer.dt);
        }

        {
            let graphics = self.world.get_resource::<Graphics>().unwrap();
            mouse::update(&graphics.window).await
                .log_error("app", "failed to update mouse input");
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
}
