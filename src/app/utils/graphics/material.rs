use {
    crate::{
        prelude::*,
        graphics::ShaderRef,
    },
    std::fmt::Debug,
};



pub use wgpu::{ColorTargetState, TextureFormat, ColorWrites};



pub trait Material: Debug + Send + Sync + 'static {
    fn get_shader(&self) -> ShaderRef;
    fn get_color_states(&self) -> &[Option<ColorTargetState>];
}
assert_obj_safe!(Material);



#[derive(Debug)]
pub struct DefaultMaterial;
assert_impl_all!(DefaultMaterial: Send, Sync, Component);

impl DefaultMaterial {
    pub const COLOR_TARGET_STATES: &'static [Option<ColorTargetState>] = &[Some(ColorTargetState {
        format: TextureFormat::Bgra8Unorm,
        blend: None,
        write_mask: ColorWrites::ALL,
    })];

    pub fn new_arc() -> Arc<dyn Material> {
        Arc::from(Self)
    }

    pub fn new_box() -> Box<dyn Material> {
        Box::from(Self)
    }
}

impl Material for DefaultMaterial {
    fn get_shader(&self) -> ShaderRef {
        default()
    }

    fn get_color_states(&self) -> &[Option<ColorTargetState>] {
        Self::COLOR_TARGET_STATES
    }
}



impl Default for Box<dyn Material> {
    fn default() -> Self {
        Box::from(DefaultMaterial)
    }
}