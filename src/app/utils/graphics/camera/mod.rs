/**
 * Camera handler.
 */

pub mod frustum;

use {
    crate::app::utils::{
        cfg::{
            self,
            camera::default as cam_def,
            window::default as window_def,
        },
        werror::prelude::*,
        user_io::{InputManager, KeyCode},
    },
    math_linear::prelude::*,
    frustum::Frustum,
};

/// Camera handler.
pub struct Camera {
    /* Screen needs */
    pub fov: Angle,
    pub aspect_ratio: f32,
    pub near_plane_dist: f32,
    pub far_plane_dist: f32,

    /* Additional control */
    pub speed_factor: f64,
    pub grabbes_cursor: bool,

    /* Position */
    pub pos: vec3,
    pub speed: vec3,
    pub speed_falloff: f32,

    /* Rotation */
    pub rotation: mat4,
    pub roll:	f64,
    pub pitch:	f64,
    pub yaw:	f64,
    pub up:		vec3,
    pub front:	vec3,
    pub right:	vec3,

    /* Frustum */
    frustum: Option<Frustum>,
}

#[allow(dead_code)]
impl Camera {
    /// Creates camera.
    pub fn new() -> Self { Default::default() }

    /// Gives camera positioned to given coordinates
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.set_position(x, y, z);
        return self;
    }

    /// Sets position.
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.pos = vecf!(x, y, z);
    }

    /// Gives camera rotated to given angles
    pub fn with_rotation(mut self, roll: f64, pitch: f64, yaw: f64) -> Self {
        self.set_rotation(roll, pitch, yaw);
        return self;
    }

    /// Stores rotation.
    pub fn set_rotation(&mut self, roll: f64, pitch: f64, yaw: f64) {
        self.roll = roll;
        self.pitch = pitch;
        self.yaw = yaw;

        self.rotation = mat4::rotation_rpy(roll as f32, pitch as f32, yaw as f32);

        self.update_vectors();
    }

    /// Sets rotation to [0.0, 0.0, 0.0].
    pub fn reset_rotation(&mut self) {
        self.set_rotation(0.0, 0.0, 0.0);
    }

    /// Moves camera towards its vectors.
    pub fn move_relative(&mut self, front: f64, up: f64, right: f64) {
        /* Front */
        self.pos += vecf!(self.front.x, 0, self.front.z).normalized() * front as f32;

        /* Up */
        self.pos += vecf!(0, up, 0);

        /* Right */
        self.pos += self.right * right as f32;
    }

    /// Moves camera towards coordinates.
    pub fn move_absolute(&mut self, ds: vec3) {
        self.pos += ds
    }

    /// Rotates camera.
    pub fn rotate(&mut self, roll: f64, pitch: f64, yaw: f64) {
        self.roll += roll;
        self.pitch += pitch;
        self.yaw += yaw;

        /* Vertical camera look boundaries */
        use std::f64::consts::FRAC_PI_2;
        const EPS: f64 = cfg::camera::VERTICAL_LOOK_EPS;
        if self.pitch > FRAC_PI_2 {
            self.pitch = FRAC_PI_2 - EPS;
        } else if self.pitch < -FRAC_PI_2 {
            self.pitch = -FRAC_PI_2 + EPS;
        }

        self.set_rotation(self.roll, self.pitch, self.yaw);
    }

    /// This function updates camera vectors from rotatiion matrix.
    pub fn update_vectors(&mut self) {
        /* Transform basic vectors with rotation matrix */
        self.up    = &self.rotation * vecf!(0,  1,  0);
        self.front = &self.rotation * vecf!(0,  0, -1);
        self.right = &self.rotation * vecf!(1,  0,  0);

        /* Frustum update */
        self.frustum = Some(Frustum::new(self));
    }

    /// Updates camera (key press checking, etc).
    pub fn update(&mut self, input: &mut InputManager, dt: f64) {
        /* Camera move vector */
        let mut new_speed = vec3::all(0.0);

        /* Movement controls */
        if input.keyboard.is_pressed(KeyCode::W)		{ new_speed += vecf!(self.front.x, 0, self.front.z).normalized() }
        if input.keyboard.is_pressed(KeyCode::S)		{ new_speed -= vecf!(self.front.x, 0, self.front.z).normalized() }
        if input.keyboard.is_pressed(KeyCode::A)		{ new_speed += self.right.normalized() }
        if input.keyboard.is_pressed(KeyCode::D)		{ new_speed -= self.right.normalized() }
        if input.keyboard.is_pressed(KeyCode::Space)	{ new_speed += vecf!(0, 1, 0) }
        if input.keyboard.is_pressed(KeyCode::LShift)	{ new_speed -= vecf!(0, 1, 0) }

        /* Calculate new speed */
        new_speed = new_speed.normalized() * self.speed_factor as f32;

        /* Normalyzing direction vector */
        self.speed = if new_speed != vec3::zero() {
            self.speed / 2.0 + new_speed / 2.0
        } else {
            if self.speed.len() > 0.1 {
                const SPEED_FALLOFF_ADDITION: f32 = 1.0;
                self.speed * self.speed_falloff.powf(dt as f32 + SPEED_FALLOFF_ADDITION)
            } else {
                vec3::all(0.0)
            }
        };

        /* Move camera with move vector */
        self.move_absolute(self.speed * dt as f32);

        /* Reset */
        if input.keyboard.just_pressed(KeyCode::P) {
            self.set_position(0.0, 0.0, 2.0);
            self.reset_rotation();
        }

        /* Cursor borrow */
        if self.grabbes_cursor {
            self.rotate(
                 0.0,
                -input.mouse.dy * dt * 0.2,
                 input.mouse.dx * dt * 0.2,
            );
        }
    }

    /// Returns view matrix.
    pub fn get_view(&self) -> [[f32; 4]; 4] {
        mat4::look_at_lh(self.pos, self.pos + self.front, self.up).as_2d_array()
    }

    /// Returns projection matrix with `aspect_ratio = height / width`
    pub fn get_proj(&self) -> [[f32; 4]; 4] {
        mat4::perspective_fov_lh(self.fov.get_radians(), self.aspect_ratio, self.near_plane_dist, self.far_plane_dist).as_2d_array()
    }

    /// Checks if position is in camera frustum
    pub fn is_pos_in_view(&self, pos: vec3) -> bool {
        self.get_frustum().is_in_frustum(pos)
    }

    /// Checks if AABB is in camera frustum
    pub fn is_aabb_in_view(&self, aabb: AABB) -> bool {
        self.get_frustum().is_aabb_in_frustum(aabb)
    }

    /// Gives frustum from camera
    pub fn get_frustum(&self) -> Frustum {
        if self.frustum.is_none() {
            Frustum::new(self)
        } else {
            self.frustum.as_ref().wunwrap().clone()
        }
    }

    /// Returns X component of pos vector.
    pub fn get_x(&self) -> f32 { self.pos.x }

    /// Returns Y component of pos vector.
    pub fn get_y(&self) -> f32 { self.pos.y }

    /// Returns Z component of pos vector.
    pub fn get_z(&self) -> f32 { self.pos.z }

    /// Spawns camera control window.
    pub fn spawn_control_window(&mut self, ui: &imgui::Ui, input: &mut InputManager) {
        /* Camera control window */
        let mut camera_window = imgui::Window::new("Camera");

        /* Move and resize if pressed I key */
        if !input.keyboard.is_pressed(KeyCode::I) {
            camera_window = camera_window
                .resizable(false)
                .movable(false)
                .collapsible(false)
        }

        /* UI building */
        camera_window.build(&ui, || {
            ui.text("Position");
            ui.text(format!("x: {x:.3}, y: {y:.3}, z: {z:.3}", x = self.get_x(), y = self.get_y(), z = self.get_z()));
            ui.text("Rotation");
            ui.text(format!("roll: {roll:.3}, pitch: {pitch:.3}, yaw: {yaw:.3}", roll = self.roll, pitch = self.pitch, yaw = self.yaw));
            ui.separator();
            imgui::Slider::new("Speed", 5.0, 300.0)
                .display_format("%.1f")
                .build(&ui, &mut self.speed_factor);
            imgui::Slider::new("Speed falloff", 0.0, 1.0)
                .display_format("%.3f")
                .build(&ui, &mut self.speed_falloff);
            imgui::Slider::new("FOV", 1.0, 180.0)
                .display_format("%.0f")
                .build(&ui, self.fov.get_degrees_mut());
            self.fov.update_from_degrees();
        });
    }
}

impl Default for Camera {
    fn default() -> Self {
        let mut cam = Camera {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            fov: Angle::from_degrees(cam_def::FOV_IN_DEGREES),
            near_plane_dist: cam_def::NEAR_PLANE,
            far_plane_dist: cam_def::FAR_PLANE,
            grabbes_cursor: false,
            speed_factor: cam_def::SPEED,
            speed_falloff: cam_def::SPEED_FALLOFF,
            aspect_ratio: window_def::HEIGHT / window_def::WIDTH,
            pos:    vecf!(0, 0, -3),
            speed:  vec3::zero(),
            up:     vecf!(0, 1, 0),
            front:  vecf!(0, 0, -1),
            right:  vecf!(1, 0, 0),
            rotation: Default::default(),
            frustum: None,
        };
        cam.update_vectors();

        return cam;
    }
}