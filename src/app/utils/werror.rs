use {
	crate::app::utils::window::message_box::MessageBox,
	std::fmt::Debug,
};

/// Standart panic error.
pub trait WErrorForward<T, E: Debug> {
	fn wexpect(self, msg: &str) -> T;
	fn wunwrap(self) -> T;
}

/// Standart panic error.
pub trait WErrorBackward<T: Debug, E> {
	fn wexpect_err(self, msg: &str) -> E;
	fn wunwrap_err(self) -> E;
}

impl<T, E: Debug> WErrorForward<T, E> for Result<T, E> {
	#[track_caller]
	fn wexpect(self, msg: &str) -> T {
		match self {
			Ok(t) => t,
			Err(e) => {
				error_message(msg, &e).unwrap();
				Err(e).expect(msg)
			}
		}
	}

	#[track_caller]
	fn wunwrap(self) -> T {
		match self {
			Ok(t) => t,
			Err(e) => {
				error_message("called `Result::wunwrap()` on an `Err` value", &e).unwrap();
				Err(e).unwrap()
			}
		}
	}
}

impl<T: Debug, E> WErrorBackward<T, E> for Result<T, E> {
	#[track_caller]
	fn wexpect_err(self, msg: &str) -> E {
		match self {
			Err(e) => e,
			Ok(t) => {
				error_message(msg, &t).unwrap();
				Ok(t).expect_err(msg)
			}
		}
	}

	#[track_caller]
	fn wunwrap_err(self) -> E {
		match self {
			Err(e) => e,
			Ok(t) => {
				error_message("called `Result::wunwrap_err()` on an `Err` value", &t).unwrap();
				Ok(t).unwrap_err()
			}
		}
	}
}

impl<T> WErrorForward<T, ()> for Option<T> {
	#[track_caller]
	fn wexpect(self, msg: &str) -> T {
		match self {
			Some(t) => t,
			None => {
				message(msg).unwrap();
				None.expect(msg)
			}
		}
	}

	#[track_caller]
	fn wunwrap(self) -> T {
		match self {
			Some(t) => t,
			None => {
				message("called `Option::wunwrap()` on an `None` value").unwrap();
				None.unwrap()
			}
		}
	}
}

/// Constructs error message box.
pub fn error_message(msg: &str, error: &dyn Debug) -> Result<(), u32> {
	MessageBox::new("Error message:", &format!("{msg}: {error:?}")).errored().show()
}

/// Constructs error message box.
pub fn message(msg: &str) -> Result<(), u32> {
	MessageBox::new("Error message:", &format!("{msg}: no details available.")).errored().show()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test() {
		let result: Result<u32, u32> = Err(15);
		result.wunwrap();
	}
}