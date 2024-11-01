use std::io;


fn main() -> io::Result<()> {
    platform::set_icon()
}


#[cfg(windows)]
mod platform {
    use std::io;


    pub fn set_icon() -> io::Result<()> {
        use winres::WindowsResource;

        WindowsResource::new()
            .set_icon("assets/images/icon.ico")
            .compile()?;

        Ok(())
    }
}


#[cfg(not(windows))]
mod platform {
    pub fn set_icon() -> std::io::Result<()> { Ok(()) }
}
