use crate::{prelude::*, transform::*, graphics::{Buffer, Binds, Device, Queue}};



#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref)]
pub struct MainCamera(pub CameraHandle);
assert_impl_all!(MainCamera: Component, Send, Sync);



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Into)]
pub struct CameraHandle {
    pub entity: Entity,
}
assert_impl_all!(CameraHandle: Send, Sync);

impl CameraHandle {
    pub fn spawn_default(world: &mut World) -> Self {
        Self::new(world.spawn(CameraBundle::default()))
    }

    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn switch_mouse_capture(&self, world: &World) -> bool {
        let mut query = world.query_one::<&mut CameraComponent>(self.entity).unwrap();
        let mut camera = query.get().unwrap();

        camera.captures_mouse = !camera.captures_mouse;
        camera.captures_mouse
    }

    pub fn update_all(world: &World) {
        let dt = world.resource::<&Timer>().unwrap().dt();
        let mut query = world.query::<(&mut CameraComponent, &mut Transform, &mut Speed)>();

        for (_entity, (camera, transform, speed)) in query.into_iter() {
            camera.update(dt, transform, speed);
        }
    }

    pub fn get_uniform(&self, world: &World) -> AnyResult<CameraUniform> {
        let mut query = world.query_one::<(&CameraComponent, &Transform)>(self.entity)?;
        let (cam, transform) = query.get()
            .context("camera entity does not have `CameraComponent` or `Transform`")?;
        Ok(CameraUniform::new(cam, transform))
    }

    pub fn spawn_control_windows(world: &World, ui: &imgui::Ui) {
        use crate::graphics::ui::imgui_ext::make_window;

        let mut query = world.query::<(&mut CameraComponent, Option<&mut Transform>)>();
        for (entity, (camera, transform)) in query.into_iter() {
            make_window(ui, format!("Camera #{}", entity.id())).build(|| {
                let transform = match transform {
                    Some(t) => &*t,
                    None => &Transform::DEFAULT,
                };

                ui.text("Position");
                ui.text(transform.translation.to_string());

                ui.text("Rotation");
                ui.text(transform.rotation.to_string());

                ui.separator();

                ui.slider_config("Speed", 5.0, 300.0)
                    .display_format("%.1f")
                    .build(&mut camera.speed_factor);

                ui.slider_config("Speed falloff", 0.0, 1.0)
                    .display_format("%.3f")
                    .build(&mut camera.speed_falloff);

                {
                    let mut fov = camera.fov.get_degrees();

                    ui.slider_config("FOV", 1.0, 180.0)
                        .display_format("%.0f")
                        .build(&mut fov);

                    camera.fov.set_degrees(fov);
                }
            });
        }
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct CameraComponent {
    pub fov: Angle,
    pub aspect_ratio: f32,

    pub near_plane: f32,
    pub far_plane: f32,

    pub speed_falloff: f32,
    pub speed_factor: f32,

    pub mouse_sensetivity: f32,
    pub captures_mouse: bool,
}
assert_impl_all!(CameraHandle: Send, Sync);

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

    pub fn update(&mut self, dt: TimeStep, transform: &mut Transform, speed: &mut Speed) {
        let dt_secs = dt.as_secs_f32();

        let (front, right) = {
            let rotation = transform.rotation.as_matrix();
            let right = vec3::from(cfg::terrain::RIGHT_NORMAL);
            let front = vec3::from(cfg::terrain::FRONT_NORMAL);
            (rotation * front, rotation * right)
        };

        let mut new_speed = vec3::ZERO;

        if keyboard::is_pressed(Key::W)      { new_speed += vecf!(front.x, 0, front.z).normalized() }
        if keyboard::is_pressed(Key::S)      { new_speed -= vecf!(front.x, 0, front.z).normalized() }
        if keyboard::is_pressed(Key::D)      { new_speed += right.normalized() }
        if keyboard::is_pressed(Key::A)      { new_speed -= right.normalized() }
        if keyboard::is_pressed(Key::Space)  { new_speed += vecf!(0, 1, 0) }
        if keyboard::is_pressed(Key::LShift) { new_speed -= vecf!(0, 1, 0) }

        new_speed = new_speed.with_len(self.speed_factor);

        *speed = if new_speed != vec3::ZERO {
            **speed / 2.0 + new_speed / 2.0
        } else if speed.len() > 0.1 {
            **speed * self.speed_falloff.powf(dt_secs + 1.0)
        } else {
            vec3::ZERO
        }.into();

        speed.affect_translation(dt, &mut transform.translation);

        if self.captures_mouse {
            let mouse_delta = mouse::get_delta();
            let angles = vec3::new(mouse_delta.y, 0.0, mouse_delta.x);

            // TODO: bound rotation by (-pi..pi)
            transform.rotation.rotate(dt_secs * self.mouse_sensetivity * angles);
        }

        if keyboard::just_pressed(Key::P) {
            *transform = default();
        }
    }

    /// Spawns camera control window.
    pub fn spawn_control_window(&mut self, ui: &imgui::Ui, transform: &mut Transform, speed: &mut Speed) {
        use crate::graphics::ui::imgui_ext::make_window;

        make_window(ui, "Camera").build(|| {
            ui.text("Position");
            ui.text(transform.translation.to_string());

            ui.text("Rotation");
            ui.text(transform.rotation.to_string());

            ui.separator();

            {
                let mut speed_module = speed.len();

                ui.slider_config("Speed", 5.0, 300.0)
                    .display_format("%.1f")
                    .build(&mut speed_module);

                *speed = speed.with_len(speed_module).into();
            }

            ui.slider_config("Speed falloff", 0.0, 1.0)
                .display_format("%.3f")
                .build(&mut self.speed_falloff);

            {
                let mut fov = self.fov.get_degrees();

                ui.slider_config("FOV", 1.0, 180.0)
                    .display_format("%.0f")
                    .build(&mut fov);

                self.fov.set_degrees(fov);
            }
        });
    }

    pub fn on_window_resize(&mut self, size: UInt2) {
        self.aspect_ratio = cfg::window::aspect_ratio(size.x as f32, size.y as f32);
    }
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov: Angle::from_degrees(cfg::camera::default::FOV_IN_DEGREES),
            aspect_ratio: cfg::window::default::ASPECT_RATIO,
            near_plane: cfg::camera::default::NEAR_PLANE,
            far_plane: cfg::camera::default::FAR_PLANE,
            speed_falloff: cfg::camera::default::SPEED_FALLOFF,
            speed_factor: cfg::camera::default::SPEED,
            mouse_sensetivity: cfg::camera::default::MOUSE_SENSETIVITY,
            captures_mouse: false,
        }
    }
}



pub type CameraBundle = (CameraComponent, Transform, Speed);



#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub proj: mat4,
    pub view: mat4,
}
assert_impl_all!(CameraUniform: Send, Sync);

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