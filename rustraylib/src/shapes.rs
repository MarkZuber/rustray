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
pub struct TriangleShape {
  // three vertices in counter-clockwise order
  va: PosVector,
  vb: PosVector,
  vc: PosVector,
  pub front_material: Arc<Material>,
  pub back_material: Arc<Material>,
  pub id: u32,

  edge_ab: PosVector,
  edge_bc: PosVector,
  edge_ca: PosVector,
  normal: PosVector,
  magnitude: f64,
  plane_coefficient: f64,
  u_beta: PosVector,
  u_gamma: PosVector,
}

impl TriangleShape {
  pub fn new(
    va: PosVector,
    vb: PosVector,
    vc: PosVector,
    front_material: Arc<Material>,
    back_material: Arc<Material>,
    id: u32,
  ) -> TriangleShape {
    let edge_ab = vb.subtract(va);
    let edge_bc = vc.subtract(vb);
    let edge_ca = va.subtract(vc);

    let mut normal = if edge_ab.dot_product(edge_bc) < edge_bc.dot_product(edge_ca) {
      edge_ab.cross(edge_bc)
    } else {
      edge_bc.cross(edge_ca)
    };
    let magnitude = normal.magnitude();
    if magnitude > 0.0 {
      normal = normal.divide_by_scalar(magnitude); // Unit vector to triangle's plane
    }
    let plane_coefficient = normal.dot_product(va); // Same coef for all three vertices.

    let mut a = edge_ab.magnitude_squared();
    let mut b = edge_ab.dot_product(edge_ca);
    let mut c = edge_ca.magnitude_squared();
    let dinv = 1.0 / ((a * c) - (b * b));

    a = a * dinv;
    b = b * dinv;
    c = c * dinv;

    let mut u_beta = edge_ab;
    u_beta = u_beta.multiply_by_scalar(c);
    u_beta = u_beta.add_scaled(edge_ca, -b);

    let mut u_gamma = edge_ca;
    u_gamma = u_gamma.multiply_by_scalar(-a);
    u_gamma = u_gamma.add_scaled(edge_ab, b);

    TriangleShape {
      va,
      vb,
      vc,
      front_material,
      back_material,
      id,
      edge_ab,
      edge_bc,
      edge_ca,
      normal,
      magnitude,
      plane_coefficient,
      u_beta,
      u_gamma,
    }
  }

  pub fn is_well_formed(&self) -> bool {
    self.normal.magnitude_squared() > 0.0
  }

  pub fn is_backface_culled(&self) -> bool {
    false // todo: this is if back material is null.  do we ever have that case?
  }
}

impl Shape for TriangleShape {
  fn get_position(&self) -> PosVector {
    self.va
  }

  fn intersect(&self, ray: &Ray) -> IntersectionInfo {
    let max_distance: f64 = 1.7976931348623157e+308f64; // todo: core::f64::MAX;

    let mdotn = ray.get_direction().dot_product(self.normal);
    let planar_dist = (ray.get_position().dot_product(self.normal)) - self.plane_coefficient;

    let mut no_intersection = false;

    let front_face = mdotn <= 0.0;
    if front_face {
      if planar_dist <= 0.0 || planar_dist >= -max_distance * mdotn {
        no_intersection = true;
      }
    } else {
      if self.is_backface_culled() || planar_dist >= 0.0 || -planar_dist >= max_distance * mdotn {
        no_intersection = true;
      }
    }

    let intersect_distance = -planar_dist / mdotn;
    let mut q = ray.get_direction();
    q = q.multiply_by_scalar(intersect_distance);
    q = q.add(ray.get_position()); // point of view line intersecting plane

    // compute barycentric coordinates
    let mut v = q;
    v = v.subtract(self.va);
    let v_coord = v.dot_product(self.u_beta);
    if v_coord < 0.0 {
      no_intersection = true;
    }

    let w_coord = v.dot_product(self.u_gamma);
    if w_coord < 0.0 || v_coord + w_coord > 1.0 {
      no_intersection = true;
    }

    let returned_pos = q; // point of intersection
    let color = if front_face {
      self.front_material.get_color(v_coord, w_coord)
    } else {
      self.back_material.get_color(v_coord, w_coord)
    };

    if no_intersection {
      IntersectionInfo::new_default()
    } else {
      // found intersection
      IntersectionInfo {
        color,
        distance: intersect_distance,
        element: Some(Arc::new(self.clone())),
        is_hit: true,
        hit_count: 1,
        normal: self.normal,
        position: q,
      }
    }
  }

  fn get_id(&self) -> u32 {
    self.id
  }

  // todo: we will want to clone a reference to the selected material into the 
  // intersection_info instead of getting it directly from the shape 
  // so we can allow front/back textures on triangles and other shapes.
  fn get_material(&self) -> Arc<Material> {
    self.front_material.clone()
  }
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
      let position = ray
        .get_position()
        .add(ray.get_direction().multiply_by_scalar(distance));
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
  pub id: u32,
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
        let intersect_position = ray
          .get_position()
          .add(ray.get_direction().multiply_by_scalar(t));

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
