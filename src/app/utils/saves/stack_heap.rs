use std::fs::File;

pub struct StackHeap {
	pub stack: File,
	pub heap: File
}

impl StackHeap {
	pub fn new(path: &str, name: &str) -> Self {
		Self {
			stack: File::create(format!("{path}/{name}.stk")).unwrap(),
			heap:  File::create(format!("{path}/{name}.hp")).unwrap()
		}
	}
}