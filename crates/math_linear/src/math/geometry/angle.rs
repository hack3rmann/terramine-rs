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

    pub fn set_radians(&mut self, radians: f32) {
        self.0 = radians;
    }

    pub fn set_degrees(&mut self, degrees: f32) {
        self.set_radians(degrees.to_radians())
    }

    pub const fn get_radians(&self) -> f32 {
        self.0
    }

    pub const fn get_degrees(&self) -> f32 {
        Self::to_degrees(self.0)
    }

    pub const fn to_radians(val: f32) -> f32 {
        val * (std::f32::consts::PI / 180.0)
    }

    pub const fn to_degrees(val: f32) -> f32 {
        const PIS_IN_180: f32 = 57.2957795130823208767981548141051703_f32;
        val * PIS_IN_180
    }
}
