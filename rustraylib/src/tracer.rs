use std::sync::Arc;
use color::ColorVector;
use posvector::PosVector;
use shapes::Shape;
use camera::{Camera, Ray};
use renderer::RenderData;
use scene::Scene;
use light::Light;

#[derive(Debug)]
pub struct IntersectionInfo {
  pub color: ColorVector,
  pub distance: f64,
  pub element: Option<Arc<Shape>>,
  pub is_hit: bool,
  pub hit_count: u32,
  pub normal: PosVector,
  pub position: PosVector,
}

impl IntersectionInfo {
  pub fn new_default() -> IntersectionInfo {
    IntersectionInfo {
      color: ColorVector::new(0.0, 0.0, 0.0),
      distance: 100000000000.0, // todo: f64 max
      element: None,
      is_hit: false,
      hit_count: 0,
      normal: PosVector::new_default(),
      position: PosVector::new_default(),
    }
  }
}

#[derive(Debug)]
pub struct RayTracer {
  pub camera: Camera,
  pub render_data: RenderData,
  pub scene: Arc<Scene>,
}

impl RayTracer {
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

  fn test_intersection(&self, ray: &Ray, exclude: Option<Arc<Shape>>) -> IntersectionInfo {
    let mut hit_count = 0;
    let mut best_info = IntersectionInfo::new_default();

    let mut exclude_shape_id = 0;

    if let Some(exclude_shape) = exclude {
      exclude_shape_id = exclude_shape.get_id();
    }

    for shape in &self.scene.shapes {
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

  fn render_diffuse(
    &self,
    current_color: ColorVector,
    intersection_info: &IntersectionInfo,
    light: &Box<Light>,
  ) -> ColorVector {
    let mut color = current_color;
    if self.scene.render_diffuse {
      let v = light
        .get_position()
        .subtract(intersection_info.position)
        .normalize();

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

    if self.scene.render_reflection {
      match intersection_info.element.clone() {
        None => {}
        Some(elem) => {
          if elem.get_material().get_reflection() > 0.0 {
            let reflection_ray = self.get_reflection_ray(
              intersection_info.position,
              intersection_info.normal,
              ray.get_direction(),
            );
            let mut refl = self.test_intersection(&reflection_ray, Some(elem.clone()));
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

    if self.scene.render_refraction {
      match intersection_info.element.clone() {
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
              match refr.element {
                None => {}
                Some(refrelem) => {
                  let element_refraction_ray = self.get_refraction_ray(
                    refr.position,
                    refr.normal,
                    refraction_ray.get_direction(),
                    refrelem.get_material().get_refraction(),
                  );
                  refr = self.test_intersection(&element_refraction_ray, Some(elem.clone()));
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
    elem: Arc<Shape>,
    shadow_intersection: &IntersectionInfo,
    light: &Box<Light>,
  ) -> ColorVector {
    let mut color = current_color;
    if self.scene.render_highlights && !shadow_intersection.is_hit
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
      let h = e.subtract(lv).normalize();
      let gloss_weight = 0.0; // todo: pow(std::max(dot(info->Normal(), h), 0.0), shininess);
      color = color.add(light.get_color().multiply_by_scalar(gloss_weight));
    }
    color
  }

  fn render_shadow_and_highlights(
    &self,
    current_color: ColorVector,
    intersection_info: &IntersectionInfo,
    light: &Box<Light>,
  ) -> ColorVector {
    let mut color = current_color;

    let v = light
      .get_position()
      .subtract(intersection_info.position)
      .normalize();

    let shadow_ray = Ray::new(intersection_info.position, v);

    match intersection_info.element.clone() {
      None => {}
      Some(elem) => {
        let shadow_intersection = self.test_intersection(&shadow_ray, Some(elem.clone()));
        if self.scene.render_shadow {
          if shadow_intersection.is_hit {
            match shadow_intersection.element.clone() {
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

        color = self.render_highlights(color, elem, &shadow_intersection, light);
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
    for light in &self.scene.lights {
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
    let intersection_info = self.test_intersection(ray, None);
    if intersection_info.is_hit {
      self.ray_trace(&intersection_info, ray, 0)
    } else {
      self.scene.background.color
    }
  }

  pub fn get_pixel_color(&self, x: u32, y: u32) -> ColorVector {
    // xp, yp are scaled as -1.0..1.0 each to represent their view range in the image regardless of final resolution.
    let xp = x as f64 / self.render_data.width as f64 * 2.0 - 1.0;
    let yp = -(y as f64 / self.render_data.height as f64 * 2.0 - 1.0);  // yp is UP but our pixels are increasing in value DOWN.  so need inversion here.

    // println!("{},{} -> {},{}", x, y, xp, yp);

    let ray = self.camera.get_ray(xp, yp);
    self.calculate_color(&ray)
  }
}
