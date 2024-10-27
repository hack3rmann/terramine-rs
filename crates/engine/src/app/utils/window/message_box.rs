#![allow(dead_code)]

use {
    derive_deref_rs::Deref,
    std::ffi::CString,
};

#[cfg(windows)]
winapi::{
    um::{
        winuser::MessageBoxA,
        errhandlingapi::GetLastError,
    },
    shared::{
        windef::HWND,
        ntdef::LPCSTR,
        minwindef::UINT
    },
};

#[derive(Clone, Debug)]
pub struct MessageBox {
    title: CString,
    body:  CString,
    flags: Flags,
}

impl MessageBox {
    /// Constructs new [`MessageBox`] with given values.
    pub fn with_flags(title: &str, body: &str, flags: Flags) -> Self {
        let title = unsafe {
            let bytes: Vec<_> = title.bytes().collect();
            CString::from_vec_unchecked(bytes)
        };
        let body = unsafe {
            let bytes: Vec<_> = body.bytes().collect();
            CString::from_vec_unchecked(bytes)
        };

        Self { title, body, flags }
    }

    /// Constructs new [`MessageBox`] with default flags.
    pub fn new(title: &str, body: &str) -> Self {
        Self::with_flags(title, body, Default::default())
    }

    /// Configures error flags.
    pub fn errored(mut self) -> Self {
        use flags::*;
        self.flags.button = button::OK;
        self.flags.icon = icon::ERROR;
        self
    }

    /// Configures info flags.
    pub fn infod(mut self) -> Self {
        self.flags = Default::default();
        self
    }

    /// Configures custom flags.
    pub fn cfg_flags(mut self, flags: Flags) -> Self {
        self.flags.merge(&flags);
        self
    }

    /// Configures `other` part of flags.
    pub fn cfg_other(mut self, other: u32) -> Self {
        self.flags.other |= other;
        self
    }

    /// Shows message.
    pub fn show(self) -> Result<MessageBoxSuccess, MessageBoxError> {
        unsafe {
            message_box(std::ptr::null_mut(), self.body.as_ptr(), self.title.as_ptr(), self.flags.compose())
                .map_err(|_err| MessageBoxError(GetLastError() as i32))
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Flags {
    pub button: u32,
    pub icon: u32,
    pub default_button: u32,
    pub modal: u32,
    pub other: u32,
}

impl Flags {
    /// Composes flags into one value.
    pub fn compose(self) -> u32 {
        self.button | self.icon | self.default_button | self.modal | self.other
    }

    /// Merges two flags with respect to new values.
    pub fn merge(&mut self, other: &Self) {
        self.button = other.button;
        self.icon = other.icon;
        self.default_button = other.default_button;
        self.modal = other.modal;
        self.other |= other.other;
    }
}

impl Default for Flags {
    fn default() -> Self {
        use flags::*;
        Self {
            button: button::OK,
            icon: icon::INFORMATION,
            default_button: default_button::ONE,
            modal: modal::APP,
            other: 0,
        }
    }
}

#[allow(dead_code)]
pub mod flags {
    /* Only one */
    pub mod button {
        pub const ABORT_RETRY_IGNORE:		u32 = 0x00000002;
        pub const CANCEL_RETRY_CONTINUE:	u32 = 0x00000006;
        pub const HELP:						u32 = 0x00004000;
        pub const OK:						u32 = 0x00000000;
        pub const OK_CANCEL:				u32 = 0x00000001;
        pub const RETRY_CANCEL:				u32 = 0x00000005;
        pub const YES_NO:					u32 = 0x00000004;
        pub const YES_NO_CANCEL:			u32 = 0x00000003;
    }
    /* Only one */
    pub mod icon {
        pub const EXCLAMATION:				u32 = 0x00000030;
        pub const WARNING:					u32 = 0x00000030;
        pub const INFORMATION:				u32 = 0x00000040;
        pub const ASTERISK:					u32 = 0x00000040;
        pub const QUESTION:					u32 = 0x00000020;
        pub const STOP:						u32 = 0x00000010;
        pub const ERROR:					u32 = 0x00000010;
        pub const HAND:						u32 = 0x00000010;
    }
    /* Only one */
    pub mod default_button {
        pub const ONE:						u32 = 0x00000000;
        pub const TWO:						u32 = 0x00000100;
        pub const THREE:					u32 = 0x00000200;
        pub const FOUR:						u32 = 0x00000300;
    }
    /* Only one */
    pub mod modal {
        pub const APP:						u32 = 0x00000000;
        pub const SYSTEM:					u32 = 0x00001000;
        pub const TASK:						u32 = 0x00002000;
    }
    /* One or more */
    pub mod other {
        pub const DEFAULT_DESKTOP_ONLY:		u32 = 0x00020000;
        pub const RIGHT:					u32 = 0x00080000;
        pub const RTL_READING:				u32 = 0x00100000;
        pub const SET_FOREGROUND:			u32 = 0x00010000;
        pub const TOP_MOST:					u32 = 0x00040000;
        pub const SERVICE_NOTIFICATION:		u32 = 0x00200000;
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq, Default)]
pub enum MessageBoxSuccess {
    #[default]
    Ok = 1,

    Cancel = 2,
    Abort = 3,
    Retry = 4,
    Ignore = 5,
    Yes = 6,
    No = 7,
    TryAgain = 10,
    Continue = 11,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq, Default, Deref)]
pub struct MessageBoxError(i32);

unsafe fn message_box(
    hwnd: HWND, lp_text: LPCSTR, lp_caption: LPCSTR, flags: UINT
) -> Result<MessageBoxSuccess, MessageBoxError> {
    match MessageBoxA(hwnd, lp_text, lp_caption, flags) {
        1  => Ok(MessageBoxSuccess::Ok),
        2  => Ok(MessageBoxSuccess::Cancel),
        3  => Ok(MessageBoxSuccess::Abort),
        4  => Ok(MessageBoxSuccess::Retry),
        5  => Ok(MessageBoxSuccess::Ignore),
        6  => Ok(MessageBoxSuccess::Yes),
        7  => Ok(MessageBoxSuccess::No),
        10 => Ok(MessageBoxSuccess::TryAgain),
        11 => Ok(MessageBoxSuccess::Continue),
        other => Err(MessageBoxError(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        MessageBox::new("Info!", "Ahahha").infod().show().unwrap();
    }
}
