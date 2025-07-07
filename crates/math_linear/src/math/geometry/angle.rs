use std::f32::consts::PI;

/// Can handle both angle types: `radians` and `degrees`
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Angle(pub f32);

impl Angle {
    pub const fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    pub const fn from_degrees(deg: f32) -> Self {
        Self::from_radians(Self::to_radians(deg))
    }

    pub const fn set_radians(&mut self, radians: f32) {
        self.0 = radians;
    }

    pub const fn set_degrees(&mut self, degrees: f32) {
        self.set_radians(degrees.to_radians())
    }

    pub const fn get_radians(&self) -> f32 {
        self.0
    }

    pub const fn get_degrees(&self) -> f32 {
        Self::to_degrees(self.0)
    }

    pub const fn to_radians(val: f32) -> f32 {
        val * PI / 180.0
    }

    pub const fn to_degrees(val: f32) -> f32 {
        val * 57.295_78_f32
    }
}
