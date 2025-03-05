pub struct Drawer {
    pub program: Option<glow::Program>,
    pub mode: u32,
    pub vertex_array: Option<glow::VertexArray>,
}

impl Drawer {
    pub fn compile_shader(&mut self, gl: &glow::Context) {
        use glow::HasContext as _;

        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        let (vertex_shader_source, fragment_shader_source) = (
            r#"
                in vec2 xyzIn;
                in vec3 rgbIn;
                out vec3 v_color;
                void main() {
                    v_color = rgbIn;
                    gl_Position = vec4(xyzIn, 0.0, 1.0);
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
    pub fn new(&mut self, gl: &glow::Context, vtx2xyrgb: &[f32]) {
        use glow::HasContext as _;
        unsafe {
            let vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vtx2xyrgb),
                glow::STATIC_DRAW,
            );

            gl.use_program(self.program);

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vertex_array));
            //
            let loc_xyz = gl
                .get_attrib_location(self.program.unwrap(), "xyzIn")
                .unwrap();
            let loc_rgb = gl
                .get_attrib_location(self.program.unwrap(), "rgbIn")
                .unwrap();
            dbg!(loc_xyz, loc_rgb);
            gl.vertex_attrib_pointer_f32(
                loc_xyz,
                2,
                glow::FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                0,
            );
            gl.vertex_attrib_pointer_f32(
                loc_rgb,
                3,
                glow::FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                2 * std::mem::size_of::<f32>() as i32,
            );
            gl.enable_vertex_attrib_array(loc_xyz);
            gl.enable_vertex_attrib_array(loc_rgb);
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

    pub fn paint(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(self.program);
            gl.bind_vertex_array(self.vertex_array);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}
