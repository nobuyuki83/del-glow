//! draw mesh position. The RGB color is defined par index

use glow::HasContext;

struct ElementBufferObject {
    mode: u32,
    elem_size: usize,
    ebo: Option<glow::NativeBuffer>,
    color: Option<[f32; 3]>,
}

pub struct Drawer {
    program: Option<glow::NativeProgram>,
    pub ndim: usize,
    num_point: usize,
    vertex_array: Option<glow::NativeVertexArray>,
    // uniform variables
    loc_texture: Option<glow::NativeUniformLocation>,
    loc_color: Option<glow::NativeUniformLocation>,
    loc_is_texture: Option<glow::NativeUniformLocation>,
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
            loc_texture: None,    // -1 is the failure flag
            loc_color: None,      // -1 is the failure flag
            loc_is_texture: None, // -1 is the failure flag
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

        const VS_SRC: &str = r#"
uniform mat4 matMV;
uniform mat4 matPrj;

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texIn;
out vec2 texPrj;

void main() {
    gl_Position = matPrj * matMV * vec4(position, 1.0);
    texPrj = texIn;
    // gl_Position = vec4(position, 1.0);
}
"#;

        const FS_SRC: &str = r#"
uniform sampler2D myTextureSampler;
uniform vec3 color;
uniform bool is_texture;

in vec2 texPrj;
out vec4 FragColor;

void main() {
    if( is_texture ){
        FragColor = texture(myTextureSampler,texPrj);
    }
    else {
        FragColor = vec4(color, 1.0);
    }
}
"#;
        unsafe {
            self.program = crate::compile_shaders(gl, shader_version, VS_SRC, FS_SRC);
            let program = self.program.unwrap();
            self.loc_mat_modelview = gl.get_uniform_location(program, "matMV");
            self.loc_mat_projection = gl.get_uniform_location(program, "matPrj");
            self.loc_texture = gl.get_uniform_location(program, "myTextureSampler");
            self.loc_color = gl.get_uniform_location(program, "color");
            self.loc_is_texture = gl.get_uniform_location(program, "is_texture");
            let vao0 = gl.create_vertex_array().unwrap();
            self.vertex_array = Some(vao0);
        }
    }

    pub fn add_elem2vtx<T>(
        &mut self,
        gl: &glow::Context,
        mode: u32,
        elem2vtx: &[T],
        color: Option<[f32; 3]>,
    ) where
        T: 'static + Copy + num_traits::AsPrimitive<u32>,
    {
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            let ebo0 = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo0));
            let elem_vtx0: Vec<u32> = elem2vtx.iter().map(|i| (*i).as_()).collect();
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&elem_vtx0),
                glow::STATIC_DRAW,
            );
            self.ebos.push(ElementBufferObject {
                mode,
                elem_size: elem_vtx0.len(),
                ebo: Some(ebo0),
                color,
            });
            gl.bind_vertex_array(None);
        }
    }

    pub fn update_vtx2xyz(&mut self, gl: &glow::Context, vtx2xyz: &[f32], ndim: usize) {
        self.ndim = ndim;
        self.num_point = vtx2xyz.len() / self.ndim;
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            //
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
                0i32,
            );
            gl.bind_vertex_array(None);
        }
    }

    pub fn set_vtx2uv(&mut self, gl: &glow::Context, vtx2tex: &[f32]) {
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vtx2tex),
                glow::STATIC_DRAW,
            );
            let pos_attrib = gl
                .get_attrib_location(self.program.unwrap(), "texIn")
                .unwrap();
            gl.enable_vertex_attrib_array(pos_attrib);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                (2 * std::mem::size_of::<f32>()) as i32,
                0,
            ); // gl24
            gl.bind_vertex_array(None);
        }
    }

    pub fn draw(&self, gl: &glow::Context, mat_modelview: &[f32], mat_projection: &[f32]) {
        let mp0 = mat_projection;
        let mp1: [f32; 16] = [
            // mp1 = [z flip] * mp0
            mp0[0], mp0[1], -mp0[2], mp0[3], mp0[4], mp0[5], -mp0[6], mp0[7], mp0[8], mp0[9],
            -mp0[10], mp0[11], mp0[12], mp0[13], -mp0[14], mp0[15],
        ];
        unsafe {
            gl.bind_vertex_array(self.vertex_array);
            gl.use_program(self.program);
            for ebo in &self.ebos {
                match ebo.color {
                    Some(color) => {
                        gl.uniform_1_i32(self.loc_is_texture.as_ref(), 0);
                        gl.uniform_3_f32(self.loc_color.as_ref(), color[0], color[1], color[2]);
                    }
                    _ => {
                        gl.uniform_1_i32(self.loc_is_texture.as_ref(), 1);
                    }
                }
                gl.uniform_matrix_4_f32_slice(
                    self.loc_mat_modelview.as_ref(),
                    false,
                    mat_modelview,
                );
                gl.uniform_matrix_4_f32_slice(self.loc_mat_projection.as_ref(), false, &mp1);
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
            gl.bind_vertex_array(self.vertex_array);
            gl.use_program(self.program);
            gl.uniform_matrix_4_f32_slice(self.loc_mat_modelview.as_ref(), false, mat_modelview);
            gl.uniform_matrix_4_f32_slice(self.loc_mat_projection.as_ref(), false, &mp1);
            gl.draw_arrays(glow::POINTS, 0, (self.num_point) as i32);
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
}
impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}
