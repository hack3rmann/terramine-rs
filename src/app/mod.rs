pub mod utils;

use {
    crate::{
        prelude::*,
        graphics::Graphics,
        camera::{*, self},
        terrain::chunk::chunk_array::ChunkArray,
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
    pub event_loop: Nullable<EventLoop<()>>,
}

impl App {
    /// Drives an [app][App] to completion.
    /// 
    /// # Note
    /// 
    /// If executable finishes with success it will stop inside this function. It implies that this
    /// function returns only [`Err`] if something gone wrong and never returns in success case.
    pub fn drive() -> AnyResult<!> {
        RUNTIME.block_on(Self::new())?.run()
    }

    /// Constructs new [`App`].
    pub async fn new() -> AnyResult<Self> {
        logger::scope!(from = "app", "new()");

        let mut app = Self {
            world: World::new(),
            event_loop: Nullable::new(default()),
        };

        app.setup().await?;

        Ok(app)
    }

    /// Setups an [`App`] after creation.
    pub async fn setup(&mut self) -> AnyResult<()> {
        logger::scope!(from = "app", "setup()");

        self.world.init_resource::<Timer>();

        let camera = self.world.spawn(camera::make_new_enabled()).into();
        self.world.insert_resource(MainCamera(camera));

        let graphics = Graphics::new(&self.event_loop)
            .await
            .context("failed to create graphics")?;

        self.world.insert_resource(CameraUniformBuffer::new(&graphics.context.device, &default()));

        self.world.insert_resource(graphics);

        let chunk_array = ChunkArray::new(&mut self.world, vecs!(2, 2, 2)).await?;
        self.world.insert_resource(chunk_array);

        Ok(())
    }

    /// Runs an [app][App]. Runs [`winit`]'s `event_loop`.
    pub fn run(mut self) -> ! {
        let event_loop = self.event_loop.take();
        event_loop.run(move |event, elw_target, control_flow| {
            let result = RUNTIME.block_on(
                self.run_frame_loop(event, elw_target, control_flow)
            );

            if let Err(error) = result {
                logger::error!(from = "app", "failed to run event loop: {error:#?}");
                panic!("panicked on {error}");
            }
        })
    }

    /// Exits [app][App]. Runs any destructor or deinitializer functions.
    pub async fn exit(&mut self, control_flow: &mut ControlFlow) {
        control_flow.set_exit();
    }

    /// Event loop run function.
    pub async fn run_frame_loop(
        &mut self, event: Event<'_, ()>,
        _elw_target: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow,
    ) -> AnyResult<()> {
        *control_flow = ControlFlow::Poll;

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
                self.exit(control_flow).await;
            },

            Event::MainEventsCleared => {
                if keyboard::just_pressed(cfg::key_bindings::APP_EXIT) {
                    self.exit(control_flow).await;
                    return Ok(());
                }

                if keyboard::just_pressed(cfg::key_bindings::SPAWN_CAMERA) {
                    self.world.spawn(camera::make_new());
                }

                let graphics = self.world.resource::<&Graphics>()?;
                graphics.window.request_redraw();
            },

            Event::RedrawRequested(window_id) => {
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

    /// Updates `ecs`'s systems. Note that non of resources can be borrowed at this point.
    async fn update_systems(&mut self) -> AnyResult<()> {
        CameraHandle::update_all(&self.world);
        Graphics::update(&self.world)?;

        Ok(())
    }

    /// Updates all in the [app][App].
    async fn update(&mut self, _window_id: WindowId) -> AnyResult<()> {
        for update in update::UPDATE_FUNCTIONS.lock().iter() {
            update();
        }

        if keyboard::just_pressed(cfg::key_bindings::RELOAD_RESOURCES) {
            let arr_entity = self.world.resource::<&ChunkArray>()?.array_entity;
            ChunkArray::make_binds(&mut self.world, arr_entity).await;
        }

        if keyboard::just_pressed(Key::N) {
            let arr = ChunkArray::new(&mut self.world, vecs!(2, 2, 2)).await?;
            self.world.insert_resource(arr);
        }

        {
            use crate::{transform::Transform, geometry::frustum::Frustum};

            let mut arr = self.world.resource::<&mut ChunkArray>()?;
            let MainCamera(cam) = self.world.copy_resource::<MainCamera>()?;

            let mut query = self.world
                .query_one::<(&CameraComponent, &Transform, &Frustum)>(cam.entity).unwrap();
            let (_cam, transform, frustum) = query.get().unwrap();

            arr.update_meshes(&self.world, transform.translation.position, frustum).await?;
            arr.update(&self.world).await?;
        }

        {
            self.world.resource::<&mut Timer>()?.update();
            let graphics = self.world.resource::<&Graphics>()?;
            
            mouse::update(&graphics.window)
                .log_error("app", "failed to update mouse input");

            if keyboard::just_pressed(cfg::key_bindings::MOUSE_CAPTURE) {
                let camera = self.world.resource::<&MainCamera>()?;
                mouse::set_capture(
                    &graphics.window,
                    camera.switch_mouse_capture(&self.world),
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

        let chunk_array = self.world.resource::<&ChunkArray>()?;

        graphics.render_with_sandbox(&self.world, |binds, encoder, view, depth, world| {
            chunk_array.render(world, binds, encoder, view.clone(), depth.cloned())
        })?;

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
}
