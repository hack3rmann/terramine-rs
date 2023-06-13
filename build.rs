use std::io;



fn main() -> io::Result<()> {
    set_icon()
}



#[cfg(any(windows, unix, mac))]
fn set_icon() -> io::Result<()> {
    #[cfg(windows)]
    set_icon_windows()?;

    #[cfg(unix)]
    set_icon_unix()?;

    #[cfg(mac)]
    set_icon_macos()?;

    Ok(())
}

#[cfg(windows)]
fn set_icon_windows() -> io::Result<()> {
    use winres::WindowsResource;

    WindowsResource::new()
        .set_icon("src/image/icon.ico")
        .compile()?;

    Ok(())
}

#[cfg(unix)]
fn set_icon_unix() -> io::Result<()> {
    todo!()
}

#[cfg(mac)]
fn set_icon_macos() -> io::Result<()> {
    todo!()
}