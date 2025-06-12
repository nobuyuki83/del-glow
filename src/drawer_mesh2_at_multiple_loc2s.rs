pub struct Drawer {
    pub drawer_quad: crate::drawer_elem2vtx_vtx2xyz::Drawer,
    pub vtx2xy: Vec<f32>,
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawer {
    pub fn new() -> Self {
        let drawer_quad = crate::drawer_elem2vtx_vtx2xyz::Drawer::new();
        Self {
            drawer_quad,
            vtx2xy: vec![],
        }
    }

    pub fn compile_shader(&mut self, gl: &glow::Context) {
        self.drawer_quad.compile_shader(gl);
    }

    pub fn add_mesh2(&mut self, gl: &glow::Context, tri2vtx: &[usize], vtx2xy: &[f32]) {
        self.drawer_quad
            .add_elem2vtx(gl, glow::TRIANGLES, &tri2vtx, [0., 0., 0.]);
        self.drawer_quad.set_vtx2xyz(gl, &vtx2xy, 2);
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.drawer_quad.destroy(gl);
    }

    pub fn draw(&self, gl: &glow::Context, mvp: &[f32; 16]) {
        let m = del_geo_core::mat4_col_major::from_identity();
        self.drawer_quad.draw(gl, &m, mvp);
    }
}
