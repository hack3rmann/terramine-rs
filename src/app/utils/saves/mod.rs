pub mod stack_heap;

use std::marker::PhantomData;

use super::reinterpreter::Reinterpret;

pub struct Save<E> {
	name: String,
	_phantom_data: PhantomData<E>,
}

impl<E: Into<u64>> Save<E> {
	pub fn new(name: &str) -> Self {
		Self { name: name.to_owned(), _phantom_data: PhantomData }
	}

	pub fn write(value: &impl Reinterpret, enumerator: E) -> Self {
		todo!()
	}
}