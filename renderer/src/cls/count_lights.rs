pub use crate::*;

pub const CLUSTER_DIMS_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(0) };
pub const TRANSLATION_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(1) };
pub const SCALE_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(2) };
pub const LIGHT_COUNT_LOC: gl::UniformLocation = unsafe { gl::UniformLocation::new_unchecked(3) };

pub struct CountLightsProgram {
    pub program: rendering::Program,
}

impl CountLightsProgram {
    pub fn new(gl: &gl::Gl, world: &mut World) -> Self {
        Self {
            program: rendering::Program::new(
                gl,
                vec![rendering::Shader::new(
                    gl,
                    gl::COMPUTE_SHADER,
                    EntryPoint::new(world, "cls/count_lights.comp"),
                )],
            )
        }
    }
}
