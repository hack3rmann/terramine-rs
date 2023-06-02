use crate::{
    prelude::*,
    transform::*,
};



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TypeUuid)]
#[uuid = "40b56f60-c643-4193-abaa-9b370c8b6672"]
pub struct CameraHandle {
    pub entity: Entity,
}
assert_impl_all!(CameraHandle: Send, Sync);

impl CameraHandle {
    pub fn spawn_default(world: &mut World) -> Self {
        Self { entity: world.spawn(CameraBundle::default()) }
    }

    pub fn from_entity(world: &World, entity: Entity) -> Self {
        let mut query = world.query_one::<(&CameraComponent, &Transform, &Speed)>(entity)
            .expect("camera entity should exist");
        query.get().expect("camera entity should have CameraComponent, Transfrom and Speed components");

        Self::from_entity_unchecked(entity)
    }

    pub fn from_entity_unchecked(entity: Entity) -> Self {
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

    pub fn spawn_control_windows(world: &World, ui: &imgui::Ui) {
        use crate::graphics::ui::imgui_ext::make_window;

        let mut query = world.query::<(&mut CameraComponent, &mut Transform)>();
        for (entity, (camera, transform)) in query.into_iter() {
            make_window(ui, format!("Camera #{}", entity.id())).build(|| {
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



#[derive(Debug, Clone, TypeUuid, PartialEq)]
#[uuid = "bb9e7b74-3847-48b9-b515-2110c4b5b0df"]
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
        if keyboard::is_pressed(Key::A)      { new_speed += right.normalized() }
        if keyboard::is_pressed(Key::D)      { new_speed -= right.normalized() }
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
            let mouse_delta = vec3::new(0.0, -mouse::get_dy_dt(), mouse::get_dx_dt());
            transform.rotation.rotate(dt_secs * self.mouse_sensetivity * mouse_delta);
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
