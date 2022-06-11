
//! PROTOTYPE!

#[profiler_target]
pub fn some_funky_func(args: [i32; 12]) -> String {
	todo!()
}

//! Turns to:

pub fn some_funky_func(args: [i32; 12]) -> String {
	static __id: ID = random!(); // Controlled by #[profiler_target] macro
	let __measure = profiler::start_capture("some_funky_func()", __id);

	/* Some funky code */

	todo!()

	// std::mem::drop(__measure);
	//	Measure::drop() {
	//		profiler::upload_measure(self)
	//	}
}

//! RESULT:
//? some_funky_func() * 13 = 0.1% = 132 ns