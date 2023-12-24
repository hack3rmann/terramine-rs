use {
    crate::prelude::*,
    std::cell::UnsafeCell,
};



#[derive(Debug, SmartDefault)]
#[default(Self::DEFAULT)]
pub struct GlobalAsset<T> {
    pub inner: UnsafeCell<Option<T>>,
}
assert_impl_all!(GlobalAsset<f32>: Send, Sync);

unsafe impl<T: Send> Send for GlobalAsset<T> { }
unsafe impl<T: Sync> Sync for GlobalAsset<T> { }

impl<T> GlobalAsset<T> {
    pub const fn new(asset: T) -> Self {
        Self { inner: UnsafeCell::new(Some(asset)) }
    }

    pub const fn unloaded() -> Self {
        Self { inner: UnsafeCell::new(None) }
    }

    /// Uploads asset into.
    /// 
    /// # Safety
    /// 
    /// - no one used or uses this [asset][GlobalAsset].
    /// - called only once.
    pub unsafe fn upload(&self, asset: T) {
        let inner = self.inner.get().as_mut().unwrap_unchecked();

        if inner.is_some() {
            panic!("asset can be uploaded only once");
        }

        *inner = Some(asset);
    }
}

impl<T> ConstDefault for GlobalAsset<T> {
    const DEFAULT: Self = Self::unloaded();
}

impl<T> Deref for GlobalAsset<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let asset = unsafe {
            self.inner.get().as_ref().unwrap_unchecked()
        };

        asset.as_ref()
            .expect("default asset should be loaded before usage")
    }
}

impl<T> DerefMut for GlobalAsset<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.get_mut()
            .as_mut()
            .expect("default asset should be loaded before usage")
    }
}

impl<T> From<T> for GlobalAsset<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}



#[async_trait]
pub trait FromFile: Sized {
    type Error;
    async fn from_file(file_name: impl AsRef<Path> + Send) -> Result<Self, Self::Error>;
}
