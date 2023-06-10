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
    pub fn get_view(&self) -> mat4 {
        let rotation = self.rotation.as_matrix();
        let front = rotation * vec3::from(cfg::terrain::FRONT_NORMAL);
        let up = rotation * vec3::from(cfg::terrain::TOP_NORMAL);
        let pos = *self.translation;

        mat4::look_at_lh(pos, pos + front, up)
    }
}

impl AsMatrix for Transform {
    fn as_matrix(&self) -> mat4 {
        self.translation.as_matrix()
            * self.rotation.as_matrix()
            * self.scaling.as_matrix()
    }
}



#[derive(Debug, Clone, Copy, PartialEq, ConstDefault, Deref, Display, From, Into)]
#[display("x: {offset.x:.3}, y: {offset.y:.3}, z: {offset.z:.3}")]
pub struct Translation {
    pub offset: vec3,
}
assert_impl_all!(Translation: Send, Sync);

impl AsMatrix for Translation {
    fn as_matrix(&self) -> mat4 {
        mat4::translation(self.offset)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, ConstDefault, Deref, Display, From)]
#[display("roll: {angles.x:.3}, pitch: {angles.y:.3}, yaw: {angles.z:.3}")]
pub struct Rotation {
    /// Rotation in `vec3 { x: roll, y: pitch, z: yaw }`.
    pub angles: vec3,
}
assert_impl_all!(Rotation: Send, Sync);

impl Rotation {
    pub fn rotate(&mut self, delta: vec3) {
        self.angles += delta
    }
}

impl AsMatrix for Rotation {
    fn as_matrix(&self) -> mat4 {
        let (roll, pitch, yaw) = self.angles.as_tuple();
        mat4::rotation_rpy(roll, pitch, yaw)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Deref, SmartDefault, Display, From, Into)]
#[display("x: {amount.x:.3}, y: {amount.y:.3}, z: {amount.z:.3}")]
#[default(Self::DEFAULT)]
pub struct Scaling {
    pub amount: vec3,
}
assert_impl_all!(Scaling: Send, Sync);

impl ConstDefault for Scaling {
    const DEFAULT: Self = Self { amount: vec3::ONE };
}

impl AsMatrix for Scaling {
    fn as_matrix(&self) -> mat4 {
        mat4::scaling(self.amount)
    }
}



pub trait AsMatrix {
    fn as_matrix(&self) -> mat4;
}
assert_obj_safe!(AsMatrix);



#[derive(Clone, Copy, Deref, Default, PartialEq, Debug, From)]
pub struct Speed {
    pub inner: vec3,
}
assert_impl_all!(Speed: Send, Sync);

impl GetOffset for Speed {
    fn get_offset(&self, dt: TimeStep) -> vec3 {
        dt.as_secs_f32() * self.inner
    }
}



#[derive(Clone, Copy, Deref, Default, PartialEq, Debug, From)]
pub struct Acceleration {
    pub inner: vec3,
}
assert_impl_all!(Acceleration: Send, Sync);

impl GetOffset for Acceleration {
    fn get_offset(&self, dt: TimeStep) -> vec3 {
        // FIXME: wrong delta
        let dt = dt.as_secs_f32();
        0.5 * dt * dt * self.inner
    }
}



pub trait GetOffset {
    fn get_offset(&self, dt: TimeStep) -> vec3;

    fn affect_translation(&self, dt: TimeStep, translation: &mut Translation) {
        translation.offset += self.get_offset(dt);
    }

    fn affect_multiple<Offsets, Offset>(offsets: Offsets, dt: TimeStep, translation: &mut Translation)
    where
        Self: Sized,
        Offset: GetOffset,
        Offsets: IntoIterator<Item = Offset>,
    {
        for offset in offsets.into_iter() {
            offset.affect_translation(dt, translation);
        }
    }
}
assert_obj_safe!(GetOffset);