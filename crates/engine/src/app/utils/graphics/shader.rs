//!
//! This module contains [`Shader`] struct.
//!

use {
    crate::app::utils::{
        cfg::shader::{DIRECTORY, FRAGMENT_FILE_EXTENTION, VERTEX_FILE_EXTENTION},
        logger,
    },
    derive_deref_rs::Deref,
    glium::ProgramCreationError,
    std::{fs, io},
    thiserror::Error,
};

/// Shader struct is container for shader source code.
#[derive(Debug, Deref)]
pub struct Shader {
    pub vertex_src: String,
    pub fragment_src: String,

    #[deref]
    pub program: glium::Program,
}

impl Shader {
    /// Returns new Shader object that contains shader source code from their path.
    /// It adds [`DIRECTORY`] before the name and special extention (a.g. `.vert` for vertex) after.
    pub fn new(
        vertex_name: &str,
        fragment_name: &str,
        display: &dyn glium::backend::Facade,
    ) -> Result<Self, ShaderError> {
        let _work_guard = logger::work("shader loader", format!("{vertex_name}, {fragment_name}."));

        let vertex_src =
            fs::read_to_string(format!("{DIRECTORY}{vertex_name}.{VERTEX_FILE_EXTENTION}"))
                .map_err(|err| ShaderError::FileRead {
                    io_err: err,
                    shader_name: vertex_name.into(),
                })?;

        let fragment_src = fs::read_to_string(format!(
            "{DIRECTORY}{fragment_name}.{FRAGMENT_FILE_EXTENTION}"
        ))
        .map_err(|err| ShaderError::FileRead {
            io_err: err,
            shader_name: fragment_name.into(),
        })?;

        Self::from_source(vertex_src, fragment_src, display)
    }

    pub fn from_source(
        vertex_src: String,
        fragment_src: String,
        display: &dyn glium::backend::Facade,
    ) -> Result<Self, ShaderError> {
        let program =
            glium::Program::from_source(display, vertex_src.as_str(), fragment_src.as_str(), None)?;

        Ok(Shader {
            vertex_src,
            fragment_src,
            program,
        })
    }
}

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("failed to create gl shader program: {0}")]
    ProgramCreation(#[from] ProgramCreationError),

    #[error("failed to read shader file, shader name: {shader_name}, io_err: {io_err}")]
    FileRead {
        io_err: io::Error,
        shader_name: String,
    },
}
