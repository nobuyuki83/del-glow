pub struct Drawer {
    pub program: Option<glow::Program>,
    uniform_loc_mvp: Option<glow::NativeUniformLocation>,
    num_vtx: usize,
    pub mode: u32,
    pub vertex_array: Option<glow::VertexArray>,
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawer {
    pub fn new() -> Self {
        Drawer {
            program: None,
            mode: 0,
            vertex_array: None,
            uniform_loc_mvp: None,
            num_vtx: 0,
        }
    }
    pub fn compile_shader(&mut self, gl: &glow::Context) {
        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        let (vertex_shader_source, fragment_shader_source) = (
            r#"
                uniform mat4 Mvp;
                in vec3 xyzIn;
                out vec3 v_color;
                void main() {
                    gl_Position = Mvp * vec4(xyzIn, 1.0);
                    gl_PointSize = 5.0;
                    v_color = vec3(1.0, 0.0, 0.0);
                }
            "#,
            r#"
                precision mediump float;
                in vec3 v_color;
                out vec4 out_color;
                void main() {
                    out_color = vec4(v_color, 1.0);
                }
            "#,
        );
        let program = crate::compile_shaders(
            gl,
            shader_version,
            vertex_shader_source,
            fragment_shader_source,
        );
        self.program = program;
    }
    pub fn set_vtx2xyz(&mut self, gl: &glow::Context, vtx2xyz: &[f32]) {
        self.num_vtx = vtx2xyz.len() / 3;
        use glow::HasContext as _;
        unsafe {
            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            let vbo = gl.create_buffer().expect("Cannot create buffer");

            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vtx2xyz),
                glow::STATIC_DRAW,
            );
            //gl.enable_vertex_attrib_array(0);

            gl.use_program(self.program);

            //
            self.uniform_loc_mvp = gl.get_uniform_location(self.program.unwrap(), "Mvp");
            let loc_xyz = gl
                .get_attrib_location(self.program.unwrap(), "xyzIn")
                .unwrap();
            dbg!(loc_xyz);
            //
            gl.vertex_attrib_pointer_f32(
                loc_xyz,
                3,
                glow::FLOAT,
                false,
                3 * std::mem::size_of::<f32>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(loc_xyz);
            //
            self.vertex_array = Some(vertex_array);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program.unwrap());
            gl.delete_vertex_array(self.vertex_array.unwrap());
        }
    }

    pub fn draw(&self, gl: &glow::Context, mvp: &[f32; 16]) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(self.program);
            gl.uniform_matrix_4_f32_slice(self.uniform_loc_mvp.as_ref(), false, mvp);
            gl.enable(glow::PROGRAM_POINT_SIZE);
            gl.bind_vertex_array(self.vertex_array);
            gl.draw_arrays(glow::POINTS, 0, self.num_vtx as i32);
        }
    }
}
