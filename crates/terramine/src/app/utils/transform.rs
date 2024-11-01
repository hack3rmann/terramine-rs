use crate::prelude::*;



#[derive(Debug, Clone, PartialEq, ConstDefault, Display)]
#[display("translation: {translation}; rotation: {rotation}; scaling: {scaling}")]
pub struct Transform {
    pub translation: Translation,
    pub rotation: Rotation,
    pub scaling: Scaling,
}
assert_impl_all!(Transform: Send, Sync);

impl Transform {
    /// Returns view matrix.
    pub fn get_view(&self) -> Mat4 {
        let rotation = self.rotation.as_matrix();
        let front = rotation * cfg::terrain::FRONT_NORMAL;
        let up = rotation * cfg::terrain::TOP_NORMAL;
        let pos = *self.translation;

        Mat4::look_at_lh(pos, pos + front, up)
    }

    pub fn as_matrix(&self) -> Mat4 {
        Mat4::from_mat3a(self.translation.as_matrix().matrix3)
            * Mat4::from_mat3(self.rotation.as_matrix())
            * Mat4::from_mat3(self.scaling.as_matrix())
    }
}



#[derive(Debug, Clone, Copy, PartialEq, ConstDefault, Deref, Display, From, Into)]
#[display("x: {position.x:.3}, y: {position.y:.3}, z: {position.z:.3}")]
pub struct Translation {
    pub position: Vec3,
}
assert_impl_all!(Translation: Send, Sync);

impl Translation {
    pub fn as_matrix(self) -> Affine3A {
        Affine3A::from_translation(self.position)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, ConstDefault, Deref, Display, From)]
#[display("roll: {angles.x:.3}, pitch: {angles.y:.3}, yaw: {angles.z:.3}")]
pub struct Rotation {
    /// Rotation in `vec3 { x: roll, y: pitch, z: yaw }`.
    pub angles: Vec3,
}
assert_impl_all!(Rotation: Send, Sync);

impl Rotation {
    pub fn rotate(&mut self, delta: Vec3) {
        self.angles += delta
    }

    pub fn right(&self) -> Vec3 {
        self.as_matrix() * cfg::terrain::RIGHT_NORMAL
    }

    pub fn up(&self) -> Vec3 {
        self.as_matrix() * cfg::terrain::TOP_NORMAL
    }

    pub fn front(&self) -> Vec3 {
        self.as_matrix() * cfg::terrain::FRONT_NORMAL
    }

    pub fn as_matrix(&self) -> Mat3 {
        let [roll, pitch, yaw] = self.angles.to_array();
        Mat3::from_euler(EulerRot::XYZ, pitch, yaw, roll)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Deref, SmartDefault, Display, From, Into)]
#[display("x: {amount.x:.3}, y: {amount.y:.3}, z: {amount.z:.3}")]
#[default(Self::DEFAULT)]
pub struct Scaling {
    pub amount: Vec3,
}
assert_impl_all!(Scaling: Send, Sync);

impl Scaling {
    pub fn as_matrix(&self) -> Mat3 {
        Mat3::from_diagonal(self.amount)
    }
}

impl ConstDefault for Scaling {
    const DEFAULT: Self = Self { amount: Vec3::ONE };
}



#[derive(Clone, Copy, Deref, ConstDefault, PartialEq, Debug, From)]
pub struct Speed {
    pub inner: Vec3,
}
assert_impl_all!(Speed: Send, Sync);

impl Speed {
    pub fn get_offset(&self, dt: TimeStep) -> Vec3 {
        dt.as_secs_f32() * self.inner
    }

    pub fn affect_translation(&self, dt: TimeStep, translation: &mut Translation) {
        translation.position += self.get_offset(dt)
    }
}



#[derive(Clone, Copy, Deref, ConstDefault, PartialEq, Debug, From)]
pub struct Acceleration {
    pub inner: Vec3,
}
assert_impl_all!(Acceleration: Send, Sync);

impl Acceleration {
    pub fn get_offset(&self, dt: TimeStep) -> Vec3 {
        // FIXME: wrong delta
        let dt = dt.as_secs_f32();
        0.5 * dt * dt * self.inner
    }
}