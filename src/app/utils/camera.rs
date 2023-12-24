use crate::{
    prelude::*, transform::*, graphics::{Buffer, Binds, Device, Queue, ui::egui_util},
    geometry::frustum::Frustum,
};



#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From, Into)]
pub struct MainCamera(pub Entity);
assert_impl_all!(MainCamera: Send, Sync, Component);

impl MainCamera {
    pub fn set(world: &World, camera: Entity) -> AnyResult<()> {
        let mut main = world.resource::<&mut Self>()
            .context("failed to find main camera")?;

        main.0 = camera;

        Ok(())
    }

    pub fn is_main(world: &World, camera: Entity) -> AnyResult<bool> {
        let main = world.resource::<&Self>()
            .context("failed to find main camera")?;

        Ok(main.0 == camera)
    }
}



#[derive(Debug, Clone, Copy, Hash)]
pub struct CameraHandle;

impl CameraHandle {
    pub fn switch_mouse_capture(world: &World, camera: Entity) -> bool {
        let mut query = world.query_one::<&mut CameraComponent>(camera).unwrap();
        let camera_component = query.get().unwrap();

        let CameraActivity::Enabled { captures_mouse }
            = &mut camera_component.activity else { return false };

        *captures_mouse = !*captures_mouse;
        *captures_mouse
    }

    pub fn update_all(world: &World) {
        let dt = world.resource::<&Timer>().unwrap().time_step();
        let mut query = world.query::<(&mut CameraComponent, &mut Transform, &mut Speed, &mut Frustum)>();

        for (_entity, (camera, transform, speed, frustum)) in query.into_iter() {
            for _ in 0..cfg::camera::N_SIMULATIONS_STEPS {
                let dt = dt / cfg::camera::N_SIMULATIONS_STEPS as u32;
                camera.update(dt, transform, speed, frustum);
            }
        }
    }

    pub fn get_uniform(world: &World, camera: Entity) -> AnyResult<CameraUniform> {
        let mut query = world.query_one::<(&CameraComponent, &Transform)>(camera)?;

        let (component, transform) = query.get()
            .context("camera entity does not have `CameraComponent` or `Transform`")?;

        Ok(CameraUniform::new(component, transform))
    }

    pub fn spawn_control_window(world: &World, ui: &mut egui::Ui) {
        let mut query = world.query::<(
            &mut CameraComponent,
            Option<&mut Transform>
        )>();

        ui.collapsing("Cameras", |ui| {
            ui.label(format!(
                "Press '{:?}' to spawn new camera",
                cfg::key_bindings::SPAWN_CAMERA
            ));

            for (entity, (camera, transform)) in query.iter() {
                ui.collapsing(format!("Camera #{}", entity.id()), |ui| {
                    let mut new_transform = match transform {
                        None => Transform::DEFAULT,
                        Some(ref transform) => (*transform).clone(),
                    };

                    ui.add(egui_util::TransformWidget::new(&mut new_transform));

                    if let Some(transform) = transform {
                        *transform = new_transform;
                    }

                    ui.separator();

                    ui.add(
                        egui::DragValue::new(&mut camera.speed_factor)
                            .clamp_range(0.0..=100_000.0)
                            .prefix("Speed: ")
                            .max_decimals(1)
                    );
    
                    ui.add(
                        egui::Slider::new(&mut camera.speed_falloff, 0.99..=1.0)
                            .text("Speed falloff")
                            .max_decimals(3)
                    );

                    {
                        let mut fov = camera.fov.get_degrees();

                        ui.add(
                            egui::Slider::new(&mut fov, 1.0..=180.0)
                                .text("FOV")
                                .integer()
                        );

                        camera.fov.set_degrees(fov);
                    }

                    if MainCamera::is_main(world, entity).unwrap() {
                        ui.label(egui::RichText::new("main").color(egui::Color32::GREEN));
                    } else {
                        ui.label(egui::RichText::new("not main").color(egui::Color32::RED));
                        if ui.button("Set main").clicked() {
                            camera.enable();
                            MainCamera::set(world, entity).unwrap();
                        }
                    }
                });
            }
        });
    }
}



#[derive(Debug, Clone, Copy, Eq, PartialEq, IsVariant)]
pub enum CameraActivity {
    Disabled,
    Enabled { captures_mouse: bool },
}

impl ConstDefault for CameraActivity {
    const DEFAULT: Self = Self::Disabled;
}

impl Default for CameraActivity {
    fn default() -> Self { const_default() }
}



assert_impl_all!(CameraHandle: Send, Sync);
#[derive(Debug, Clone, PartialEq)]
pub struct CameraComponent {
    pub fov: Angle,
    pub aspect_ratio: f32,

    pub near_plane: f32,
    pub far_plane: f32,

    pub speed_falloff: f32,
    pub speed_factor: f32,

    pub mouse_sensetivity: f32,
    pub activity: CameraActivity,
}

impl CameraComponent {
    /// Returns projection matrix.
    pub fn get_proj(&self) -> mat4 {
        mat4::perspective_fov_lh(
            self.fov.get_radians(),
            self.aspect_ratio,
            self.near_plane,
            self.far_plane,
        )
    }

    pub fn update(&mut self, dt: TimeStep, transform: &mut Transform, speed: &mut Speed, frustum: &mut Frustum) {
        let dt_secs = dt.as_secs_f32();

        let (front, right) = (transform.rotation.front(), transform.rotation.right());

        let mut new_speed = vec3::ZERO;

        if let CameraActivity::Enabled { captures_mouse: true } = self.activity {
            if keyboard::is_pressed(Key::W)      { new_speed += vecf!(front.x, 0, front.z).normalized() }
            if keyboard::is_pressed(Key::S)      { new_speed -= vecf!(front.x, 0, front.z).normalized() }
            if keyboard::is_pressed(Key::D)      { new_speed += right.normalized() }
            if keyboard::is_pressed(Key::A)      { new_speed -= right.normalized() }
            if keyboard::is_pressed(Key::Space)  { new_speed += vecf!(0, 1, 0) }
            if keyboard::is_pressed(Key::LShift) { new_speed -= vecf!(0, 1, 0) }
        }

        new_speed = new_speed.with_len(self.speed_factor);

        *speed = if new_speed != vec3::ZERO {
            **speed / 2.0 + new_speed / 2.0
        } else if speed.len() > 0.1 {
            **speed * self.speed_falloff.powf(dt_secs + 1.0)
        } else {
            vec3::ZERO
        }.into();

        speed.affect_translation(dt, &mut transform.translation);

        if let CameraActivity::Enabled { captures_mouse: true } = self.activity {
            let mouse_delta = mouse::get_delta();

            let accel_multiple = if cfg::camera::IS_CAMERA_LOOK_ACCELERATION_ENABLED {
                cfg::camera::MOUSE_SENSETIVITY * dt_secs
            } else { 1.0 };

            let angles = accel_multiple * vec3::new(mouse_delta.y, 0.0, mouse_delta.x);

            // TODO: bound rotation by (-pi..pi)
            transform.rotation.rotate(dt_secs * self.mouse_sensetivity * angles);
        }

        if keyboard::just_pressed(Key::P) {
            *transform = default();
        }

        *frustum = Frustum::new(self, transform);
    }

    pub fn on_window_resize(&mut self, size: UInt2) {
        self.aspect_ratio = cfg::window::aspect_ratio(size.x as f32, size.y as f32);
    }

    pub fn disable(&mut self) {
        self.activity = CameraActivity::Disabled;
    }

    pub fn enable(&mut self) {
        self.activity = CameraActivity::Enabled { captures_mouse: false };
    }
}

impl ConstDefault for CameraComponent {
    const DEFAULT: Self = Self {
        fov: Angle::from_degrees(cfg::camera::default::FOV_IN_DEGREES),
        aspect_ratio: cfg::window::default::ASPECT_RATIO,
        near_plane: cfg::camera::default::NEAR_PLANE,
        far_plane: cfg::camera::default::FAR_PLANE,
        speed_falloff: cfg::camera::default::SPEED_FALLOFF,
        speed_factor: cfg::camera::default::SPEED,
        mouse_sensetivity: cfg::camera::default::MOUSE_SENSETIVITY,
        activity: const_default(),
    };
}

impl Default for CameraComponent {
    fn default() -> Self { const_default() }
}

impl egui_util::ShowUi for CameraComponent {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::DragValue::new(&mut self.speed_factor)
                .clamp_range(0.0..=100_000.0)
                .prefix("Speed: ")
                .max_decimals(1)
        );

        ui.add(
            egui::Slider::new(&mut self.speed_falloff, 0.99..=1.0)
                .text("Speed falloff")
                .max_decimals(3)
        );

        {
            let mut fov = self.fov.get_degrees();

            ui.add(
                egui::Slider::new(&mut fov, 1.0..=180.0)
                    .text("FOV")
                    .integer()
            );

            self.fov.set_degrees(fov);
        }
    }
}



pub type CameraBundle = (CameraComponent, Transform, Speed, Frustum);

pub fn make_new_camera_bundle_enabled() -> CameraBundle {
    let mut result = make_new_camera_bundle();
    result.0.enable();
    result
}

pub fn make_new_camera_bundle() -> CameraBundle {
    let cam = CameraComponent::DEFAULT;
    let transform = Transform::DEFAULT;
    let speed = Speed::DEFAULT;
    let frustum = Frustum::new(&cam, &transform);

    (cam, transform, speed, frustum)
}



assert_impl_all!(CameraUniform: Send, Sync);
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub proj: mat4,
    pub view: mat4,
}

impl CameraUniform {
    pub fn new(cam: &CameraComponent, transform: &Transform) -> Self {
        Self { proj: cam.get_proj(), view: transform.get_view() }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new(&default(), &default())
    }
}



#[derive(Debug)]
pub struct CameraUniformBuffer {
    pub buffer: Buffer,
    pub binds: Binds,
}

impl CameraUniformBuffer {
    pub fn new(device: &Device, init: &CameraUniform) -> Self {
        use crate::graphics::*;

        let buffer = Buffer::new(
            device,
            &BufferInitDescriptor {
                label: Some("camera_uniform_buffer"),
                contents: bytemuck::bytes_of(init),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            },
        );

        let layout = BindGroupLayout::new(
            device,
            &BindGroupLayoutDescriptor {
                label: Some("camera_unifors_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            },
        );

        let bind_group = BindGroup::new(
            device,
            &BindGroupDescriptor {
                label: Some("common_uniforms_bind_group"),
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            },
        );

        let binds = Binds::from_iter([
            (bind_group, Some(layout)),
        ]);

        Self { binds, buffer }
    }

    pub fn update(&self, queue: &Queue, uniform: &CameraUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform));
        queue.submit(None);
    }
}