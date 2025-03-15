use glow::HasContext;

pub struct Drawer {
    vertex_array: Option<glow::NativeVertexArray>,
    pub program: Option<glow::Program>,
    uniform_loc_mvp: Option<glow::NativeUniformLocation>,
    num_elem: usize,
    num_node: usize,
}

impl Drawer {
    pub fn new() -> Self {
        Drawer {
            program: None,
            vertex_array: None,
            uniform_loc_mvp: None,
            num_elem: 3,
            num_node: 3,
        }
    }

    pub fn compile_shader(&mut self, gl: &glow::Context) {
        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        let vs_src = r#"
        uniform mat4 Mvp;
        layout (location=0) in vec3 in_position;
        layout (location=1) in vec3 in_color;
        out vec3 color;
        void main() {
            color = in_color;
            gl_Position = Mvp * vec4(in_position, 1.0);
        }
"#;
        let fs_src = r#"
                in vec3 color;
                out vec4 f_color;
                void main() {
                    f_color = vec4(color, 1.0);
                }
"#;
        unsafe {
            self.program = crate::compile_shaders(gl, shader_version, vs_src, fs_src);
            {
                let vao0 = gl.create_vertex_array().unwrap();
                self.vertex_array = Some(vao0);
                //
                let program = self.program.unwrap();
                self.uniform_loc_mvp = gl.get_uniform_location(program, "Mvp");
                //
                let attrib_loc_position = gl
                    .get_attrib_location(self.program.unwrap(), "in_position")
                    .unwrap();
                let attrib_loc_color = gl
                    .get_attrib_location(self.program.unwrap(), "in_color")
                    .unwrap();
                dbg!(attrib_loc_position, attrib_loc_color);
                gl.bind_vertex_array(self.vertex_array);
            }
        }
    }

    pub fn update_tri2node2xyz(&mut self, gl: &glow::Context, tri2node2xyz: &[f32]) {
        self.num_elem = tri2node2xyz.len() / 9;
        self.num_node = 3;
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(tri2node2xyz),
                glow::STATIC_DRAW,
            );
            let attrib_loc_position = gl
                .get_attrib_location(self.program.unwrap(), "in_position")
                .unwrap();
            gl.enable_vertex_attrib_array(attrib_loc_position);
            gl.vertex_attrib_pointer_f32(
                attrib_loc_position,
                3,
                glow::FLOAT,
                false,
                (3 * std::mem::size_of::<f32>()) as i32,
                0i32,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
        }
    }

    pub fn update_tri2node2rgb(&mut self, gl: &glow::Context, tri2color: &[f32]) {
        // assert_eq!(tri2color.len(), self.num_elem * 3);
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            //
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(tri2color),
                glow::STATIC_DRAW,
            );
            let attrib_loc_color = gl
                .get_attrib_location(self.program.unwrap(), "in_color")
                .unwrap();
            gl.enable_vertex_attrib_array(attrib_loc_color);
            gl.vertex_attrib_pointer_f32(
                attrib_loc_color,
                3,
                glow::FLOAT,
                false,
                (3 * std::mem::size_of::<f32>()) as i32,
                0i32,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw(&self, gl: &glow::Context, mat_modelview: &[f32; 16], mat_projection: &[f32; 16]) {
        let mvp = del_geo_core::mat4_col_major::mult_mat_col_major(mat_projection, mat_modelview);
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            gl.use_program(self.program);
            gl.uniform_matrix_4_f32_slice(self.uniform_loc_mvp.as_ref(), false, &mvp);
            gl.draw_arrays(glow::TRIANGLES, 0, (self.num_elem * self.num_node) as i32);
            gl.bind_vertex_array(None);
        }
    }
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}
