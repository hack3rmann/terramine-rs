/**
 *  This file contains Shader struct.
 */

use std::fs;

/* Shader struct is container for shader source code */
pub struct Shader {
	pub vertex_src: String,
	pub fragment_src: String,
	pub program: glium::Program
}

impl Shader {
	/* Returns new Shader object that contains shader source code from their path */
	pub fn new(vertex_shader_path: &str, fragment_shader_path: &str, display: &glium::Display) -> Self {
		/* File reading */
		let vertex_shader_src = fs::read_to_string(vertex_shader_path).expect("Can't read vertex shader file!");
		let fragment_shader_src = fs::read_to_string(fragment_shader_path).expect("Can't read fragment shader file!");

		/* Construct the struct */
		Shader {
			vertex_src: String::from(&vertex_shader_src),
			fragment_src: String::from(&fragment_shader_src),
			program: glium::Program::from_source(display, vertex_shader_src.as_str(), fragment_shader_src.as_str(), None).unwrap()
		}
	}
}