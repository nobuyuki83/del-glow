#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use eframe::{egui, egui_glow, glow};

use egui::mutex::Mutex;
use glow::HasContext;
use std::sync::Arc;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 600.0]),
        multisampling: 4,
        depth_buffer: 32,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Custom 3D painting in eframe using glow",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    drawer_edge: Arc<Mutex<del_glow::drawer_elem2vtx_vtx2xyz::Drawer>>,
    drawer_tri: Arc<Mutex<del_glow::drawer_tri2node2xyz_tri2node2rgb::Drawer>>,
    // mat_modelview: [f32;16],
    mat_projection: [f32; 16],
    trackball: del_geo_core::view_rotation::Trackball,
    tri2vtx: Vec<u32>,
    vtx2xyz: Vec<f32>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");
        let (tri2vtx, vtx2xyz) = {
            let mut obj = del_msh_core::io_obj::WavefrontObj::<u32, f32>::new();
            obj.load("examples/asset/spot_triangulated.obj").unwrap();
            (obj.idx2vtx_xyz, obj.vtx2xyz)
        };
        let num_tri = tri2vtx.len() / 3;
        let tri2node2xyz =
            del_msh_core::unindex::unidex_vertex_attribute_for_triangle_mesh(&tri2vtx, &vtx2xyz, 3);
        assert_eq!(tri2node2xyz.len(), num_tri * 9);
        let tri2node2rgb = vec![0.9; num_tri * 9];
        let drawer_edge = {
            let mut drawer_mesh = del_glow::drawer_elem2vtx_vtx2xyz::Drawer::new();
            drawer_mesh.compile_shader(&gl);
            let edge2vtx = del_msh_core::edge2vtx::from_triangle_mesh(&tri2vtx, vtx2xyz.len() / 3);
            drawer_mesh.set_vtx2xyz(&gl, &vtx2xyz, 3);
            drawer_mesh.add_elem2vtx(&gl, glow::LINES, &edge2vtx, [0.0, 0.0, 0.0]);
            // drawer_mesh.add_element(&gl, glow::TRIANGLES, &tri2vtx, [0.8, 0.8, 0.9]);
            drawer_mesh
        };
        let drawer_tri = {
            let mut drawer_tri = del_glow::drawer_tri2node2xyz_tri2node2rgb::Drawer::new();
            drawer_tri.compile_shader(&gl);
            drawer_tri.update_tri2node2xyz(&gl, &tri2node2xyz);
            drawer_tri.update_tri2node2rgb(&gl, &tri2node2rgb);
            drawer_tri
        };
        Self {
            drawer_edge: Arc::new(Mutex::new(drawer_edge)),
            drawer_tri: Arc::new(Mutex::new(drawer_tri)),
            trackball: del_geo_core::view_rotation::Trackball::default(),
            mat_projection: del_geo_core::mat4_col_major::from_identity(),
            tri2vtx,
            vtx2xyz,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.label("Drag to rotate!");
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.drawer_edge.lock().destroy(gl);
        }
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let mat_modelview = self.trackball.mat4_col_major();
        let mat_projection = self.mat_projection;
        let transform_world2ndc =
            del_geo_core::mat4_col_major::mult_mat_col_major(&mat_projection, &mat_modelview);
        let transform_ndc2world =
            del_geo_core::mat4_col_major::try_inverse(&transform_world2ndc).unwrap();
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(500.0), egui::Sense::drag());
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let pos = pos - rect.left_top();
                let ndc_x = 2. * pos.x / rect.width() - 1.;
                let ndc_y = 1. - 2. * pos.y / rect.height();
                let world_stt = del_geo_core::mat4_col_major::transform_homogeneous(
                    &transform_ndc2world,
                    &[ndc_x, ndc_y, 1.],
                )
                .unwrap();
                let world_end = del_geo_core::mat4_col_major::transform_homogeneous(
                    &transform_ndc2world,
                    &[ndc_x, ndc_y, -1.],
                )
                .unwrap();
                let ray_org = world_stt;
                let ray_dir = del_geo_core::vec3::sub(&world_end, &world_stt);
                let res = del_msh_core::trimesh3_search_bruteforce::first_intersection_ray(
                    &ray_org,
                    &ray_dir,
                    &self.tri2vtx,
                    &self.vtx2xyz,
                );
                if let Some((depth, i_tri)) = res {
                    let pos = del_geo_core::vec3::axpy(depth, &ray_dir, &ray_org);
                    Some((i_tri as usize, pos))
                } else {
                    None
                }
            } else {
                None
            };
        }
        let z_flip = del_geo_core::mat4_col_major::from_diagonal(1., 1., -1., 1.);
        let mat_projection_for_opengl =
            del_geo_core::mat4_col_major::mult_mat_col_major(&z_flip, &mat_projection);
        // del_geo_core::view_rotation::Trackball::new();
        let drawer_edge = self.drawer_edge.clone();
        let drawer_tri = self.drawer_tri.clone();
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                let gl = painter.gl();
                unsafe {
                    gl.clear(glow::DEPTH_BUFFER_BIT);
                    gl.enable(glow::DEPTH_TEST);
                }
                drawer_edge
                    .lock()
                    .draw(painter.gl(), &mat_modelview, &mat_projection_for_opengl);
                drawer_tri
                    .lock()
                    .draw(painter.gl(), &mat_modelview, &mat_projection_for_opengl);
            })),
        };
        ui.painter().add(callback);
        //
        let xy = response.drag_motion();
        let dx = 2.0 * xy.x / rect.width() as f32;
        let dy = -2.0 * xy.y / rect.height() as f32;
        self.trackball.camera_rotation(dx as f64, dy as f64);
    }
}
