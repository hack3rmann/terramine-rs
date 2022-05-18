use std::fs;

pub struct Shader {
	pub vertex_src: String,
	pub fragment_src: String
}

impl Shader {
	pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
		let vertex_shader_src = fs::read_to_string(vertex_shader_path).expect("Can't read vertex shader file!");
		let fragment_shader_src = fs::read_to_string(fragment_shader_path).expect("Can't read fragment shader file!");

		let shader = Shader { vertex_src: String::from(vertex_shader_src), fragment_src: String::from(fragment_shader_src) };

		return shader;
	}
}