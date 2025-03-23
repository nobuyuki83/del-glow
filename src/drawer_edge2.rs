pub struct Drawer {
    pub drawer_quad: crate::drawer_elem2vtx_vtx2xyz::Drawer,
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawer {
    pub fn new() -> Self {
        let drawer_quad = crate::drawer_elem2vtx_vtx2xyz::Drawer::new();
        Self { drawer_quad }
    }

    pub fn compile_shader(&mut self, gl: &glow::Context) {
        self.drawer_quad.compile_shader(gl);
        let vtx2xy = vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let tri2vtx = vec![0, 1, 2, 0, 2, 3];
        self.drawer_quad
            .add_elem2vtx(gl, glow::TRIANGLES, &tri2vtx, [0., 0., 0.]);
        self.drawer_quad.set_vtx2xyz(gl, &vtx2xy, 2);
    }

    pub fn set_color(&mut self, rgb: &[f32; 3]) {
        self.drawer_quad.set_color(0, rgb);
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.drawer_quad.destroy(gl);
    }

    pub fn draw_edge2(
        &self,
        gl: &glow::Context,
        mvp: &[f32; 16],
        ps: &[f32; 2],
        pe: &[f32; 2],
        width_ndc: f32,
    ) {
        use del_geo_core::vec2::Vec2;
        let v_se = pe.sub(ps);
        let u_se = v_se.normalize().rot90().scale(width_ndc);
        let r = del_geo_core::mat2_col_major::from_columns(&v_se, &u_se);
        let left_bottom_corner_world = ps.sub(&u_se.scale(0.5));
        let m = del_geo_core::mat3_col_major::from_affine_linear_and_translation(
            &r,
            &left_bottom_corner_world,
        );
        let m = del_geo_core::mat4_col_major::from_mat3_col_major_adding_z(&m);
        self.drawer_quad.draw(gl, &m, mvp);
    }

    pub fn draw_polyloop2(
        &self,
        gl: &glow::Context,
        mvp: &[f32; 16],
        vtx2xy: &[f32],
        width_ndc: f32,
    ) {
        let num_vtx = vtx2xy.len() / 2;
        for i0_vtx in 0..num_vtx {
            let i1_vtx = (i0_vtx + 1) % num_vtx;
            let p0 = arrayref::array_ref![vtx2xy, i0_vtx * 2, 2];
            let p1 = arrayref::array_ref![vtx2xy, i1_vtx * 2, 2];
            self.draw_edge2(gl, mvp, p0, p1, width_ndc);
        }
    }
}
