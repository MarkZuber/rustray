use std::fmt;
use std::sync::Arc;

use material::Material;
use posvector::PosVector;
use camera::Ray;
use tracer::IntersectionInfo;

pub trait Shape: fmt::Debug {
  fn get_position(&self) -> PosVector;
  fn intersect(&self, ray: &Ray) -> IntersectionInfo;
  fn get_id(&self) -> u32;
  fn get_material(&self) -> Arc<Material>;
}

#[derive(Debug, Clone)]
pub struct SphereShape {
  pub position: PosVector,
  pub radius: f64,
  pub material: Arc<Material>,
  pub id: u32,
}

impl Shape for SphereShape {
  fn get_position(&self) -> PosVector {
    self.position
  }

  fn intersect(&self, ray: &Ray) -> IntersectionInfo {
    let dst = ray.get_position().subtract(self.position);
    let b = dst.dot_product(ray.get_direction());
    let c = dst.dot_product(dst) - (self.radius * self.radius);
    let d = b * b - c;

    if d > 0.0 {
      let distance = -b - d.sqrt();
      let position = ray.get_position().add(ray.get_direction().multiply_by_scalar(distance));
      let normal = position.subtract(self.position).normalize();

      // todo: u/v coordinate texture mapping if self.material has a texture
      let color = self.material.get_color(0.0, 0.0);

      // println!("intersected sphere!");

      // found intersection
      IntersectionInfo {
        color,
        distance,
        element: Some(Arc::new(self.clone())),
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

#[derive(Debug, Clone)]
pub struct PlaneShape {
  pub position: PosVector,
  pub d_val: f64,
  pub material: Arc<Material>,
  pub id: u32
}

impl Shape for PlaneShape {
  fn get_position(&self) -> PosVector {
    self.position
  }
  fn intersect(&self, ray: &Ray) -> IntersectionInfo {
    let vd = self.position.dot_product(ray.get_direction());

    if vd >= 0.0 {
      IntersectionInfo::new_default()
    } else {
      let t = -(self.position.dot_product(ray.get_position()) + self.d_val) / vd;

      if t <= 0.0 {
        IntersectionInfo::new_default()
      } else {
        let intersect_position = ray.get_position().add(ray.get_direction().multiply_by_scalar(t));

        let mut color = self.material.get_color(0.0, 0.0);
        if self.material.has_texture() {
          let vec_u = PosVector::new(self.position.y, self.position.z, -self.position.x);
          let vec_v = vec_u.cross(self.position);
          let u = intersect_position.dot_product(vec_u);
          let v = intersect_position.dot_product(vec_v);
          color = self.material.get_color(u, v);
        }

        // println!("intersected plane!");

        IntersectionInfo {
          color,
          distance: t,
          element: Some(Arc::new(self.clone())),
          is_hit: true,
          hit_count: 1,
          normal: self.position,
          position: intersect_position,
        }        
      } 
    }
  }

  fn get_id(&self) -> u32 {
    self.id
  }
  fn get_material(&self) -> Arc<Material> {
    self.material.clone()
  }
}
