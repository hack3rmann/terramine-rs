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
        werror::prelude::*,
    },
    std::fs,
};

/// Shader struct is container for shader source code.
#[derive(Debug)]
pub struct Shader {
    pub vertex_src: String,
    pub fragment_src: String,
    pub program: glium::Program
}

impl Shader {
    /// Returns new Shader object that contains shader source code from their path.
    /// It adds [`DIRECTORY`] before the name and special extention (a.g. `.vert` for vertex) after.
    pub fn new(vertex_shader_name: &str, fragment_shader_name: &str, display: &glium::Display) -> Self {
        let vertex_shader_src = fs::read_to_string(
            format!("{DIRECTORY}{}.{VERTEX_FILE_EXTENTION}", vertex_shader_name)
        ).wexpect("Can't read vertex shader file!");

        let fragment_shader_src = fs::read_to_string(
            format!("{DIRECTORY}{}.{FRAGMENT_FILE_EXTENTION}", fragment_shader_name)
        ).wexpect("Can't read fragment shader file!");

        Shader {
            vertex_src: String::from(&vertex_shader_src),
            fragment_src: String::from(&fragment_shader_src),
            program: glium::Program::from_source(display, vertex_shader_src.as_str(), fragment_shader_src.as_str(), None).wunwrap()
        }
    }
}