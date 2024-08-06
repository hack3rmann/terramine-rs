//!
//! Adds container to window stuff
//!

pub mod message_box;

use {
    crate::prelude::*, math_linear::prelude::*, winit::{
        dpi::PhysicalSize, error::{EventLoopError, OsError}, event_loop::EventLoop, window::{Icon, Window as WinitWindow, WindowBuilder}
    }
};

/// Wrapper around `winit`'s window.
#[derive(Debug, Deref, From, Into)]
pub struct Window {
    #[deref]
    pub inner: WinitWindow,
    pub event_loop: Nullable<EventLoop<()>>,
}

// TODO: block `event_loop` from usage
unsafe impl Send for Window { }
unsafe impl Sync for Window { }

impl Window {
    /// Constructs window.
    pub fn new(sizes: USize2) -> Result<Self, WindowCreationError> {
        let event_loop = EventLoop::new()?;

        let window = WindowBuilder::new()
            .with_title("Terramine")
            .with_resizable(true)
            .with_inner_size(PhysicalSize::new(sizes.x as u32, sizes.y as u32))
            .with_window_icon(Some(Self::load_icon()))
            .build(&event_loop)?;
        
        Ok(Self { inner: window, event_loop: Nullable::new(event_loop) })
    }

    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take()
    }

    fn load_icon() -> Icon {
        // Bytes vector from bmp file
        // File formatted in BGRA
        // TODO: reformat file to RGBA format so we can avoid byteswapping at the program start
        // FIXME: ico to bitmap converter
        let raw_data = include_bytes!("../../../image/icon.ico");
        let mut raw_data = *raw_data;

        // Bytemap pointer load from 4 bytes of file
        // This pointer is 4 bytes large and can be found on 10th byte position from file begin
        let start_bytes = (raw_data[13] as usize) << 24 |
                          (raw_data[12] as usize) << 16 |
                          (raw_data[11] as usize) << 8  |
                          (raw_data[10] as usize);

        // Trim useless information
        let raw_data = raw_data[start_bytes..].as_mut();

        // Converting BGRA into RGBA formats
        let mut current: usize = 0;
        while current <= raw_data.len() - 3 {
            raw_data.swap(current, current + 2);
            current += 4;
        }

        let mut data = Vec::with_capacity(raw_data.len());
        data.extend_from_slice(raw_data);
        Icon::from_rgba(data, 32, 32)
            .expect("failed to load icon")
    }
}

#[derive(Debug, Error)]
pub enum WindowCreationError {
    #[error(transparent)]
    Os(#[from] OsError),

    #[error(transparent)]
    EventLoop(#[from] EventLoopError),
}