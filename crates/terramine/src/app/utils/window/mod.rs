pub mod message_box;

use {
    crate::prelude::*, math_linear::prelude::*, winit::{
        dpi::PhysicalSize,
        error::{EventLoopError, OsError},
        event_loop::EventLoop,
        window::{Icon, Window as WinitWindow, WindowBuilder},
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
    pub fn new(sizes: USize2) -> Result<Self, WindowCreationError> {
        let event_loop = EventLoop::new()?;

        let window = WindowBuilder::new()
            .with_title("Terramine")
            .with_resizable(true)
            .with_inner_size(PhysicalSize::new(sizes.x as u32, sizes.y as u32))
            .with_window_icon(Some(Self::load_icon()?))
            .build(&event_loop)?;
        
        Ok(Self { inner: window, event_loop: Nullable::new(event_loop) })
    }

    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take()
    }

    fn load_icon() -> Result<Icon, ReadIconError> {
        use { std::fs::File, ico::IconDir };

        let file = File::open("assets/images/icon.ico")?;

        let icon_dir = IconDir::read(file).unwrap();
        let image = icon_dir.entries()[0].decode().unwrap();

        let icon = Icon::from_rgba(
            image.rgba_data().to_vec(),
            image.width(),
            image.height(),
        )?;
    
        Ok(icon)
    }
}


#[derive(Debug, Error)]
pub enum ReadIconError {
    #[error("failed to open icon file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to read bad icon file: {0}")]
    BadIcon(#[from] winit::window::BadIcon),
}


#[derive(Debug, Error)]
pub enum WindowCreationError {
    #[error(transparent)]
    Os(#[from] OsError),

    #[error(transparent)]
    EventLoop(#[from] EventLoopError),

    #[error(transparent)]
    ReadIcon(#[from] ReadIconError),
}