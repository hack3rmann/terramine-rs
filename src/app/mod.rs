pub mod utils;

use {
    crate::{
        prelude::*,
        graphics::{
            Graphics,
            RenderDescriptor,
        },
        camera::*,
    },

    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoopWindowTarget, EventLoop},
        window::WindowId,
    },
};



/// Struct that handles [application][App] stuff.
pub struct App {
    pub world: World,
    pub camera: CameraHandle,

    pub update_functions: Vec<fn()>,
    pub event_loop: Nullable<EventLoop<()>>,
}

impl App {
    /// Constructs [`App`].
    pub async fn new() -> Self {
        let _work_guard = logger::scope("app", "new");

        let mut app = Self {
            update_functions: vec![
                loading::update,
                logger::update,
                keyboard::update,
            ],
            camera: CameraHandle::from_entity_unchecked(Entity::DANGLING),
            world: World::new(),
            event_loop: Nullable::new(default()),
        };

        app.setup().await;

        app
    }

    /// Setups an [`App`] after creation.
    pub async fn setup(&mut self) {
        let _work_guard = logger::scope("app", "setup");


        self.world.init_resource::<Timer>();

        self.world.insert_resource(
            Graphics::new(&self.event_loop)
                .await
                .expect("failed to create graphics")
        );

        self.camera = CameraHandle::spawn_default(&mut self.world);


        let mut graphics = self.world.resource::<&mut Graphics>().unwrap();

        graphics.imgui.add_window_builder_bunch([
            logger::spawn_window,
            loading::spawn_info_window,
            crate::terrain::voxel::generator::spawn_control_window,
        ]);
    }

    /// Runs app. Runs glium's `event_loop`.
    pub fn run(mut self) -> ! {
        let event_loop = self.event_loop.take();
        event_loop.run(move |event, elw_target, control_flow| RUNTIME.block_on(
            self.run_frame_loop(event, elw_target, control_flow)
        ))
    }

    /// Exits app. Runs any destructor or deinitializer functions.
    pub async fn exit(&mut self, control_flow: &mut ControlFlow) {
        control_flow.set_exit();
    }

    /// Event loop run function.
    pub async fn run_frame_loop(
        &mut self, event: Event<'_, ()>,
        _elw_target: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow,
    ) {
        *control_flow = ControlFlow::Poll;

        let cur_window_id = {
            let mut graphics = self.world.resource::<&mut Graphics>().unwrap();

            graphics.handle_event(&event);
            user_io::handle_event(&event, &graphics.window);

            graphics.window.id()
        };

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }
                if window_id == cur_window_id =>
            {
                self.exit(control_flow).await;
            },

            Event::MainEventsCleared => {
                if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
                    self.exit(control_flow).await;
                    return;
                }

                let graphics = self.world.resource::<&Graphics>().unwrap();
                graphics.window.request_redraw();
            },

            Event::RedrawRequested(window_id) =>
                self.do_frame(window_id).await,

            _ => (),
        }
    }

    /// Updates `ecs`'s systems. Note that non of resources can be borrowed at this point.
    async fn update_systems(&mut self) {
        CameraHandle::update_all(&self.world);
    }

    /// Updates all in the [app][App].
    async fn update(&mut self, _window_id: WindowId) {
        for update in self.update_functions.iter() {
            update();
        }

        {
            let mut timer = self.world.resource::<&mut Timer>().unwrap();
            let mut graphics = self.world.resource::<&mut Graphics>().unwrap();
            
            timer.update();
            graphics.update(timer.dt()).await;

            mouse::update(&graphics.window).await
                .log_error("app", "failed to update mouse input");

            keyboard::set_input_capture(graphics.imgui.context.io().want_text_input);
    
            if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
                mouse::set_capture(
                    &graphics.window,
                    self.camera.switch_mouse_capture(&self.world),
                )
            }
        }

        self.update_systems().await;
    }

    /// Prepares a frame.
    async fn prepare_frame(&mut self, _window_id: WindowId) {
        let fps = self.world.resource::<&Timer>().unwrap().fps();
        let mut graphics = self.world.resource::<&mut Graphics>().unwrap();

        // Prepare frame to render.
        graphics.prepare_frame(fps)
            .expect("failed to prepare a frame");
    }

    /// Draws a frame on main window.
    async fn draw_frame(&mut self, _window_id: WindowId) {
        let timer = self.world.resource::<&Timer>().unwrap();

        // InGui draw data.
        let use_ui = |ui: &mut imgui::Ui| {
            CameraHandle::spawn_control_windows(&self.world, ui);
            profiler::update_and_build_window(ui, timer.dt());
        };

        let mut graphics = self.world.resource::<&mut Graphics>().unwrap();
        graphics.render(
            RenderDescriptor {
                use_imgui_ui: use_ui,
                time: timer.time(),
            }
        ).expect("failed to render graphics");
    }

    /// Does a frame.
    async fn do_frame(&mut self, window_id: WindowId) {
        // Skip a frame if the window is not main.
        {
            let graphics = self.world.resource::<&Graphics>().unwrap();
            if window_id != graphics.window.id() { return }
        }

        self.update(window_id).await;
        self.prepare_frame(window_id).await;
        self.draw_frame(window_id).await;
    }
}
