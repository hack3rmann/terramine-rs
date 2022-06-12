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