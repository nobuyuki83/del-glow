pub mod drawer_edge2;
pub mod drawer_elem2vtx_vtx2xyz;
pub mod drawer_elem2vtx_vtx2xyz_vtx2uv;
pub mod drawer_mesh2_at_multiple_loc2s;
pub mod drawer_tri2node2xyz_tri2node2rgb;
pub mod drawer_vtx2xyrgb;

pub fn compile_shaders(
    gl: &glow::Context,
    shader_version: &str,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> Option<glow::NativeProgram> {
    use glow::HasContext;
    unsafe {
        let program = gl.create_program().expect("Cannot create program");
        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let shaders: Vec<_> = shader_sources
            .iter()
            .map(|(shader_type, shader_source)| {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, &format!("{shader_version}\n{shader_source}"));
                gl.compile_shader(shader);
                assert!(
                    gl.get_shader_compile_status(shader),
                    "Failed to compile {shader_type}: {}",
                    gl.get_shader_info_log(shader)
                );
                gl.attach_shader(program, shader);
                shader
            })
            .collect();

        gl.link_program(program);
        assert!(
            gl.get_program_link_status(program),
            "{}",
            gl.get_program_info_log(program)
        );

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        Some(program)
    }
}
