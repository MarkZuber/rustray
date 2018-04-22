use std::sync::Arc;
use color::ColorVector;
use posvector::PosVector;
use camera::{Camera, Ray};
use renderer::RenderData;
use scene::{Scene,CompiledShape,CompiledLight};

#[derive(Debug)]
pub struct IntersectionInfo {
  pub color: ColorVector,
  pub distance: f64,
  pub element_id: u32,
  pub is_hit: bool,
  pub normal: PosVector,
  pub position: PosVector,
}

impl IntersectionInfo {
  pub fn new_default() -> IntersectionInfo {
    IntersectionInfo {
      color: ColorVector::new(0.0, 0.0, 0.0),
      distance: 100000000000.0, // todo: f64 max
      element_id: 0,
      is_hit: false,
      normal: PosVector::new_default(),
      position: PosVector::new_default(),
    }
  }

  pub fn new(color: ColorVector, distance: f64, normal: PosVector, position: PosVector) -> IntersectionInfo {
    IntersectionInfo {
      color,
      distance,
      element_id: 0,
      is_hit: true,
      normal,
      position
    }    
  }
}

#[derive(Debug)]
pub struct RayTraceStatistics {
  pub num_rays_traced: u64,
}

impl RayTraceStatistics {
  pub fn new() -> RayTraceStatistics {
    RayTraceStatistics { num_rays_traced: 0 }
  }

  pub fn add_ray_traced(&mut self) {
    self.num_rays_traced = self.num_rays_traced + 1;
  }
}

#[derive(Debug)]
pub struct RayTracer {
  pub camera: Camera,
  pub render_data: RenderData,
  pub scene: Arc<Scene>,
  pub use_kd_tree: bool,
  pub stats: Arc<RayTraceStatistics>,
}

impl RayTracer {
  pub fn new(camera: Camera, render_data: RenderData, scene: Arc<Scene>) -> RayTracer {
    RayTracer {
      camera,
      render_data,
      scene,
      use_kd_tree: false,
      stats: Arc::new(RayTraceStatistics::new()),
    }
  }

  fn get_reflection_ray(&self, p: PosVector, n: PosVector, v: PosVector) -> Ray {
    let c1 = -(n.dot_product(v));
    let rl = v.add(n.multiply_by_scalar(2.0).multiply_by_scalar(c1));
    Ray::new(p, rl)
  }

  fn get_refraction_ray(&self, p: PosVector, n: PosVector, v: PosVector, refraction: f64) -> Ray {
    let c1 = n.dot_product(v);
    let c2 = 1.0 - refraction * refraction * (1.0 - c1 * c1).sqrt();
    let t = n.multiply_by_scalar(refraction * c1 - c2)
      .subtract(v.multiply_by_scalar(refraction))
      .multiply_by_scalar(-1.0)
      .normalize();
    Ray::new(p, t)
  }

  fn test_intersection_basic(&self, ray: &Ray, exclude_id: u32) -> IntersectionInfo {
    let mut best_info = IntersectionInfo::new_default();

    for (_, shape) in &self.scene.shapes {
      if shape.get_id() != exclude_id {
        let info = shape.intersect(ray);
        if info.is_hit && info.distance < best_info.distance && info.distance >= 0.0 {
          best_info = info;
        }
      }
    }

    best_info
  }

  fn test_intersection_kd(&self, _ray: &Ray, _exclude_id: u32) -> IntersectionInfo {
    IntersectionInfo::new_default()
  }

  fn test_intersection(&self, ray: &Ray, exclude_id: u32) -> IntersectionInfo {
    // self.stats.add_ray_traced();
    if self.use_kd_tree {
      self.test_intersection_kd(ray, exclude_id)
    } else {
      self.test_intersection_basic(ray, exclude_id)
    }
  }

  fn render_diffuse(
    &self,
    current_color: ColorVector,
    intersection_info: &IntersectionInfo,
    light: &Box<CompiledLight>,
  ) -> ColorVector {
    let mut color = current_color;
    if self.render_data.render_diffuse {
      let v = light
        .get_position()
        .subtract(intersection_info.position)
        .normalize();

      let l = v.dot_product(intersection_info.normal);
      if l > 0.0 {
        color = color.add(
          intersection_info
            .color
            .multiply(light.get_color())
            .multiply_by_scalar(l),
        )
      }
    }
    color
  }

  fn render_reflection(
    &self,
    current_color: ColorVector,
    intersection_info: &IntersectionInfo,
    ray: &Ray,
    depth: u32,
  ) -> ColorVector {
    let mut color = current_color;

    if self.render_data.render_reflection {
      match self.scene.get_shape(&intersection_info.element_id) {
        None => {}
        Some(elem) => {
          if elem.get_material().get_reflection() > 0.0 {
            let reflection_ray = self.get_reflection_ray(
              intersection_info.position,
              intersection_info.normal,
              ray.get_direction(),
            );
            let mut refl = self.test_intersection(&reflection_ray, elem.get_id());
            if refl.is_hit && refl.distance > 0.0 {
              refl.color = self.ray_trace(&refl, &reflection_ray, depth + 1);
            } else {
              refl.color = self.scene.background.color;
            }

            color = color.blend(refl.color, elem.get_material().get_reflection());
          }
        }
      }
    }

    color
  }

  fn render_refraction(
    &self,
    current_color: ColorVector,
    intersection_info: &IntersectionInfo,
    ray: &Ray,
    depth: u32,
  ) -> ColorVector {
    let mut color = current_color;

    if self.render_data.render_refraction {
      match self.scene.get_shape(&intersection_info.element_id) {
        None => {}
        Some(elem) => {
          if elem.get_material().get_transparency() > 0.0 {
            let refraction_ray = self.get_refraction_ray(
              intersection_info.position,
              intersection_info.normal,
              ray.get_direction(),
              elem.clone().get_material().get_refraction(),
            );
            let mut refr = elem.clone().intersect(&refraction_ray);
            if refr.is_hit {
              match self.scene.get_shape(&refr.element_id) {
                None => {}
                Some(refrelem) => {
                  let element_refraction_ray = self.get_refraction_ray(
                    refr.position,
                    refr.normal,
                    refraction_ray.get_direction(),
                    refrelem.get_material().get_refraction(),
                  );
                  refr = self.test_intersection(&element_refraction_ray, elem.get_id());
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
            color = color.blend(refr.color, elem.get_material().get_transparency());
          }
        }
      }
    }
    color
  }

  fn render_highlights(
    &self,
    current_color: ColorVector,
    elem: &CompiledShape,
    shadow_intersection: &IntersectionInfo,
    light: &Box<CompiledLight>,
  ) -> ColorVector {
    let mut color = current_color;
    if self.render_data.render_highlights && !shadow_intersection.is_hit
      && elem.get_material().get_gloss() > 0.0
    {
      let lv = elem
        .get_position()
        .subtract(light.get_position())
        .normalize();
      let e = self
        .camera
        .get_position()
        .subtract(elem.get_position())
        .normalize();
      let _h = e.subtract(lv).normalize();
      let gloss_weight = 0.0; // todo: pow(std::max(dot(info->Normal(), h), 0.0), shininess);
      color = color.add(light.get_color().multiply_by_scalar(gloss_weight));
    }
    color
  }

  fn render_shadow_and_highlights(
    &self,
    current_color: ColorVector,
    intersection_info: &IntersectionInfo,
    light: &Box<CompiledLight>,
  ) -> ColorVector {
    let mut color = current_color;

    let v = light
      .get_position()
      .subtract(intersection_info.position)
      .normalize();

    let shadow_ray = Ray::new(intersection_info.position, v);

    match self.scene.get_shape(&intersection_info.element_id) {
      None => {}
      Some(elem) => {
        let shadow_intersection = self.test_intersection(&shadow_ray, elem.get_id());
        if self.render_data.render_shadow {
          if shadow_intersection.is_hit {
            match self.scene.get_shape(&shadow_intersection.element_id) {
              None => {}
              Some(shadowelem) => {
                if shadowelem.clone().get_id() != elem.get_id() {
                  let trans = shadowelem.clone().get_material().get_transparency();
                  let trans_power = trans.powf(0.5);
                  color = color.multiply_by_scalar(0.5 + (0.5 * trans_power)); // todo: make sure this is ordered correctly for the power calculation
                }
              }
            }
          }
        }

        color = self.render_highlights(color, elem.clone(), &shadow_intersection, light);
      }
    }
    color
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
    for (_, light) in &self.scene.lights {
      color = self.render_diffuse(color, intersection_info, light);

      // max depth of raytracing.  increasing depth calculates more color, but takes exp longer
      if depth < self.render_data.ray_trace_depth {
        color = self.render_reflection(color, intersection_info, ray, depth);
        color = self.render_refraction(color, intersection_info, ray, depth);
        color = self.render_shadow_and_highlights(color, intersection_info, light);
      }
    }

    color
  }

  fn calculate_color(&self, ray: &Ray) -> ColorVector {
    let intersection_info = self.test_intersection(ray, 0);
    if intersection_info.is_hit {
      self.ray_trace(&intersection_info, ray, 0)
    } else {
      self.scene.background.color
    }
  }

  pub fn get_pixel_color(&self, x: u32, y: u32) -> ColorVector {
    // xp, yp are scaled as -1.0..1.0 each to represent their view range in the image regardless of final resolution.
    let xp = x as f64 / self.render_data.width as f64 * 2.0 - 1.0;
    let yp = -(y as f64 / self.render_data.height as f64 * 2.0 - 1.0); // yp is UP but our pixels are increasing in value DOWN.  so need inversion here.

    // println!("{},{} -> {},{}", x, y, xp, yp);

    let ray = self.camera.get_ray(xp, yp);
    self.calculate_color(&ray)
  }
}
