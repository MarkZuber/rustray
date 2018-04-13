extern crate rustraylib;
extern crate elapsed;
use elapsed::measure_time;

use std::sync::Arc;

use rustraylib::PosVector;
use rustraylib::ColorVector;
use rustraylib::Scene;
use rustraylib::RenderData;
use rustraylib::Renderer;
use rustraylib::Camera;

fn create_output_file_path(iter: u32) -> String {
    format!("render_{}.png", iter)
}

fn main() {
    let processor_count = 8;
    let width = 1500;
    let height = 1500;
    let ray_trace_depth = 5;

    let camera_pos = PosVector::new(50.0, 30.0, 50.0);

    let camera_look_at = PosVector::new(-0.1, 0.1, 0.0);
    let camera_up = PosVector::new(0.0, 0.0, 1.0);
    let camera_fov: f64 = 50.0;

    let background_color = ColorVector::new(0.0, 0.0, 0.0);
    let background_ambience: f64 = 0.0;

    let sphere_radius: f64 = 2.0;
    let sphere_distance_increment: f64 = 5.0;
    let num_spheres_per_axis: i32 = 10;
    let show_plane = true;
    let plane_pos = PosVector::new(0.0, 0.0, 1.0);
    let plane_d_val: f64 = 0.0;

    let render_data = RenderData {
        width,
        height,
        ray_trace_depth,
        processor_count,
        thread_per_line: true
    };

    let scene = Arc::new(Scene::new_marbles_scene(
        background_color,
        background_ambience,
        sphere_radius,
        sphere_distance_increment,
        num_spheres_per_axis,
        show_plane,
        plane_pos,
        plane_d_val,
    ));

    let mut camera = Camera::new(camera_pos, camera_look_at, camera_up, camera_fov);

    // let scene = Arc::new(Scene::new_basic_scene());
    // let mut camera = Camera::new(PosVector::new(7.5, 7.5, 2.3), PosVector::new(0.0, 0.0, 0.0), PosVector::new(0.0, 0.0, 1.0), 50.0);

    println!("Hello, world!");


    for i in 1..2 {
        let output_file_path = create_output_file_path(i);
        let (elapsed, _) = measure_time(|| {
            Renderer::render_frame(camera.clone(), render_data, scene.clone(), &output_file_path);
        });
        // camera = Camera::new(PosVector::new(camera.position.x, camera.position.y, camera.position.z - (0.1 * (i as f64))), camera_look_at, camera_up, camera_fov);
        // camera = Camera::new(PosVector::new(camera.position.x, camera.position.y, camera.position.z), camera_look_at, camera_up, camera_fov);
        println!("elapsed = {:?}ms", elapsed.millis());        
    }
}
