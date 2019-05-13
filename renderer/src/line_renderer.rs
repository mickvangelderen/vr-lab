use crate::parameters;
use crate::convert::*;
use crate::gl_ext::*;
use crate::shader_defines;
use gl_typed as gl;
use gl_typed::convert::*;

pub struct Renderer {
    pub program_name: gl::ProgramName,
    pub vertex_shader_name: gl::ShaderName,
    pub fragment_shader_name: gl::ShaderName,
    pub vertex_array_name: gl::VertexArrayName,
    pub vertex_buffer_name: gl::BufferName,
    pub element_buffer_name: gl::BufferName,
    pub pos_from_obj_to_wld_loc: gl::OptionUniformLocation,
    pub view_dep_uniforms: parameters::ViewDependentUniforms,
    pub view_ind_uniforms: parameters::ViewIndependentUniforms,
}

pub struct Parameters<'a> {
    pub framebuffer: Option<gl::FramebufferName>,
    pub width: i32,
    pub height: i32,
    pub view_dep_params: parameters::ViewDependentParameters,
    pub view_ind_params: parameters::ViewIndependentParameters,
    pub vertices: &'a [[f32; 3]],
    pub indices: &'a [[u32; 2]],
}

#[derive(Default)]
pub struct Update<B: AsRef<[u8]>> {
    pub vertex_shader: Option<B>,
    pub fragment_shader: Option<B>,
}

impl<B: AsRef<[u8]>> Update<B> {
    pub fn should_update(&self) -> bool {
        self.vertex_shader.is_some() || self.fragment_shader.is_some()
    }
}

impl Renderer {
    pub fn render<'a>(&self, gl: &gl::Gl, params: &Parameters<'a>) {
        unsafe {
            gl.enable(gl::DEPTH_TEST);
            gl.enable(gl::CULL_FACE);
            gl.cull_face(gl::BACK);
            gl.viewport(0, 0, params.width, params.height);
            gl.bind_framebuffer(gl::FRAMEBUFFER, params.framebuffer);
            gl.draw_buffers(&[gl::COLOR_ATTACHMENT0.into(), gl::COLOR_ATTACHMENT1.into()]);

            gl.use_program(self.program_name);

            if let Some(loc) = self.pos_from_obj_to_wld_loc.into() {
                gl.uniform_matrix4f(loc, gl::MajorAxis::Column, params.view_ind_params.light_pos_from_cam_to_wld.as_ref());
            }

            self.view_ind_uniforms.set(gl, params.view_ind_params);
            self.view_dep_uniforms.set(gl, params.view_dep_params);

            gl.bind_vertex_array(self.vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, self.vertex_buffer_name);
            gl.buffer_data(gl::ARRAY_BUFFER, params.vertices.flatten(), gl::STREAM_DRAW);

            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer_name);
            gl.buffer_data(gl::ELEMENT_ARRAY_BUFFER, params.indices.flatten(), gl::STATIC_DRAW);

            gl.draw_elements(gl::LINES, params.indices.flatten().len(), gl::UNSIGNED_INT, 0);

            gl.unbind_vertex_array();

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.unuse_program();
        }
    }

    pub fn update<B: AsRef<[u8]>>(&mut self, gl: &gl::Gl, update: Update<B>) {
        unsafe {
            let mut should_link = false;

            if let Some(bytes) = update.vertex_shader {
                self.vertex_shader_name
                    .compile(gl, &[shader_defines::VERSION, shader_defines::DEFINES, bytes.as_ref()])
                    .unwrap_or_else(|e| eprintln!("{} (vertex):\n{}", file!(), e));
                should_link = true;
            }

            if let Some(bytes) = update.fragment_shader {
                self.fragment_shader_name
                    .compile(gl, &[shader_defines::VERSION, shader_defines::DEFINES, bytes.as_ref()])
                    .unwrap_or_else(|e| eprintln!("{} (fragment):\n{}", file!(), e));
                should_link = true;
            }

            if should_link {
                self.program_name
                    .link(gl)
                    .unwrap_or_else(|e| eprintln!("{} (program):\n{}", file!(), e));

                gl.use_program(self.program_name);

                self.pos_from_obj_to_wld_loc = get_uniform_location!(gl, self.program_name, "pos_from_obj_to_wld");
                self.view_ind_uniforms.update(gl, self.program_name);
                self.view_dep_uniforms.update(gl, self.program_name);

                gl.unuse_program();
            }
        }
    }

    pub fn new(gl: &gl::Gl) -> Self {
        unsafe {
            let vertex_shader_name = gl.create_shader(gl::VERTEX_SHADER).expect("Failed to create shader.");

            let fragment_shader_name = gl.create_shader(gl::FRAGMENT_SHADER).expect("Failed to create shader.");

            let program_name = gl.create_program().expect("Failed to create program_name.");
            gl.attach_shader(program_name, vertex_shader_name);
            gl.attach_shader(program_name, fragment_shader_name);

            let [vertex_array_name]: [gl::VertexArrayName; 1] = {
                let mut names: [Option<gl::VertexArrayName>; 1] = std::mem::uninitialized();
                gl.gen_vertex_arrays(&mut names);
                names.try_transmute_each().unwrap()
            };

            let [vertex_buffer_name, element_buffer_name]: [gl::BufferName; 2] = {
                let mut names: [Option<gl::BufferName>; 2] = std::mem::uninitialized();
                gl.gen_buffers(&mut names);
                names.try_transmute_each().unwrap()
            };

            gl.bind_vertex_array(vertex_array_name);
            gl.bind_buffer(gl::ARRAY_BUFFER, vertex_buffer_name);
            gl.buffer_reserve(gl::ARRAY_BUFFER, 4, gl::STREAM_DRAW);
            gl.vertex_attrib_pointer(
                shader_defines::VS_POS_IN_OBJ_LOC,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<[f32; 3]>(),
                0,
            );
            gl.enable_vertex_attrib_array(shader_defines::VS_POS_IN_OBJ_LOC);
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer_name);
            gl.buffer_reserve(gl::ELEMENT_ARRAY_BUFFER, 4, gl::STATIC_DRAW);
            gl.unbind_vertex_array();
            gl.unbind_buffer(gl::ARRAY_BUFFER);
            gl.unbind_buffer(gl::ELEMENT_ARRAY_BUFFER);

            Renderer {
                program_name,
                vertex_shader_name,
                fragment_shader_name,
                vertex_array_name,
                vertex_buffer_name,
                element_buffer_name,
                pos_from_obj_to_wld_loc: gl::OptionUniformLocation::NONE,
                view_ind_uniforms: Default::default(),
                view_dep_uniforms: Default::default(),
            }
        }
    }
}