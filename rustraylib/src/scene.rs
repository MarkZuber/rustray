use std::fmt;
use image;

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use posvector::PosVector;
use color::ColorVector;

#[derive(Debug)]
pub struct Camera {
  position: PosVector,
  look_at: PosVector,
  up: PosVector,
  fov: f64,

  a1: PosVector,
  a2: PosVector,
  a3: PosVector,
  dval: f64,
}

impl Camera {
  pub fn new(position: PosVector, look_at: PosVector, up: PosVector, fov: f64) -> Camera {
    let a3 = look_at.subtract(position);
    let a1 = a3.cross(up);
    let a2 = a1.cross(a3);
    let view_angle_radians = fov * 0.017453239;
    let dval: f64 = (view_angle_radians / 2.0).cos() / (view_angle_radians / 2.0).sin();

    Camera {
      position,
      look_at,
      up,
      fov,
      a1: a1.normalize(),
      a2: a2.normalize(),
      a3: a3.normalize(),
      dval,
    }
  }

  pub fn get_ray(&self, vx: f64, vy: f64) -> Ray {
    let center = self.a3.multiply_by_scalar(self.dval);
    let dir = center
      .add(self.a1.multiply_by_scalar(vx))
      .add(self.a2.multiply_by_scalar(vy));

    Ray {
      position: self.position,
      direction: dir.normalize(),
    }
  }
}

#[derive(Debug)]
pub struct Background {
  color: ColorVector,
  ambience: f64,
}

impl Background {
  pub fn new(color: ColorVector, ambience: f64) -> Background {
    Background { color, ambience }
  }
}

trait Material: fmt::Debug {
  fn get_color(&self, u: f64, v: f64) -> ColorVector;
  fn has_texture(&self) -> bool;
  fn get_gloss(&self) -> f64;
  fn get_reflection(&self) -> f64;
  fn get_refraction(&self) -> f64;
  fn get_transparency(&self) -> f64;
}

#[derive(Debug, Copy, Clone)]
pub struct BaseMaterial {
  gloss: f64,
  reflection: f64,
  refraction: f64,
  transparency: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct SolidMaterial {
  material: BaseMaterial,
  color: ColorVector,
}

impl SolidMaterial {
  pub fn new(
    gloss: f64,
    reflection: f64,
    refraction: f64,
    transparency: f64,
    color: ColorVector,
  ) -> SolidMaterial {
    SolidMaterial {
      material: BaseMaterial {
        gloss,
        reflection,
        refraction,
        transparency,
      },
      color,
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct ChessboardMaterial {
  material: BaseMaterial,
  color_even: ColorVector,
  color_odd: ColorVector,
  density: f64,
}

impl Material for SolidMaterial {
  fn get_color(&self, _u: f64, _v: f64) -> ColorVector {
    self.color.clone()
  }
  fn has_texture(&self) -> bool {
    false
  }
  fn get_gloss(&self) -> f64 {
    self.material.gloss
  }
  fn get_reflection(&self) -> f64 {
    self.material.reflection
  }
  fn get_refraction(&self) -> f64 {
    self.material.refraction
  }
  fn get_transparency(&self) -> f64 {
    self.material.transparency
  }
}

impl Material for ChessboardMaterial {
  fn get_color(&self, u: f64, v: f64) -> ColorVector {
    self.color_even.clone()
  }
  fn has_texture(&self) -> bool {
    false
  }
  fn get_gloss(&self) -> f64 {
    self.material.gloss
  }
  fn get_reflection(&self) -> f64 {
    self.material.reflection
  }
  fn get_refraction(&self) -> f64 {
    self.material.refraction
  }
  fn get_transparency(&self) -> f64 {
    self.material.transparency
  }
}

trait Shape: fmt::Debug {
  fn get_position(&self) -> PosVector;
  fn intersect(&self, ray: &Ray) -> IntersectionInfo;
  fn get_id(&self) -> u32;
  fn get_material(&self) -> Arc<Material>;
}

trait Light: fmt::Debug {
  fn get_position(&self) -> PosVector;
  fn get_color(&self) -> ColorVector;
}

#[derive(Debug)]
pub struct PointLight {
  position: PosVector,
  color: ColorVector,
}

impl Light for PointLight {
  fn get_position(&self) -> PosVector {
    self.position
  }
  fn get_color(&self) -> ColorVector {
    self.color
  }
}

#[derive(Debug, Clone)]
pub struct SphereShape {
  position: PosVector,
  radius: f64,
  material: Arc<Material>,
  id: u32,
}

impl Shape for SphereShape {
  fn get_position(&self) -> PosVector {
    self.position
  }

  fn intersect(&self, ray: &Ray) -> IntersectionInfo {
    let dst = ray.position.subtract(self.position);
    let b = dst.dot_product(ray.direction);
    let c = dst.dot_product(dst) - (self.radius * self.radius);
    let d = b * b - c;

    if d > 0.0 {
      let distance = -b - d.sqrt();
      let position = ray.position.add(ray.direction.multiply_by_scalar(distance));
      let normal = position.subtract(self.position).normalize();

      // todo: u/v coordinate texture mapping if self.material has a texture
      let color = self.material.get_color(0.0, 0.0);

      // println!("intersected!");

      // found intersection
      IntersectionInfo {
        color,
        distance,
        element: Some(Arc::new(self.clone())), // Some(Box::from((*self).clone()), // Box::from((*self).clone()),
        is_hit: true,
        hit_count: 1,
        normal,
        position,
      }
    } else {
      IntersectionInfo::new_default()
    }
  }

  fn get_id(&self) -> u32 {
    self.id
  }

  fn get_material(&self) -> Arc<Material> {
    self.material.clone()
  }
}

#[derive(Debug)]
pub struct PlaneShape {
  position: PosVector,
  d_val: f64,
  material: Arc<Material>,
}

impl Shape for PlaneShape {
  fn get_position(&self) -> PosVector {
    self.position
  }
  fn intersect(&self, ray: &Ray) -> IntersectionInfo {
    IntersectionInfo::new_default()
  }
  fn get_id(&self) -> u32 {
    1
  }
  fn get_material(&self) -> Arc<Material> {
    self.material.clone()
  }
}

#[derive(Debug)]
pub struct Scene {
  background: Background,
  shapes: Vec<Box<Shape>>,
  lights: Vec<Box<Light>>,

  render_diffuse: bool,
  render_reflection: bool,
  render_refraction: bool,
  render_shadow: bool,
  render_highlights: bool,
}

unsafe impl Send for Scene {}
unsafe impl Sync for Scene {}

#[derive(Debug)]
pub struct IntersectionInfo {
  color: ColorVector,
  distance: f64,
  element: Option<Arc<Shape>>,
  is_hit: bool,
  hit_count: u32,
  normal: PosVector,
  position: PosVector,
}

// #[derive(Debug)]
// pub struct NullShape {}

// impl NullShape {
//   pub fn new() -> NullShape {
//     NullShape {}
//   }
// }

// #[derive(Debug)]
// pub struct NullMaterial {}

// impl Material for NullMaterial {
//   fn get_color(&self, u: f64, v: f64) -> ColorVector {
//     ColorVector {
//       r: 0.0,
//       g: 0.0,
//       b: 0.0,
//     }
//   }
//   fn has_texture(&self) -> bool {
//     false
//   }
//   fn get_gloss(&self) -> f64 {
//     0.0
//   }
//   fn get_reflection(&self) -> f64 {
//     0.0
//   }
//   fn get_refraction(&self) -> f64 {
//     0.0
//   }
//   fn get_transparency(&self) -> f64 {
//     0.0
//   }
// }

// impl Shape for NullShape {
//   fn get_position(&self) -> PosVector {
//     PosVector {
//       x: 0.0,
//       y: 0.0,
//       z: 0.0,
//     }
//   }

//   fn intersect(&self, ray: &Ray) -> IntersectionInfo {
//     IntersectionInfo::new_default()
//   }

//   fn get_id(&self) -> u32 {
//     0
//   }

//   fn get_material(&self) -> Arc<Material> {
//     Arc::new(NullMaterial {})
//   }
// }

impl IntersectionInfo {
  pub fn new_default() -> IntersectionInfo {
    IntersectionInfo {
      color: ColorVector {
        r: 0.0,
        g: 0.0,
        b: 0.0,
      },
      distance: 100000000000.0, // todo: f64 max
      element: None,
      is_hit: false,
      hit_count: 0,
      normal: PosVector {
        x: 0.0,
        y: 0.0,
        z: 0.0,
      },
      position: PosVector {
        x: 0.0,
        y: 0.0,
        z: 0.0,
      },
    }
  }
}

impl Scene {
  pub fn new_marbles_scene(
    background_color: ColorVector,
    background_ambience: f64,
    sphere_radius: f64,
    sphere_distance_increment: f64,
    num_spheres_per_axis: i32,
    show_plane: bool,
    plane_pos: PosVector,
    plane_d_val: f64,
  ) -> Scene {
    let background = Background::new(background_color, background_ambience);

    let blue_material = SolidMaterial::new(
      2.0,
      0.2,
      0.0,
      0.0,
      ColorVector {
        r: 0.0,
        g: 0.0,
        b: 0.9,
      },
    );

    let red_material = SolidMaterial::new(
      2.0,
      0.2,
      0.0,
      0.0,
      ColorVector {
        r: 0.9,
        g: 0.0,
        b: 0.0,
      },
    );

    let green_material = SolidMaterial::new(
      2.0,
      0.2,
      0.0,
      0.0,
      ColorVector {
        r: 0.0,
        g: 0.9,
        b: 0.0,
      },
    );

    let max_axis = sphere_distance_increment * num_spheres_per_axis as f64;

    let mut shapes: Vec<Box<Shape>> = Vec::new();

    let mut x = 0.0;
    let mut id = 1;
    while x <= max_axis {
      shapes.push(Box::new(SphereShape {
        position: PosVector {
          x: x,
          y: 0.0,
          z: 0.0,
        },
        radius: sphere_radius,
        material: Arc::new(blue_material),
        id,
      }));
      x += sphere_distance_increment;
      id = id + 1;
    }

    let mut y = 0.0;
    while y <= max_axis {
      shapes.push(Box::new(SphereShape {
        position: PosVector {
          x: 0.0,
          y: y,
          z: 0.0,
        },
        radius: sphere_radius,
        material: Arc::new(green_material),
        id,
      }));
      y += sphere_distance_increment;
      id = id + 1;
    }

    let mut z = 0.0;
    while z <= max_axis {
      shapes.push(Box::new(SphereShape {
        position: PosVector {
          x: 0.0,
          y: 0.0,
          z: z,
        },
        radius: sphere_radius,
        material: Arc::new(red_material),
        id,
      }));
      z += sphere_distance_increment;
      id = id + 1;
    }

    let chess_mat = ChessboardMaterial {
      color_even: ColorVector {
        r: 0.8,
        g: 0.8,
        b: 0.8,
      },
      color_odd: ColorVector {
        r: 0.0,
        g: 0.0,
        b: 0.0,
      },
      material: BaseMaterial {
        reflection: 0.2,
        transparency: 0.0,
        gloss: 1.0,
        refraction: 0.0,
      },
      density: 15.0,
    };

    if show_plane {
      shapes.push(Box::new(PlaneShape {
        position: plane_pos,
        d_val: plane_d_val,
        material: Arc::new(chess_mat),
      }));
    }

    let mut lights: Vec<Box<Light>> = Vec::new();
    lights.push(Box::new(PointLight {
      position: PosVector {
        x: -5.0,
        y: 10.0,
        z: 10.0,
      },
      color: ColorVector {
        r: 0.8,
        g: 0.8,
        b: 0.8,
      },
    }));

    lights.push(Box::new(PointLight {
      position: PosVector {
        x: 5.0,
        y: 10.0,
        z: 10.0,
      },
      color: ColorVector {
        r: 0.8,
        g: 0.8,
        b: 0.8,
      },
    }));

    let scene = Scene {
      background,
      shapes,
      lights,
      render_diffuse: true,
      render_reflection: true,
      render_refraction: true,
      render_shadow: true,
      render_highlights: true,
    };

    scene
  }
}

#[derive(Debug, Copy, Clone)]
pub struct RenderData {
  pub width: u32,
  pub height: u32,
  pub ray_trace_depth: u32,
  pub processor_count: u32,
}

pub struct Ray {
  position: PosVector,
  direction: PosVector,
}

#[derive(Debug)]
pub struct PixelArray {
  pub width: u32,
  pub height: u32,
  imgbuf: image::RgbImage,
}

impl PixelArray {
  pub fn new(width: u32, height: u32) -> PixelArray {
    PixelArray {
      width,
      height,
      imgbuf: image::RgbImage::new(width, height),
    }
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

#[derive(Debug)]
pub struct RayTracer {
  pub camera: Camera,
  pub render_data: RenderData,
  pub scene: Scene,
}

impl RayTracer {
  fn get_reflection_ray(&self, p: PosVector, n: PosVector, v: PosVector) -> Ray {
    let c1 = -(n.dot_product(v));
    let rl = v.add(n.multiply_by_scalar(2.0).multiply_by_scalar(c1));
    Ray {
      position: p,
      direction: rl,
    }
  }

  fn get_refraction_ray(&self, p: PosVector, n: PosVector, v: PosVector, refraction: f64) -> Ray {
    let c1 = n.dot_product(v);
    let c2 = 1.0 - refraction * refraction * (1.0 - c1 * c1).sqrt();
    let t = n.multiply_by_scalar(refraction * c1 - c2)
      .subtract(v.multiply_by_scalar(refraction))
      .multiply_by_scalar(-1.0)
      .normalize();
    Ray {
      position: p,
      direction: t,
    }
  }

  fn test_intersection(&self, ray: &Ray, exclude: Option<Arc<Shape>>) -> IntersectionInfo {
    let mut hit_count = 0;
    let mut best_info = IntersectionInfo::new_default();

    let mut exclude_shape_id = 0;

    if let Some(exclude_shape) = exclude {
      exclude_shape_id = exclude_shape.get_id();
    } 

    for shape in &self.scene.shapes {
      /*todo: Some(exclude) || */

      if shape.get_id() != exclude_shape_id {
        let info = shape.intersect(ray);
        if info.is_hit && info.distance < best_info.distance && info.distance >= 0.0 {
          best_info = info;
          hit_count = hit_count + 1;
        }
      }
    }

    best_info.hit_count = hit_count;
    best_info
  }

  fn ray_trace(&self, intersection_info: &IntersectionInfo, ray: &Ray, depth: u32) -> ColorVector {
    let mut color = intersection_info
      .color
      .multiply_by_scalar(self.scene.background.ambience);
    // let shininess = 10.pow(intersection_info.element.material.gloss + 1.0);

    // foreach light
    // rust note:  need the & since we want references to the items in the iteration.
    // by default, for loops use into_iter<> which is a MOVE
    // http://hermanradtke.com/2015/06/22/effectively-using-iterators-in-rust.html
    for light in &self.scene.lights {
      // calc diffuse lighting
      let v = light
        .get_position()
        .subtract(intersection_info.position)
        .normalize();

      if self.scene.render_diffuse {
        let l = v.dot_product(intersection_info.normal);
        if l > 0.0 {
          let temp_color = ColorVector {
            r: 0.0,
            g: 0.0,
            b: 0.0,
          };
          color = color.add(temp_color.multiply_by_scalar(l))
        }
      }

      // max depth of raytracing.  increasing depth calculates more color, but takes exp longer
      if depth < self.render_data.ray_trace_depth {
        // calculate reflection ray

        if self.scene.render_reflection
        {
          match intersection_info.element.clone() {
            None => {},
            Some(elem) => {
              if elem.get_material().get_reflection() > 0.0 {
                let reflection_ray = self.get_reflection_ray(
                  intersection_info.position,
                  intersection_info.normal,
                  ray.direction,
                );
                let mut refl = self.test_intersection(&reflection_ray, Some(elem.clone()));
                if refl.is_hit && refl.distance > 0.0 {
                  refl.color = self.ray_trace(&refl, &reflection_ray, depth + 1);
                } else {
                  refl.color = self.scene.background.color;
                }

                color = color.blend(
                  refl.color,
                  elem.get_material().get_reflection(),
                );
              }
            }
          }
        }

        if self.scene.render_refraction {
          match intersection_info.element.clone() {
            None => {},
            Some(elem) => {
              if elem.get_material().get_transparency() > 0.0 {
                let refraction_ray = self.get_refraction_ray(
                  intersection_info.position,
                  intersection_info.normal,
                  ray.direction,
                  elem.clone().get_material().get_refraction(),
                );
                let mut refr = elem.clone().intersect(&refraction_ray);
                if refr.is_hit {
                  match refr.element {
                    None => {},
                    Some(refrelem) => {
                      let element_refraction_ray = self.get_refraction_ray(
                        refr.position,
                        refr.normal,
                        refraction_ray.direction,
                        refrelem.get_material().get_refraction(),
                      );
                      refr =
                        self.test_intersection(&element_refraction_ray, Some(elem.clone()));
                      if refr.is_hit && refr.distance > 0.0 {
                        refr.color = self.ray_trace(&refr, &element_refraction_ray, depth + 1);
                      } else {
                        refr.color = self.scene.background.color;
                      }
                    }
                  }
                } else {
                  refr.color = self.scene.background.color;
                }
                color = color.blend(
                  refr.color,
                  elem.get_material().get_transparency(),
                );
              }
            }
          }
        }

        let shadow_ray = Ray {
          position: intersection_info.position,
          direction: v,
        };

        match intersection_info.element.clone() {
          None => {},
          Some(elem) => {
            let shadow_intersection =
              self.test_intersection(&shadow_ray, Some(elem.clone()));
            if self.scene.render_shadow {
              if shadow_intersection.is_hit {
                match shadow_intersection.element {
                  None => {},
                  Some(shadowelem) => {
                    if shadowelem.get_id() != elem.get_id()
                    {
                      let trans = shadowelem
                        .get_material()
                        .get_transparency();
                      let trans_power = trans.powf(0.5);
                      color = color.multiply_by_scalar(0.5 + (0.5 * trans_power)); // todo: make sure this is ordered correctly for the power calculation
                    }
                  }
                }
              }
            }

            if self.scene.render_highlights && !shadow_intersection.is_hit
              && elem.get_material().get_gloss() > 0.0
            {
              let lv = elem
                .get_position()
                .subtract(light.get_position())
                .normalize();
              let e = self
                .camera
                .position
                .subtract(elem.get_position())
                .normalize();
              let h = e.subtract(lv).normalize();
              let gloss_weight = 0.0; // todo: pow(std::max(dot(info->Normal(), h), 0.0), shininess);
              color = color.add(light.get_color().multiply_by_scalar(gloss_weight));
            }
          }
        }
      }
    }

    color
  }

  fn calculate_color(&self, ray: &Ray) -> ColorVector {
    let intersection_info = self.test_intersection(ray, None);
    if intersection_info.is_hit {
      self.ray_trace(&intersection_info, ray, 0)
    } else {
      self.scene.background.color
    }
  }

  pub fn get_pixel_color(&self, x: u32, y: u32) -> ColorVector {
    let xp = x as f64 / self.render_data.width as f64 * 2.0 - 1.0;
    let yp = y as f64 / self.render_data.height as f64 * 2.0 - 1.0;

    let ray = self.camera.get_ray(xp, yp);
    self.calculate_color(&ray)
  }
}

// threading
enum Message {
  NewJob(Job),
  Terminate,
}

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>,
}

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>) {
    (*self)()
  }
}

type Job = Box<FnBox + Send + 'static>;

impl ThreadPool {
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);

    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::with_capacity(size);

    for id in 0..size {
      workers.push(Worker::new(id, Arc::clone(&receiver)));
    }

    ThreadPool { workers, sender }
  }

  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);
    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    // println!("Sending terminate message to all workers.");

    for _ in &mut self.workers {
      self.sender.send(Message::Terminate).unwrap();
    }

    // println!("Shutting down all workers.");
    for worker in &mut self.workers {
      // println!("Shutting down worker {}", worker.id);

      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

struct Worker {
  id: usize,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    let thread = thread::spawn(move || loop {
      let message = receiver.lock().unwrap().recv().unwrap();

      match message {
        Message::NewJob(job) => {
          // println!("Worker {} got a job; executing.", id);
          job.call_box();
        }
        Message::Terminate => {
          // println!("Worker {} was told to terminate.", id);
          break;
        }
      }
    });

    Worker {
      id,
      thread: Some(thread),
    }
  }
}
