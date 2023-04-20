pub mod message_box;

/**
 *  Adds container to window stuff
 */

use {
    crate::prelude::*,
    winit::{
        window::{WindowBuilder, Window as WinitWindow, Icon},
        event_loop::EventLoop,
        dpi::PhysicalSize,
    },
    math_linear::prelude::*,
};

/// Wrapper around `winit`'s window.
#[derive(Debug, Deref)]
pub struct Window {
    pub inner: WinitWindow,
}

impl Window {
    /// Constructs window.
    pub fn from(event_loop: &EventLoop<()>, sizes: USize2) -> Result<Self, winit::error::OsError> {
        let window = WindowBuilder::new()
            .with_title("Terramine")
            .with_resizable(true)
            .with_inner_size(PhysicalSize::new(sizes.x as u32, sizes.y as u32))
            .with_window_icon(Some(Self::load_icon()))
            .build(event_loop)?;
        
        Ok(Self { inner: window })
    }

    fn load_icon() -> Icon {
        // Bytes vector from bmp file
        // File formatted in BGRA
        let raw_data = include_bytes!("../../../image/terramine_icon_32p.bmp");
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
            .expect("length of data should be divisible by 4, \
                     and width * height must equal data.len() / 4")
    }
}