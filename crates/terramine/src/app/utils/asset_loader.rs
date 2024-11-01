use { crate::prelude::*, tokio::task::JoinHandle };



pub struct OwnedAsset {
    pub(crate) value: Box<dyn Any + Send + Sync>,
}
assert_impl_all!(OwnedAsset: Send, Sync);

impl OwnedAsset {
    pub fn new(value: impl Any + Send + Sync) -> Self {
        Self { value: Box::new(value) }
    }

    pub fn get<A: Any>(&self) -> Option<&A> {
        self.value.downcast_ref()
    }

    pub fn get_mut<A: Any>(&mut self) -> Option<&mut A> {
        self.value.downcast_mut()
    }

    pub fn get_value<A: Any>(self) -> Result<Box<A>, Self> {
        self.value.downcast().map_err(|any| Self { value: any })
    }
}

impl Debug for OwnedAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}



pub struct Parse<T> {
    pub call: Arc<dyn Fn(Vec<u8>) -> AnyResult<T> + Send + Sync + 'static>,
}
assert_impl_all!(Parse<i32>: Send, Sync);

impl<T> Debug for Parse<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!("Parse<{}>", std::any::type_name::<T>()))
            .finish_non_exhaustive()
    }
}

impl<T, F> From<F> for Parse<T>
where
    F: Fn(Vec<u8>) -> AnyResult<T> + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self { call: Arc::new(value) }
    }
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Self {
        Self { call: Arc::clone(&self.call) }
    }
}



pub struct LoadingProccess {
    pub(crate) join: Option<JoinHandle<AnyResult<OwnedAsset>>>,
    pub(crate) parse: Parse<OwnedAsset>,
    pub(crate) path: PathBuf,
}

impl LoadingProccess {
    pub fn new<T>(parse: impl Into<Parse<T>>, path: PathBuf) -> Self
    where
        T: Any + Send + Sync,
    {
        let parse = parse.into();

        Self {
            join: None,
            path,
            parse: Parse { call: Arc::new(move |src| (parse.call)(src)
                .map(|value| OwnedAsset::new(value))
            )},
        }
    }

    pub fn start(&mut self) {
        match self.join {
            Some(_) => logger::error!(
                from = "asset-loader",
                "failed to start already started loading proccess, aborting"
            ),
            None => {
                let parse = self.parse.clone();
                let path = mem::take(&mut self.path);

                self.join = Some(tokio::spawn(async move {
                    use tokio::fs;

                    let contents = fs::read(&path).await
                        .with_context(||
                            format!("failed to read a file {:?}", &path)
                        )?;

                    (parse.call)(contents)
                }));
            },
        }
    }

    pub fn is_finishable(&self) -> bool {
        self.join.is_some()
    }

    pub fn is_finished(&self) -> bool {
        let Some(ref handle) = self.join else { return true };

        handle.is_finished()
    }

    pub async fn finish(&mut self) -> Option<OwnedAsset> {
        let handle = self.join.take()?;

        match handle.await {
            Ok(Ok(value)) => Some(value),

            Err(err) => {
                logger::error!(
                    from = "asset-loader",
                    "failed to finish asset loading: {err}",
                );

                None
            },

            Ok(Err(err)) => {
                logger::error!(
                    from = "asset-loader",
                    "failed to finish asset loading: {err}",
                );

                None
            }
        }
    }

    pub async fn try_finish(&mut self) -> Option<OwnedAsset> {
        if !self.is_finished() {
            return None;
        }

        self.finish().await
    }
}

impl Debug for LoadingProccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadingProccess")
            .field("join", &self.join)
            .finish_non_exhaustive()
    }
}



// TODO: emit `Loaded(path)` event when somthing had loaded
#[derive(Debug, Default)]
pub struct AssetLoader {
    pub loaded: HashMap<PathBuf, OwnedAsset>,
    pub unloaded: HashMap<PathBuf, LoadingProccess>,
}
assert_impl_all!(AssetLoader: Send, Sync, Component);

impl AssetLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extract<A: Any>(&mut self, path: impl AsRef<Path>) -> Option<Box<A>> {
        match self.loaded.remove(path.as_ref()) {
            Some(owned) => owned.get_value::<A>().ok(),
            None => None,
        }
    }

    pub fn get<A: Any>(&self, path: impl AsRef<Path>) -> Option<&A> {
        self.loaded.get(path.as_ref())
            .and_then(OwnedAsset::get)
    }

    pub fn get_mut<A: Any>(&mut self, path: impl AsRef<Path>) -> Option<&mut A> {
        self.loaded.get_mut(path.as_ref())
            .and_then(OwnedAsset::get_mut)
    }

    pub fn get_or_start<A: Any + Send + Sync>(
        &mut self, path: impl AsRef<Path>, parse: impl Into<Parse<A>>,
    ) -> Option<&mut A> {
        let path = path.as_ref();

        if self.is_loading(path) {
            return None;
        }

        if !self.is_loaded(path) {
            self.start_loading(path, parse);
            return None;
        }

        self.get_mut::<A>(path)
    }

    pub fn start_loading<A: Any + Send + Sync>(
        &mut self, path: impl Into<PathBuf>, parse: impl Into<Parse<A>>,
    ) {
        let path = path.into();
        let parse = parse.into();

        let mut proccess = LoadingProccess::new(parse, path.clone());
        proccess.start();

        self.unloaded.insert(path, proccess);
    }

    pub fn is_loading(&self, path: impl AsRef<Path>) -> bool {
        self.unloaded.contains_key(path.as_ref())
    }

    pub fn is_loaded(&self, path: impl AsRef<Path>) -> bool {
        self.loaded.contains_key(path.as_ref())
    }

    pub fn contains(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        self.is_loading(path) || self.is_loaded(path)
    }

    pub async fn try_finish_all(&mut self) {
        let mut finished = vec![];

        for (path, proccess) in self.unloaded.iter_mut() {
            if !proccess.is_finishable() {
                finished.push(path.to_owned());
                continue;
            }

            if let Some(asset) = proccess.try_finish().await {
                self.loaded.insert(path.to_owned(), asset);
                finished.push(path.to_owned());
            }
        }

        for path in finished {
            self.unloaded.remove(&path);
        }
    }
}