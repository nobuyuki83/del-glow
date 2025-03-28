//! draw mesh position. The RGB color is defined par index

use glow::HasContext;

struct ElementBufferObject {
    mode: u32,
    elem_size: usize,
    ebo: Option<glow::NativeBuffer>,
    color: [f32; 3],
}

pub struct Drawer {
    program: Option<glow::NativeProgram>,
    pub ndim: usize,
    num_point: usize,
    vertex_array: Option<glow::NativeVertexArray>,
    // uniform variables
    loc_color: Option<glow::NativeUniformLocation>,
    loc_mat_modelview: Option<glow::NativeUniformLocation>,
    loc_mat_projection: Option<glow::NativeUniformLocation>,
    // elemenb buffer object
    ebos: Vec<ElementBufferObject>,
}

impl Drawer {
    pub fn new() -> Self {
        Drawer {
            program: None,
            ndim: 0,
            num_point: 0,
            vertex_array: None,
            loc_color: None, // -1 is the failure flag
            loc_mat_modelview: None,
            loc_mat_projection: None,
            ebos: Vec::<ElementBufferObject>::new(),
        }
    }
    pub fn compile_shader(&mut self, gl: &glow::Context) {
        let shader_version = if cfg!(target_arch = "wasm32") {
            "#version 300 es"
        } else {
            "#version 330"
        };

        let vs_src = r#"
uniform mat4 matMV;
uniform mat4 matPrj;
in vec3 position;

void main() {
    gl_Position = matPrj * matMV * vec4(position, 1.0);
    // gl_Position = vec4(position, 1.0);
}
"#;

        let fs_src = r#"
uniform vec3 color;
out vec4 FragColor;

void main() {
    FragColor = vec4(color, 1.0);
}
"#;

        unsafe {
            self.program = crate::compile_shaders(gl, shader_version, vs_src, fs_src);
            self.loc_mat_modelview = gl.get_uniform_location(self.program.unwrap(), "matMV");
            self.loc_mat_projection = gl.get_uniform_location(self.program.unwrap(), "matPrj");
            self.loc_color = gl.get_uniform_location(self.program.unwrap(), "color");
            gl.use_program(None);
            let vao0 = gl.create_vertex_array().unwrap();
            self.vertex_array = Some(vao0);
        }
    }

    pub fn add_elem2vtx<T>(
        &mut self,
        gl: &glow::Context,
        mode: u32,
        elem2vtx: &[T],
        color: [f32; 3],
    ) where
        T: 'static + Copy + num_traits::AsPrimitive<u32>,
    {
        let elem2vtx0: Vec<u32> = elem2vtx.iter().map(|i| (*i).as_()).collect();
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            let ebo0 = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo0));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&elem2vtx0),
                glow::STATIC_DRAW,
            );
            self.ebos.push(ElementBufferObject {
                mode,
                elem_size: elem2vtx0.len(),
                ebo: Some(ebo0),
                color,
            });
            gl.bind_vertex_array(None);
        }
    }

    pub fn set_vtx2xyz(&mut self, gl: &glow::Context, vtx2xyz: &[f32], ndim: usize) {
        self.ndim = ndim;
        self.num_point = vtx2xyz.len() / self.ndim;
        unsafe {
            gl.use_program(None);
            gl.bind_vertex_array(self.vertex_array);
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vtx2xyz),
                glow::STATIC_DRAW,
            );
            let pos_attrib = gl
                .get_attrib_location(self.program.unwrap(), "position")
                .unwrap();
            gl.enable_vertex_attrib_array(pos_attrib);
            gl.vertex_attrib_pointer_f32(
                pos_attrib,
                self.ndim as i32,
                glow::FLOAT,
                false,
                (self.ndim * std::mem::size_of::<f32>()) as i32,
                0,
            );
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw(&self, gl: &glow::Context, mat_modelview: &[f32; 16], mat_projection: &[f32; 16]) {
        let mp1 = mat_projection;
        unsafe {
            // gl.clear_color(1.0, 1.0, 1.0, 1.0);
            // gl.clear(glow::COLOR_BUFFER_BIT);
            gl.use_program(self.program);
            gl.bind_vertex_array(self.vertex_array);
            for ebo in &self.ebos {
                gl.enable(glow::DEPTH_TEST);
                gl.uniform_3_f32(
                    self.loc_color.as_ref(),
                    ebo.color[0],
                    ebo.color[1],
                    ebo.color[2],
                );
                gl.uniform_matrix_4_f32_slice(
                    self.loc_mat_modelview.as_ref(),
                    false,
                    mat_modelview,
                );
                gl.uniform_matrix_4_f32_slice(self.loc_mat_projection.as_ref(), false, mp1);
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, ebo.ebo);
                gl.draw_elements(ebo.mode, ebo.elem_size as i32, glow::UNSIGNED_INT, 0);
            }
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw_points(&self, gl: &glow::Context, mat_modelview: &[f32], mat_projection: &[f32]) {
        let mp0 = mat_projection;
        let mp1: [f32; 16] = [
            // mp1 = [z flip] * mp0
            mp0[0], mp0[1], -mp0[2], mp0[3], mp0[4], mp0[5], -mp0[6], mp0[7], mp0[8], mp0[9],
            -mp0[10], mp0[11], mp0[12], mp0[13], -mp0[14], mp0[15],
        ];
        unsafe {
            gl.use_program(self.program);
            gl.bind_vertex_array(self.vertex_array);
            gl.uniform_3_f32(self.loc_color.as_ref(), 0., 0., 0.);
            gl.uniform_matrix_4_f32_slice(self.loc_mat_modelview.as_ref(), false, mat_modelview);
            gl.uniform_matrix_4_f32_slice(self.loc_mat_projection.as_ref(), false, &mp1);
            gl.draw_arrays(glow::POINTS, 0, self.num_point as i32);
            gl.bind_vertex_array(None);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.delete_program(self.program.unwrap());
            gl.delete_vertex_array(self.vertex_array.unwrap());
        }
    }

    pub fn set_color(&mut self, i_ebo: usize, rgb: &[f32; 3]) {
        self.ebos[i_ebo].color = *rgb;
    }
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}
