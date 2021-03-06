use crate::*;

pub struct Renderer {
    pub program: rendering::Program,
}

glsl_defines! {
    fixed_header {
        bindings: {
            CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING = 0;
            ACTIVE_CLUSTER_CLUSTER_INDICES_BUFFER_BINDING = 1;
            ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING = 2;
            ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING = 3;
            LIGHT_INDICES_BUFFER_BINDING = 4;
            CLUSTER_SPACE_BUFFER_BINDING = 5;
        },
        uniforms: {
            CLU_CAM_TO_REN_CLP_LOC = 0;
            VISIBLE_ONLY_LOC = 1;
            VISUALISATION_LOC = 2;
            PASS_LOC = 3;
        },
    }
}

pub struct Parameters<'a> {
    pub cluster_resources_index: ClusterResourcesIndex,
    pub clu_cam_to_ren_clp: &'a Matrix4<f64>,
    pub visualisation: configuration::ClusterVisualisation,
    pub visible_only: bool,
}

impl Context<'_> {
    pub fn render_debug_clusters(&mut self, params: &Parameters) {
        unsafe {
            let Self {
                ref gl,
                ref resources,
                ref cluster_resources_pool,
                ref mut cluster_renderer,
                ..
            } = *self;

            cluster_renderer.program.update(&mut rendering_context!(self));

            let cluster_resources = &cluster_resources_pool[params.cluster_resources_index];

            if let ProgramName::Linked(program_name) = cluster_renderer.program.name {
                gl.use_program(program_name);
                gl.bind_vertex_array(resources.cluster_vao);

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    CLUSTER_FRAGMENT_COUNTS_BUFFER_BINDING,
                    cluster_resources.cluster_fragment_counts_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    ACTIVE_CLUSTER_CLUSTER_INDICES_BUFFER_BINDING,
                    cluster_resources.active_cluster_cluster_indices_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    ACTIVE_CLUSTER_LIGHT_COUNTS_BUFFER_BINDING,
                    cluster_resources.active_cluster_light_counts_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    ACTIVE_CLUSTER_LIGHT_OFFSETS_BUFFER_BINDING,
                    cluster_resources.active_cluster_light_offsets_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::SHADER_STORAGE_BUFFER,
                    LIGHT_INDICES_BUFFER_BINDING,
                    cluster_resources.light_indices_buffer.name(),
                );

                gl.bind_buffer_base(
                    gl::UNIFORM_BUFFER,
                    CLUSTER_SPACE_BUFFER_BINDING,
                    cluster_resources.cluster_space_buffer.name(),
                );

                gl.bind_buffer(gl::DRAW_INDIRECT_BUFFER, cluster_resources.draw_commands_buffer.name());

                let clu_cam_to_ren_clp = params.clu_cam_to_ren_clp.cast::<f32>().unwrap();
                gl.uniform_matrix4f(
                    CLU_CAM_TO_REN_CLP_LOC,
                    gl::MajorAxis::Column,
                    clu_cam_to_ren_clp.as_ref(),
                );

                gl.uniform_1ui(
                    VISIBLE_ONLY_LOC,
                    params.visible_only as u32,
                );

                use configuration::ClusterVisualisation;
                gl.uniform_1ui(
                    VISUALISATION_LOC,
                    match params.visualisation {
                        ClusterVisualisation::ClusterIndices => 1,
                        ClusterVisualisation::LightCountHeatmap => 8,
                        ClusterVisualisation::LightCountVolumetric => 9,
                        ClusterVisualisation::FragmentCountHeatmap => 16,
                        ClusterVisualisation::FragmentCountVolumetric => 17,
                    },
                );

                // Draw opaque regardless.
                gl.enable(gl::DEPTH_TEST);
                gl.depth_func(gl::GREATER);
                gl.depth_mask(gl::TRUE);
                gl.uniform_1ui(PASS_LOC, 0);
                if params.visible_only {
                    gl.draw_elements_indirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0);
                } else {
                    gl.draw_elements_instanced_base_vertex (
                        gl::TRIANGLES,
                        36,
                        gl::UNSIGNED_INT,
                        0,
                        cluster_resources.computed.dimensions.product(),
                        0,
                    );
                }

                if let ClusterVisualisation::LightCountVolumetric | ClusterVisualisation::FragmentCountVolumetric =
                    params.visualisation
                {
                    gl.depth_mask(gl::FALSE);
                    gl.enable(gl::BLEND);
                    gl.blend_func(gl::SRC_ALPHA, gl::ONE);
                    gl.uniform_1ui(PASS_LOC, 1);
                    if params.visible_only {
                        gl.draw_elements_indirect(gl::TRIANGLES, gl::UNSIGNED_INT, 0);
                    } else {
                        gl.draw_elements_instanced_base_vertex(
                            gl::TRIANGLES,
                            36,
                            gl::UNSIGNED_INT,
                            0,
                            cluster_resources.computed.dimensions.product(),
                            0,
                        );
                    }
                }

                gl.depth_mask(gl::TRUE);
                gl.disable(gl::BLEND);

                gl.unbind_vertex_array();
                gl.unuse_program();
            }
        }
    }
}

impl Renderer {
    pub fn new(context: &mut RenderingContext) -> Self {
        Renderer {
            program: vs_fs_program(
                context,
                "cls/cluster_renderer.vert",
                "cls/cluster_renderer.frag",
                fixed_header(),
            ),
        }
    }
}
