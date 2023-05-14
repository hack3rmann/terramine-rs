use {
    crate::{
        prelude::*,
        graphics::{Shader, ShaderSource, Device, DefaultVertex, Vertex},
    },
    std::cell::UnsafeCell,
};



pub static DEFAULT_SHADER: GlobalAsset<Arc<Shader>> = GlobalAsset::unloaded();

/// Uploads shader into [`DEFAULT_SHADER`].
/// 
/// # Safety
/// 
/// - no one used this [asset][GlobalAsset].
/// - called only once.
pub async unsafe fn load_default_shader(device: &Device) {
    let source = ShaderSource::from_file("default.wgsl")
        .await
        .expect("failed to load default.wgsl");

    let shader = Shader::new(device, source, vec![DefaultVertex::BUFFER_LAYOUT]);

    DEFAULT_SHADER.upload(Arc::new(shader))
}

/// Uploads default assets.
/// 
/// # Safety
/// 
/// - no one used any default [asset][GlobalAsset]s.
/// - called only once.
pub async unsafe fn load_default_assets(device: &Device) {
    load_default_shader(device).await;
}



#[derive(Clone, Debug, Default)]
pub enum ShaderRef {
    #[default]
    Default,
    Module(Arc<Shader>),
}
assert_impl_all!(ShaderRef: Send, Sync);

impl From<Shader> for ShaderRef {
    fn from(value: Shader) -> Self {
        Self::from(Arc::new(value))
    }
}

impl From<&Arc<Shader>> for ShaderRef {
    fn from(value: &Arc<Shader>) -> Self {
        Self::from(Arc::clone(value))
    }
}

impl From<Arc<Shader>> for ShaderRef {
    fn from(value: Arc<Shader>) -> Self {
        Self::Module(value)
    }
}

impl ShaderRef {
    pub fn as_module(&self) -> Arc<Shader> {
        match self {
            Self::Default => Arc::clone(&DEFAULT_SHADER),
            Self::Module(module) => Arc::clone(module),
        }
    }
}



#[derive(Debug, SmartDefault)]
pub struct GlobalAsset<T> {
    #[default(UnsafeCell::new(None))]
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
