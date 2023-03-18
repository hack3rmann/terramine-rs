/**
 *  This file contains Shader struct.
 */

use {
    crate::app::utils::{
        cfg::shader::{
            DIRECTORY,
            VERTEX_FILE_EXTENTION,
            FRAGMENT_FILE_EXTENTION
        },
    },
    std::{fs, io},
    thiserror::Error,
    glium::ProgramCreationError,
};

/// Shader struct is container for shader source code.
#[derive(Debug)]
pub struct Shader {
    pub vertex_src: String,
    pub fragment_src: String,
    pub program: glium::Program,
}

impl Shader {
    /// Returns new Shader object that contains shader source code from their path.
    /// It adds [`DIRECTORY`] before the name and special extention (a.g. `.vert` for vertex) after.
    pub fn new(
        vertex_shader_name: &str,
        fragment_shader_name: &str,
        display: &dyn glium::backend::Facade
    ) -> Result<Self, ShaderError> {
        let vertex_src = fs::read_to_string(
            format!("{DIRECTORY}{}.{VERTEX_FILE_EXTENTION}", vertex_shader_name)
        ).map_err(|err| ShaderError::FileRead { io_err: err, shader_name: vertex_shader_name.into() })?;

        let fragment_src = fs::read_to_string(
            format!("{DIRECTORY}{}.{FRAGMENT_FILE_EXTENTION}", fragment_shader_name)
        ).map_err(|err| ShaderError::FileRead { io_err: err, shader_name: fragment_shader_name.into() })?;

        Self::from_source(vertex_src, fragment_src, display)
    }

    pub fn from_source(vertex_src: String, fragment_src: String, display: &dyn glium::backend::Facade) -> Result<Self, ShaderError> {
        let program = glium::Program::from_source(
            display,
            vertex_src.as_str(),
            fragment_src.as_str(),
            None,
        )?;

        Ok(Shader { vertex_src, fragment_src, program })
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