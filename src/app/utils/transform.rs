use crate::prelude::*;



#[derive(Debug, Clone, PartialEq, Default, Display, TypeUuid)]
#[uuid = "92a577ad-4152-4b10-b03b-78b142d74e8c"]
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



#[derive(Debug, Clone, Copy, PartialEq, Default, Deref, Display, TypeUuid)]
#[uuid = "ee6d03c0-9ac1-435c-a25b-8d4fe2bf3a00"]
#[display("x: {offset.x:.3}, y: {offset.y:.3}, z: {offset.z:.3}")]
pub struct Translation {
    pub offset: vec3,
}
assert_impl_all!(Translation: Send, Sync);

impl From<vec3> for Translation {
    fn from(value: vec3) -> Self {
        Self { offset: value }
    }
}

impl AsMatrix for Translation {
    fn as_matrix(&self) -> mat4 {
        mat4::translation(self.offset)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Default, Deref, Display, TypeUuid)]
#[uuid = "3cf29db0-3c57-4b13-a7ab-92e228117806"]
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

impl From<vec3> for Rotation {
    fn from(value: vec3) -> Self {
        Self { angles: value }
    }
}

impl AsMatrix for Rotation {
    fn as_matrix(&self) -> mat4 {
        let (roll, pitch, yaw) = self.angles.as_tuple();
        mat4::rotation_rpy(roll, pitch, yaw)
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Deref, SmartDefault, Display, TypeUuid)]
#[uuid = "3cf29db0-3c57-4b13-a7ab-92e228117806"]
#[display("x: {amount.x:.3}, y: {amount.y:.3}, z: {amount.z:.3}")]
pub struct Scaling {
    #[default(vec3::ONE)]
    pub amount: vec3,
}
assert_impl_all!(Scaling: Send, Sync);

impl From<vec3> for Scaling {
    fn from(value: vec3) -> Self {
        Self { amount: value }
    }
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



#[derive(Clone, Copy, Deref, Default, PartialEq, Debug, TypeUuid)]
#[uuid = "02e06374-1036-4fa8-8372-76f9921e4305"]
pub struct Speed {
    pub inner: vec3,
}
assert_impl_all!(Speed: Send, Sync);

impl From<vec3> for Speed {
    fn from(value: vec3) -> Self {
        Self { inner: value }
    }
}

impl GetOffset for Speed {
    fn get_offset(&self, dt: f32) -> vec3 {
        dt * self.inner
    }
}



#[derive(Clone, Copy, Deref, Default, PartialEq, Debug, TypeUuid)]
#[uuid = "9714431b-f71f-4dc9-9484-07e66dba464e"]
pub struct Acceleration {
    pub inner: vec3,
}
assert_impl_all!(Acceleration: Send, Sync);

impl From<vec3> for Acceleration {
    fn from(value: vec3) -> Self {
        Self { inner: value }
    }
}

impl GetOffset for Acceleration {
    fn get_offset(&self, dt: f32) -> vec3 {
        // FIXME: wrong delta
        0.5 * dt * dt * self.inner
    }
}



pub trait GetOffset {
    fn get_offset(&self, dt: f32) -> vec3;

    fn affect_translation(&self, dt: f32, translation: &mut Translation) {
        translation.offset += self.get_offset(dt);
    }

    fn affect_multiple<Offsets, Offset>(offsets: Offsets, dt: f32, translation: &mut Translation)
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