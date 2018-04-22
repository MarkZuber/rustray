extern crate elapsed;
extern crate num_cpus;
extern crate rustraylib;
use elapsed::measure_time;

use std::sync::Arc;

use rustraylib::Camera;
use rustraylib::ColorVector;
use rustraylib::PosVector;
use rustraylib::RenderData;
use rustraylib::Renderer;
use rustraylib::Scene;
use rustraylib::nffparsing;

use rustraylib::scene::{new_basic_scene, new_marbles_scene};


#[allow(dead_code)]
fn create_output_file_path(iter: u32) -> String {
    format!("render_{}.png", iter)
}

#[allow(dead_code)]
fn get_marbles_scene_and_camera() -> (Arc<Scene>, Camera) {
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

    let scene = Arc::new(new_marbles_scene(
        background_color,
        background_ambience,
        sphere_radius,
        sphere_distance_increment,
        num_spheres_per_axis,
        show_plane,
        plane_pos,
        plane_d_val,
    ));

    (
        scene,
        Camera::new(camera_pos, camera_look_at, camera_up, camera_fov),
    )
}

#[allow(dead_code)]
fn get_simple_scene_and_camera() -> (Arc<Scene>, Camera) {
    (
        Arc::new(new_basic_scene()),
        Camera::new(
            PosVector::new(7.5, 7.5, 2.3),
            PosVector::new(0.0, 0.0, 0.0),
            PosVector::new(0.0, 0.0, 1.0),
            50.0,
        ),
    )
}

fn render_nff(filename_no_ext: &str) {
    println!("preparing to render nff: {}", filename_no_ext);

    let nff_path = format!("nff/{}.nff", filename_no_ext);
    let parse_result =
        nffparsing::parse_nff_file(&nff_path, num_cpus::get() as u32, 5);

    let (elapsed, _) = measure_time(|| {
        let scene_path = format!("output/render_{}.png", filename_no_ext);
        Renderer::render_frame(
            parse_result.camera,
            parse_result.render_data,
            Arc::new(parse_result.scene),
            &scene_path,
        );
    });
    println!("elapsed = {:?}ms", elapsed.millis());
}

fn main() {
    render_nff("balls1");
    // render_nff("balls2");
    // render_nff("balls3");
    // render_nff("mountain");
    // // render_nff("champagne");   // need to support polygonal patches for this one
    // // render_nff("spirale");     // need to support polygonal patches for this one
    // // render_nff("spirale2");    // need to support polygonal patches for this one
    // render_nff("teapot");
    // // render_nff("teapot2");     // need to support polygonal patches for this one
    // render_nff("temple");         // need to support polygonal patches for this one
    // render_nff("jacks1");
    // render_nff("jacks2");
    // render_nff("jacks3");
    // render_nff("jacks4");
}

#[allow(dead_code)]
fn main_and_stuff() {
    let num_threads = num_cpus::get() as u32;
    let width = 1500;
    let height = 1500;
    let ray_trace_depth = 5;

    println!("Perparing to render scene 1");
    let (elapsed1, _) = measure_time(|| {
        let (scene1, cam1) = get_marbles_scene_and_camera();
        let scene1_path = create_output_file_path(1);
        Renderer::render_frame(
            cam1.clone(),
            RenderData::new(
                width,
                height,
                ray_trace_depth,
                num_threads,
                true,
            ),
            scene1.clone(),
            &scene1_path,
        );
    });
    println!("elapsed = {:?}ms", elapsed1.millis());

    println!("Preparing to render scene 2");
    let (elapsed2, _) = measure_time(|| {
        let (scene2, cam2) = get_simple_scene_and_camera();
        let scene2_path = create_output_file_path(2);
        Renderer::render_frame(
            cam2.clone(),
            RenderData::new(
                width,
                height,
                ray_trace_depth,
                num_threads,
                true,
            ),
            scene2.clone(),
            &scene2_path,
        );
    });
    println!("elapsed = {:?}ms", elapsed2.millis());

    // for i in 1..2 {
    //     let output_file_path = create_output_file_path(i);
    //     let (elapsed, _) = measure_time(|| {
    //         Renderer::render_frame(camera.clone(), render_data, scene.clone(), &output_file_path);
    //     });
    //     // camera = Camera::new(PosVector::new(camera.position.x, camera.position.y, camera.position.z - (0.1 * (i as f64))), camera_look_at, camera_up, camera_fov);
    //     // camera = Camera::new(PosVector::new(camera.position.x, camera.position.y, camera.position.z), camera_look_at, camera_up, camera_fov);
    //     println!("elapsed = {:?}ms", elapsed.millis());
    // }
}
