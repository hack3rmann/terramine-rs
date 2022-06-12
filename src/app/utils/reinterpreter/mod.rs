use std::mem::transmute;

pub trait Reinterpret: ReinterpretAsBytes + ReinterpretFromBytes { }

pub trait ReinterpretAsBytes {
	fn reinterpret_as_bytes(&self) -> Vec<u8>;
}

pub trait ReinterpretFromBytes {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self;
}



impl Reinterpret for u8 { }

impl ReinterpretAsBytes for u8 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> { vec![*self] }
}

impl ReinterpretFromBytes for u8 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		source[0]
	}
}



impl Reinterpret for i8 { }

impl ReinterpretAsBytes for i8 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> { unsafe { vec![transmute(*self)] } }
}

impl ReinterpretFromBytes for i8 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe { transmute(source[0]) }
	}
}



impl Reinterpret for u16 { }

impl ReinterpretAsBytes for u16 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 2] = transmute(*self);
			vec![bytes[0], bytes[1]]
		}
	}
}

impl ReinterpretFromBytes for u16 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1]])
		}
	}
}



impl Reinterpret for i16 { }

impl ReinterpretAsBytes for i16 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 2] = transmute(*self);
			vec![bytes[0], bytes[1]]
		}
	}
}

impl ReinterpretFromBytes for i16 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1]])
		}
	}
}



impl Reinterpret for u32 { }

impl ReinterpretAsBytes for u32 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 4] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3]]
		}
	}
}

impl ReinterpretFromBytes for u32 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3]])
		}
	}
}



impl Reinterpret for i32 { }

impl ReinterpretAsBytes for i32 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 4] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3]]
		}
	}
}

impl ReinterpretFromBytes for i32 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3]])
		}
	}
}



impl Reinterpret for u64 { }

impl ReinterpretAsBytes for u64 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 8] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]
		}
	}
}

impl ReinterpretFromBytes for u64 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
		}
	}
}



impl Reinterpret for i64 { }

impl ReinterpretAsBytes for i64 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 8] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]
		}
	}
}

impl ReinterpretFromBytes for i64 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7]])
		}
	}
}



impl Reinterpret for u128 { }

impl ReinterpretAsBytes for u128 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 16] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2],  bytes[3],  bytes[4],  bytes[5],  bytes[6],  bytes[7],
				 bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]
		}
	}
}

impl ReinterpretFromBytes for u128 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2],  source[3],  source[4],  source[5],  source[6],  source[7],
					   source[8], source[9], source[10], source[11], source[12], source[13], source[14], source[15]])
		}
	}
}



impl Reinterpret for i128 { }

impl ReinterpretAsBytes for i128 {
	fn reinterpret_as_bytes(&self) -> Vec<u8> {
		unsafe {
			let bytes: [u8; 16] = transmute(*self);
			vec![bytes[0], bytes[1], bytes[2],  bytes[3],  bytes[4],  bytes[5],  bytes[6],  bytes[7],
				 bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]
		}
	}
}

impl ReinterpretFromBytes for i128 {
	fn reinterpret_from_bytes(source: &Vec<u8>) -> Self {
		unsafe {
			transmute([source[0], source[1], source[2],  source[3],  source[4],  source[5],  source[6],  source[7],
					   source[8], source[9], source[10], source[11], source[12], source[13], source[14], source[15]])
		}
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reinterpret_u8() {
		let before: u8 = 23;
		let after = u8::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_i8() {
		let before: i8 = 23;
		let after = i8::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_u16() {
		let before: u16 = 13243;
		let after = u16::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_i16() {
		let before: i16 = 1442;
		let after = i16::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_u32() {
		let before: u32 = 41432;
		let after = u32::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_i32() {
		let before: i32 = 2454;
		let after = i32::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_u64() {
		let before: u64 = 234;
		let after = u64::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_i64() {
		let before: i64 = 5424254;
		let after = i64::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_u128() {
		let before: u128 = 23452523453452334;
		let after = u128::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}

	#[test]
	fn reinterpret_i128() {
		let before: i128 = 243523452345;
		let after = i128::reinterpret_from_bytes(&before.reinterpret_as_bytes());

		assert_eq!(before, after);
	}
}