
//! PROTOTYPE!

#[profiler_target]
pub fn some_funky_func(args: [i32; 12]) -> String {
	todo!()
}

//! Truns to:

pub fn some_funky_func(args: [i32; 12]) -> String {
	let __profiler = Profiler::start_capture("some_funky_func()");

	/* Some funky code */

	std::mem::drop(__profiler);

	todo!()
}

//! RESULT:
//? some_funky_func() * 13 = 0.1% = 132 ns