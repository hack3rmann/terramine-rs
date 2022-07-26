use crate::app::utils::werror::prelude::*;

pub mod prelude {
	pub use super::{
		runtime,
	};
}

static mut RUNTIME: Option<tokio::runtime::Runtime> = None;

pub fn initialyze() {
	unsafe {
		RUNTIME.replace(
			tokio::runtime::Builder::new_multi_thread()
				.enable_all()
				.build()
				.wunwrap()
		)
	};
}

pub fn runtime<'l>() -> &'l tokio::runtime::Runtime {
	unsafe {
		RUNTIME.as_ref().wexpect("Runtime is not initialyzed!")
	}
}