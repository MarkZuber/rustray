extern crate rustraylib;
extern crate elapsed;
use elapsed::measure_time;

use rustraylib::PosVector;
use rustraylib::ColorVector;
use rustraylib::Scene;
use rustraylib::RenderData;
use rustraylib::Renderer;
use rustraylib::Camera;

fn main() {
    let processor_count = 8;
    let width = 1500;
    let height = 1500;
    let ray_trace_depth = 5;

    let camera_pos = PosVector::new(30.0, 30.0, 70.0);

    let camera_look_at = PosVector::new(-0.1, 0.1, 0.0);
    let camera_up = PosVector::new(0.0, 0.0, 1.0);
    let camera_fov: f64 = 50.0;

    let background_color = ColorVector::new(0.0, 0.0, 0.0);
    let background_ambience: f64 = 0.2;

    let sphere_radius: f64 = 2.0;
    let sphere_distance_increment: f64 = 4.0;
    let num_spheres_per_axis: i32 = 10;
    let show_plane = true;
    let plane_pos = PosVector::new(1.0, 0.0, 0.0);
    let plane_d_val: f64 = 1.2;

    let render_data = RenderData {
        width,
        height,
        ray_trace_depth,
        processor_count,
        thread_per_line: true
    };

    let scene = Scene::new_marbles_scene(
        background_color,
        background_ambience,
        sphere_radius,
        sphere_distance_increment,
        num_spheres_per_axis,
        show_plane,
        plane_pos,
        plane_d_val,
    );

    println!("Hello, world!");
    let output_file_path = "foo.png";

    let camera = Camera::new(camera_pos, camera_look_at, camera_up, camera_fov);

    let (elapsed, _) = measure_time(|| {
        Renderer::render_frame(camera, render_data, scene, output_file_path);
    });
    println!("elapsed = {:?}ms", elapsed.millis());
}
