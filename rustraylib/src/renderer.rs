use image;
use std::sync::{Arc, Mutex};

use camera::Camera;
use color::ColorVector;
use scene::Scene;
use threading::ThreadPool;
use tracer::RayTracer;

#[derive(Debug)]
pub struct PixelArray {
  imgbuf: image::RgbImage,
}

impl PixelArray {
  pub fn new(width: u32, height: u32) -> PixelArray {
    PixelArray {
      imgbuf: image::RgbImage::new(width, height),
    }
  }

  pub fn get_width(&self) -> u32 {
    self.imgbuf.width()
  }

  pub fn get_height(&self) -> u32 {
    self.imgbuf.height()
  }

  fn f64_to_rgb(val: f64) -> u8 {
    (val * 255.0) as u8
  }

  fn clamp_to_pixel(&self, color: ColorVector) -> image::Rgb<u8> {
    let double_clamped = color.clamp();

    image::Rgb([
      PixelArray::f64_to_rgb(double_clamped.r),
      PixelArray::f64_to_rgb(double_clamped.g),
      PixelArray::f64_to_rgb(double_clamped.b),
    ])
  }

  pub fn set_pixel_color(&mut self, x: u32, y: u32, color: ColorVector) {
    let pixel = self.clamp_to_pixel(color);
    self.imgbuf.put_pixel(x, y, pixel);
  }

  pub fn save_as_png(&self, output_file_path: &str) {
    self.imgbuf.save(output_file_path).unwrap();
  }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderData {
  pub width: u32,
  pub height: u32,
  pub ray_trace_depth: u32,
  pub num_threads: u32,
  pub thread_per_line: bool,
  pub render_diffuse: bool,
  pub render_reflection: bool,
  pub render_refraction: bool,
  pub render_shadow: bool,
  pub render_highlights: bool
}

impl RenderData {
  pub fn new(width: u32, height: u32,
                ray_trace_depth: u32,
                num_threads: u32,
                thread_per_line: bool) -> RenderData {
    RenderData{width, height, ray_trace_depth, num_threads, thread_per_line, render_diffuse: true, render_reflection: true, render_refraction: true, render_shadow: true, render_highlights: true }
  }
}

pub struct Renderer {}

impl Renderer {
  fn handle_render_pixel(tracer: Arc<RayTracer>, pixels: Arc<Mutex<PixelArray>>, x: u32, y: u32) {
    let color = tracer.get_pixel_color(x, y);
    pixels.lock().unwrap().set_pixel_color(x, y, color);
  }

  fn handle_render_line(tracer: Arc<RayTracer>, pixels: Arc<Mutex<PixelArray>>, y: u32) {
    let mut line_colors: Vec<ColorVector> = Vec::new();
    for x in 0..tracer.render_data.width {
      let color = tracer.get_pixel_color(x, y);
      line_colors.push(color);
    }

    {
      let mut pix = pixels.lock().unwrap();
      for x in 0..tracer.render_data.width {
        pix.set_pixel_color(x, y, line_colors[x as usize]);
      }
    }
  }

  pub fn render_frame(
    camera: Camera,
    render_data: RenderData,
    scene: Arc<Scene>,
    output_file_path: &str,
  ) {
    // println!("Scene: {:?}", scene);
    // println!();
    println!("RenderData: {:?}", render_data);
    println!();
    println!("Camera: {:?}", camera);
    println!();

    let pixel_array = Renderer::render(camera, scene, render_data);
    pixel_array.lock().unwrap().save_as_png(output_file_path);
  }

  fn render_single_threaded(
    camera: Camera,
    scene: Arc<Scene>,
    render_data: RenderData,
  ) -> Arc<Mutex<PixelArray>> {
    let pixels = Arc::new(Mutex::new(PixelArray::new(
      render_data.width,
      render_data.height,
    )));

    let tracer = RayTracer::new(camera, render_data, scene);

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

  fn render_multi_threaded(
    camera: Camera,
    scene: Arc<Scene>,
    render_data: RenderData,
  ) -> Arc<Mutex<PixelArray>> {
    let pixels = Arc::new(Mutex::new(PixelArray::new(
      render_data.width,
      render_data.height,
    )));

    let tracer = Arc::new(RayTracer::new(camera, render_data.clone(), scene));

    let pool = ThreadPool::new(render_data.num_threads as usize);

    if render_data.thread_per_line {
      for y in 0..render_data.height {
        let pixely = y.clone();

        // this clones the reference and is done OUTSIDE of the move block within pool.execute so we have the
        // cloned reference here and can then capture it in the closure below.
        let job_tracer = tracer.clone();
        let job_pixels = pixels.clone();

        pool.execute(move || {
          Renderer::handle_render_line(job_tracer, job_pixels, pixely);
        });
      }
    } else {
      for y in 0..render_data.height {
        for x in 0..render_data.width {
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
      }
    }

    pixels
  }

  fn render(camera: Camera, scene: Arc<Scene>, render_data: RenderData) -> Arc<Mutex<PixelArray>> {
    if render_data.num_threads <= 1 {
      Renderer::render_single_threaded(camera, scene, render_data)
    } else {
      Renderer::render_multi_threaded(camera, scene, render_data)
    }
  }
}
