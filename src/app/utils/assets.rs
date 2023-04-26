use {
    crate::prelude::*,
    std::path::Path,
    futures::future::BoxFuture,
    tokio::{fs, io},
};

pub trait FromFile: Sized {
    type Error: std::error::Error;

    fn from_file<'p>(path: impl AsRef<Path> + Send + Sync + 'p) -> BoxFuture<'p, Result<Self, Self::Error>>;
}

#[derive(Debug, Error)]
pub enum WithIoError<E: std::error::Error> {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(E),
}

impl<T, E> FromFile for T
where
    E: std::error::Error,
    T: FromSource<Source = String, Error = E>,
{
    type Error = WithIoError<E>;
    default fn from_file<'p>(path: impl AsRef<Path> + Send + Sync + 'p) -> BoxFuture<'p, Result<Self, Self::Error>> {
        Box::pin(async move {
            let file_content = fs::read_to_string(path.as_ref()).await?;
            Self::from_source(file_content)
                .map_err(|err| WithIoError::Other(err))
        })
    }
}

pub trait FromSource: Sized {
    type Source;
    type Error: std::error::Error;

    fn from_source(source: Self::Source) -> Result<Self, Self::Error>;
}