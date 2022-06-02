
/// Can handle both angle types: `radians` and `degrees`
#[derive(Clone, Copy, Default)]
pub struct Angle {
	angle_in_rad: f32,
	angle_in_degree: f32
}

#[allow(dead_code)]
impl Angle {
	/// Constructs angle from radians measure.
	pub fn from_radians(radians: f32) -> Self {
		Angle {
			angle_in_degree: radian_to_degree(radians),
			angle_in_rad: radians
		}
	}

	/// Constructs angle from degrees measure.
	pub fn from_degrees(degrees: f32) -> Self {
		Angle {
			angle_in_degree: degrees,
			angle_in_rad: degree_to_radian(degrees)
		}
	}

	/// Set angle from radians measure.
	pub fn set_radians(&mut self, radians: f32) {
		self.angle_in_degree = radian_to_degree(radians);
		self.angle_in_rad = radians;
	}

	/// Set angle from degree measure.
	pub fn set_degrees(&mut self, degrees: f32) {
		self.angle_in_degree = degrees;
		self.angle_in_rad = degree_to_radian(degrees);
	}

	/// Gives angle in radians.
	pub fn get_radians(&self) -> f32 {
		self.angle_in_rad
	}

	/// Gives angle in degrees.
	pub fn get_degrees(&self) -> f32 {
		self.angle_in_degree
	}

	/// Gives mutable radians reference. Should be flushed with `update_from_radians(&mut self)` explicitly.
	pub fn get_radians_mut(&mut self) -> &mut f32 {
		&mut self.angle_in_rad
	}
	
	/// Gives mutable degrees reference. Should be flushed with `update_from_degrees(&mut self)` explicitly.
	pub fn get_degrees_mut(&mut self) -> &mut f32 {
		&mut self.angle_in_degree
	}

	/// Updates second angle measure based on first.
	pub fn update_from_radians(&mut self) {
		self.angle_in_degree = radian_to_degree(self.angle_in_rad);
	}

	/// Updates second angle measure based on first.
	pub fn update_from_degrees(&mut self) {
		self.angle_in_rad = degree_to_radian(self.angle_in_degree);
	}
}

/// Converts degrees to radians.
#[allow(dead_code)]
pub fn degree_to_radian(degrees: f32) -> f32 {
	degrees * std::f32::consts::PI / 180.0
}

/// Converts radians to degrees.
#[allow(dead_code)]
pub fn radian_to_degree(radians: f32) -> f32 {
	radians * 180.0 / std::f32::consts::PI
}