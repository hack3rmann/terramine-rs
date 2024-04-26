pub mod utils;
pub mod plugin;

use {
    crate::{
        prelude::*,
        graphics::{Graphics, GraphicsPlugin},
        camera::{*, self},
        components::Name,
    },
    winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoopWindowTarget,
        window::WindowId,
    },
};



/// Struct that handles [application][App] stuff.
pub struct App {
    pub world: World,
    pub startup_is_done: bool,
}

impl App {
    /// Drives an [app][App] to completion.
    /// 
    /// # Note
    /// 
    /// If executable finishes with success it will stop inside this function. It implies that this
    /// function returns only [`Err`] if something gone wrong and never returns in success case.
    pub fn drive() -> AnyResult<()> {
        RUNTIME.block_on(Self::new())?.run()
    }

    /// Constructs new [`App`].
    pub async fn new() -> AnyResult<Self> {
        logger::scope!(from = "app", "new()");

        let mut app = Self {
            world: World::new(),
            startup_is_done: false,
        };

        app.setup().await?;

        Ok(app)
    }

    pub async fn insert_plugin(&mut self, plugin: impl Plugin) -> AnyResult<()> {
        plugin.init(&mut self.world).await
    }

    pub async fn init_plugin<P: Plugin + Default>(&mut self) -> AnyResult<()> {
        self.insert_plugin(P::default()).await
    }

    /// Setups an [`App`] after creation.
    #[profile]
    pub async fn setup(&mut self) -> AnyResult<()> {
        logger::scope!(from = "app", "setup()");

        self.world.insert_resource(Name::new("Resources"));

        self.init_plugin::<GraphicsPlugin>().await?;
        self.init_plugin::<CameraPlugin>().await?;

        self.world.init_resource::<Timer>();
        self.world.init_resource::<AssetLoader>();

        self.world.resource::<&mut graphics::RenderGraph>()?
            .add(graphics::RenderNode::new(
                crate::terrain::chunk::array::render::render, "chunk-array",
            ));

        {
            use crate::terrain::chunk::{array::ChunkArray, mesh};

            let mut array = ChunkArray::new_empty(vecs!(2, 2, 2));
            array.generate();

            let mesh = mesh::make(&array);

            self.world.spawn((array, mesh, Name::new("Chunk array")));
        }

        Ok(())
    }

    /// Runs an [app][App]. Runs [`winit`]'s `event_loop`.
    pub fn run(mut self) -> AnyResult<()> {
        let event_loop = self.world.resource::<&mut Graphics>()
            .expect("failed to get graphics")
            .window
            .take_event_loop();

        event_loop.run(move |event, elw_target| {
            let result = RUNTIME.block_on(
                self.run_frame_loop(event, elw_target)
            );

            if let Err(error) = result {
                logger::error!(from = "app", "failed to run event loop: {error:#?}");
                
                if cfg::app::PANIC_ON_ERROR {
                    panic!("panicked on {error}");
                }
            }
        })?;

        Ok(())
    }

    /// Event loop run function.
    pub async fn run_frame_loop(
        &mut self, event: Event<()>,
        target: &EventLoopWindowTarget<()>,
    ) -> AnyResult<()> {
        if !self.startup_is_done {
            self.run_startup_systems().await?;
            self.startup_is_done = true;
        }

        let cur_window_id = {
            Graphics::handle_event(&self.world, &event)?;

            let graphics = self.world.resource::<&mut Graphics>()?;

            user_io::handle_event(&event, &graphics.window);

            graphics.window.id()
        };

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }
                if window_id == cur_window_id =>
            {
                target.exit();
            },

            Event::NewEvents(_) => {
                if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
                    target.exit();
                    return Ok(());
                }

                if keyboard::just_pressed(cfg::key_bindings::SPAWN_CAMERA) {
                    self.world.spawn(camera::make_new_camera_bundle());
                }

                let graphics = self.world.resource::<&Graphics>()?;
                graphics.window.request_redraw();
            },

            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } => {
                self.do_frame(window_id).await?;

                let best_time = Duration::from_secs_f32(1.0 / 120.0);
                let time_step = self.world.resource::<&Timer>()?.time_step();

                if time_step < best_time {
                    tokio::time::sleep(best_time - time_step).await;
                }
            }

            _ => (),
        }

        Ok(())
    }

    async fn run_startup_systems(&mut self) -> AnyResult<()> {
        Ok(())
    }

    /// Updates `ecs`'s systems. Note that non of resources can be borrowed at this point.
    #[profile]
    async fn update_systems(&mut self) -> AnyResult<()> {
        use crate::{physics::PhysicalComponent, terrain::chunk::array::render};

        // ignoring error, because it can probably setup next frame
        _ = render::try_setup_pipeline(&mut self.world);

        camera::update(&self.world)?;
        Graphics::update(&mut self.world)?;
        PhysicalComponent::update_all(&self.world)?;
        graphics::GpuMesh::make_renderable(&mut self.world);

        Ok(())
    }

    /// Updates all in the [app][App].
    async fn update(&mut self, _window_id: WindowId) -> AnyResult<()> {
        update::run();

        self.world.resource::<&mut AssetLoader>()?
            .try_finish_all().await;

        {
            self.world.resource::<&mut Timer>()?.update();
            let graphics = self.world.resource::<&Graphics>()?;
            
            mouse::update(&graphics.window)
                .log_error("app", "failed to update mouse input");

            if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
                let MainCamera(camera) = self.world.copy_resource::<MainCamera>()?;

                mouse::set_capture(
                    &graphics.window,
                    CameraHandle::switch_mouse_capture(&self.world, camera),
                )
            }
        }

        self.update_systems().await?;

        Ok(())
    }

    /// Prepares a frame.
    async fn prepare_frame(&mut self, _window_id: WindowId) -> AnyResult<()> {
        let fps = self.world.resource::<&Timer>()?.fps();
        let mut graphics = self.world.resource::<&mut Graphics>()?;

        // Prepare frame to render.
        graphics.prepare_frame(fps);

        Ok(())
    }

    /// Draws a frame on main window.
    #[profile]
    async fn draw_frame(&mut self, _window_id: WindowId) -> AnyResult<()> {
        let mut graphics = self.world.resource::<&mut Graphics>()?;

        graphics.render(&self.world)?;

        Ok(())
    }

    /// Does a frame.
    async fn do_frame(&mut self, window_id: WindowId) -> AnyResult<()> {
        // Skip a frame if the window is not main.
        {
            let graphics = self.world.resource::<&Graphics>()?;
            ensure_or!(window_id == graphics.window.id(), return Ok(()));
        }

        self.update(window_id).await?;
        self.prepare_frame(window_id).await?;
        self.draw_frame(window_id).await?;

        Ok(())
    }
}



pub mod update {
    use crate::prelude::*;

    pub type UpdateFunctionVec<const N: usize> = SmallVec<[fn(); N]>;

    pub static UPDATE_FUNCTIONS: Mutex<UpdateFunctionVec<64>> = const_default();

    /// Adds update function to update list.
    /// That functon will be executed before each frame.
    pub fn push_function(function: fn()) {
        UPDATE_FUNCTIONS.lock().push(function);
    }

    /// Adds update function to update list without aquireing [`Mutex`]'s lock.
    /// That functon will be executed before each frame.
    /// 
    /// # Safety
    /// 
    /// - should be called on main thread.
    /// - there's no threads pushing update functions.
    pub unsafe fn push_function_lock_free(function: fn()) {
        UPDATE_FUNCTIONS
            .data_ptr()
            .as_mut()
            .unwrap_unchecked()
            .push(function);
    }

    pub fn run() {
        for &update in UPDATE_FUNCTIONS.lock().iter() {
            update();
        }
    }
}
