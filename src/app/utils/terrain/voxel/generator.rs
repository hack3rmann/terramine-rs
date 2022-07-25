use crate::app::utils::math::prelude::*;

pub fn sine(pos: Int3) -> bool {
	pos.y() <
		((pos.x() as f32 / 80.0).sin() * 30.0 +
		 (pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32
}

pub fn trees(pos: Int3) -> bool {
	pos.y() <
		((pos.x() as f32).sin() * 3.0 +
		 (pos.z() as f32).sin() * 3.0 +
		 (pos.x() as f32 / 80.0).sin() * 30.0 +
		 (pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32 &&
	pos.y() >=
		((pos.x() as f32 / 80.0).sin() * 30.0 +
		 (pos.z() as f32 / 80.0).sin() * 30.0 + 8.0) as i32
}