use {
    crate::{
        prelude::*,
        graphics::{ShaderRef, SURFACE_CFG},
    },
    std::fmt::Debug,
};



pub use wgpu::{ColorTargetState, TextureFormat, ColorWrites, BlendState};



pub trait Material: Debug + Send + Sync + 'static {
    fn get_shader(&self) -> ShaderRef;
    fn get_color_states(&self) -> &[Option<ColorTargetState>];

    fn to_arc(self) -> Arc<dyn Material> where Self: Sized {
        Arc::from(self)
    }

    fn to_box(self) -> Box<dyn Material> where Self: Sized {
        Box::from(self)
    }
}
assert_obj_safe!(Material);

impl Default for Box<dyn Material> {
    fn default() -> Self {
        Box::from(DefaultMaterial)
    }
}



lazy_static! {
    static ref DEFAULT_MATERIAL_COLOR_TARGET_STATES: Vec<Option<ColorTargetState>> = vec![Some({
        let format = SURFACE_CFG.read().format;
        ColorTargetState {
            format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        }
    })];
}



#[derive(Debug)]
pub struct DefaultMaterial;
assert_impl_all!(DefaultMaterial: Send, Sync, Component);

impl Material for DefaultMaterial {
    fn get_shader(&self) -> ShaderRef {
        default()
    }

    fn get_color_states(&self) -> &'static [Option<ColorTargetState>] {
        &DEFAULT_MATERIAL_COLOR_TARGET_STATES
    }
}



#[derive(Debug)]
pub struct StandartMaterial {
    pub shader: ShaderRef,
}
assert_impl_all!(StandartMaterial: Send, Sync, Component);

impl<S: Into<ShaderRef>> From<S> for StandartMaterial {
    fn from(value: S) -> Self {
        Self { shader: value.into() }
    }
}

impl Material for StandartMaterial {
    fn get_shader(&self) -> ShaderRef {
        self.shader.clone()
    }

    fn get_color_states(&self) -> &[Option<ColorTargetState>] {
        &DEFAULT_MATERIAL_COLOR_TARGET_STATES
    }
}