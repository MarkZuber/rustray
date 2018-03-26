extern crate image;

use std::sync::{Arc, Mutex};


pub mod posvector;
pub mod color;
pub mod scene;

pub use scene::{Camera, PixelArray, RayTracer, RenderData, Scene, ThreadPool};
pub use posvector::PosVector;
pub use color::ColorVector;

pub struct Renderer {}

impl Renderer {
  fn handle_render_pixel(tracer: Arc<RayTracer>, pixels: Arc<Mutex<PixelArray>>, x: u32, y: u32) {
    // println!("handle_render_pixel: ({}, {})", x, y);
    // println!(".");

    let color = tracer.get_pixel_color(x, y);
    pixels.lock().unwrap().set_pixel_color(x, y, color);
  }

  pub fn render_frame(
    camera: Camera,
    render_data: RenderData,
    scene: Scene,
    output_file_path: &str,
  ) {
    println!("Scene: {:?}", scene);
    println!("RenderData: {:?}", render_data);
    println!("Camera: {:?}", camera);

    let pixel_array = Renderer::render(camera, scene, render_data);
    pixel_array.lock().unwrap().save_as_png(output_file_path);
  }

  fn render_single_threaded(camera: Camera, scene: Scene, render_data: RenderData) -> Arc<Mutex<PixelArray>> {
    let pixels = Arc::new(Mutex::new(PixelArray::new(render_data.width, render_data.height)));

    let tracer = RayTracer {
      camera,
      render_data,
      scene: scene,
    };

    for y in 0..render_data.height {
      for x in 0..render_data.width {
        // print!(".");
        let color = tracer.get_pixel_color(x, y);
        pixels.lock().unwrap().set_pixel_color(x, y, color);
      }
      // println!();
    }

    pixels
  }

  fn render_multi_threaded(camera: Camera, scene: Scene, render_data: RenderData) -> Arc<Mutex<PixelArray>> {
    let pixels = Arc::new(Mutex::new(PixelArray::new(render_data.width, render_data.height)));

    let tracer = Arc::new(RayTracer {
      camera,
      render_data: render_data.clone(),
      scene,
    });

    let pool = ThreadPool::new(render_data.processor_count as usize);

    for y in 0..render_data.height {
      for x in 0..render_data.width {
        // print!(".");

        let pixelx = x.clone();
        let pixely = y.clone();

        // this clones the reference and is done OUTSIDE of the move block within pool.execute so we have the
        // cloned reference here and can then capture it in the closure below.
        let job_tracer = tracer.clone();
        let job_pixels = pixels.clone();

        pool.execute(move || {
          Renderer::handle_render_pixel(job_tracer, job_pixels, pixelx, pixely);
        });
      }
      // println!();
    }

    pixels
  }

  fn render(camera: Camera, scene: Scene, render_data: RenderData) -> Arc<Mutex<PixelArray>> {
    if render_data.processor_count <= 1 {
      Renderer::render_single_threaded(camera, scene, render_data)
    } else {
      Renderer::render_multi_threaded(camera, scene, render_data)
    }
  }
}
