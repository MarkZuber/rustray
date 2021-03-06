use std::fmt;
use std::f64;
use std::sync::Arc;

use material::Material;
use posvector::PosVector;
use camera::Ray;
use tracer::IntersectionInfo;

#[derive(Debug)]
pub struct Bound {
  pub min: f64,
  pub max: f64
}

impl Bound {
  pub fn new(min: f64, max: f64) -> Bound {
    Bound { min, max }
  }
}

// Axis Aligned Bounding Box for kdTree subdivision of shapes
#[derive(Debug)]
pub struct BoundingBox {
  pub boxmin: PosVector, // lower corner (min value for all coords)
  pub boxmax: PosVector, // upper corner (max value for all coords)
}

enum ValSign {
  Negative,
  Zero,
  Positive,
}

impl BoundingBox {
  fn minf64(x: f64, y: f64) -> f64 {
    if x < y {
      x
    } else {
      y
    }
  }

  fn maxf64(x: f64, y: f64) -> f64 {
    if x > y {
      x
    } else {
      y
    }
  }

  pub fn from_shape(shape: &Arc<Box<Shape>>) -> BoundingBox {
    BoundingBox::new(
      shape.calculate_bounding_planes(PosVector::new_unit_x()),
      shape.calculate_bounding_planes(PosVector::new_unit_y()),
      shape.calculate_bounding_planes(PosVector::new_unit_z()),
    )
  }

  pub fn new(
    bounding_x: Bound,
    bounding_y: Bound,
    bounding_z: Bound,
  ) -> BoundingBox {
    BoundingBox {
      boxmin: PosVector::new(bounding_x.min, bounding_y.min, bounding_z.min),
      boxmax: PosVector::new(bounding_x.max, bounding_y.max, bounding_z.max)
    }
  }

  pub fn get_enlarged_to_enclose(&self, other: &BoundingBox) -> BoundingBox {
    BoundingBox::new(
        Bound::new(BoundingBox::minf64(self.boxmin.x, other.boxmin.x), BoundingBox::maxf64(self.boxmax.x, other.boxmax.x)),
        Bound::new(BoundingBox::minf64(self.boxmin.x, other.boxmin.x), BoundingBox::maxf64(self.boxmax.x, other.boxmax.x)),
        Bound::new(BoundingBox::minf64(self.boxmin.x, other.boxmin.x), BoundingBox::maxf64(self.boxmax.x, other.boxmax.x)),
      )
  }

  pub fn get_box_min(&self) -> PosVector {
    self.boxmin
  }

  pub fn get_box_max(&self) -> PosVector {
    self.boxmax
  }

  pub fn is_well_formed(&self) -> bool {
    self.boxmin.x <= self.boxmax.x && self.boxmin.y <= self.boxmax.y
      && self.boxmin.z <= self.boxmax.z
  }

  pub fn is_empty(&self) -> bool {
    self.boxmax.x < self.boxmin.x || self.boxmax.y < self.boxmin.y || self.boxmax.z < self.boxmin.z
  }

  pub fn get_surface_area(&self) -> f64 {
    let delta = self.boxmax.subtract(self.boxmin);
    (delta.x * delta.y + delta.x * delta.z + delta.y * delta.z) * 2.0
  }

  fn calc_sign(&self, val: f64) -> ValSign {
    if val < 0.0 {
      ValSign::Negative
    } else if val == 0.0 {
      ValSign::Zero
    } else {
      ValSign::Positive
    }
  }

  fn invert_posvector(&self, posvec: PosVector) -> (PosVector, ValSign, ValSign, ValSign) {
    let sign_dir_x = self.calc_sign(posvec.x);
    let sign_dir_y = self.calc_sign(posvec.y);
    let sign_dir_z = self.calc_sign(posvec.z);

    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut z: f64 = 0.0;

    match sign_dir_x {
      ValSign::Zero => {}
      _ => x = 1.0 / posvec.x,
    }
    match sign_dir_y {
      ValSign::Zero => {}
      _ => y = 1.0 / posvec.y,
    }
    match sign_dir_z {
      ValSign::Zero => {}
      _ => z = 1.0 / posvec.z,
    }

    (PosVector::new(x, y, z), sign_dir_x, sign_dir_y, sign_dir_z)
  }

  fn is_possible_intersect(&self, sign_dir: &ValSign, pos: f64, min: f64, max: f64) -> bool {
    match *sign_dir {
      ValSign::Zero => !(pos < min || pos > max),
      _ => true,
    }
  }

  pub fn get_ray_intersects(&self, ray: &Ray) -> bool {
    // Set sign of dir components and inverse values of non-zero entries.
    let (dir_inv, sign_dir_x, sign_dir_y, sign_dir_z) = self.invert_posvector(ray.get_direction());

    if self.is_possible_intersect(
      &sign_dir_x,
      ray.get_position().x,
      self.boxmin.x,
      self.boxmax.x,
    )
      || self.is_possible_intersect(
        &sign_dir_y,
        ray.get_position().y,
        self.boxmin.y,
        self.boxmax.y,
      )
      || self.is_possible_intersect(
        &sign_dir_z,
        ray.get_position().z,
        self.boxmin.z,
        self.boxmax.z,
      ) {
      let mut max_enter_dist: f64 = 0.0;
      // let mut max_enter_axis: i32 = 0;
      let mut min_exit_dist: f64 = 0.0;
      // let mut min_exit_axis: i32 = 0;

      let mut mx: f64 = 0.0;
      let mut mn: f64 = 0.0;

      match sign_dir_x {
        ValSign::Zero => {
          max_enter_dist = -f64::MAX;
          min_exit_dist = f64::MAX;
          // max_enter_axis = -1;
          // min_exit_axis = -1;
        }
        ValSign::Positive => {
          mx = self.boxmax.x;
          mn = self.boxmin.x;
        }
        ValSign::Negative => {
          mx = self.boxmin.x;
          mn = self.boxmax.x;
        }
      };

      match sign_dir_x {
        ValSign::Zero => {}
        _ => {
          max_enter_dist = (mn - ray.get_position().x) * dir_inv.x;
          min_exit_dist = (mx - ray.get_position().x) * dir_inv.x;
          // max_enter_axis = 0;
          // min_exit_axis = 0;
        }
      }

      match sign_dir_y {
        ValSign::Zero => {}
        ValSign::Positive => {
          mx = self.boxmax.y;
          mn = self.boxmin.y;
        }
        ValSign::Negative => {
          mx = self.boxmin.y;
          mn = self.boxmax.y;
        }
      };

      match sign_dir_y {
        ValSign::Zero => {}
        _ => {
          let new_enter_dist = (mn - ray.get_position().y) * dir_inv.y;
          let new_exit_dist = (mx - ray.get_position().y) * dir_inv.y;
          if max_enter_dist < new_enter_dist {
            max_enter_dist = new_enter_dist;
            // max_enter_axis = 1;
          }
          if min_exit_dist > new_exit_dist {
            min_exit_dist = new_exit_dist;
            // min_exit_axis = 1;
          }
        }
      }

      match sign_dir_z {
        ValSign::Zero => {}
        ValSign::Positive => {
          mx = self.boxmax.z;
          mn = self.boxmin.z;
        }
        ValSign::Negative => {
          mx = self.boxmin.z;
          mn = self.boxmax.z;
        }
      }

      match sign_dir_z {
        ValSign::Zero => {}
        _ => {
          let new_enter_dist = (mn - ray.get_position().z) * dir_inv.z;
          let new_exit_dist = (mx - ray.get_position().z) * dir_inv.z;
          if max_enter_dist < new_enter_dist {
            max_enter_dist = new_enter_dist;
            // max_enter_axis = 2;
          }
          if min_exit_dist > new_exit_dist {
            min_exit_dist = new_exit_dist;
            // min_exit_axis = 2;
          }
        }
      }

      if min_exit_dist < max_enter_dist || min_exit_dist < 0.0 {
        false
      } else {
        true
      }
    } else {
      false
    }
  }
}

pub trait Shape: fmt::Debug {
  fn get_position(&self) -> PosVector;
  fn intersect(&self, ray: &Ray) -> IntersectionInfo;
  fn get_material(&self) -> Arc<Material>;
  fn calculate_bounding_planes(&self, unit_vec: PosVector) -> Bound;
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
    let max_distance: f64 = f64::MAX;

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

    let _returned_pos = q; // point of intersection
    let color = if front_face {
      self.front_material.get_color(v_coord, w_coord)
    } else {
      self.back_material.get_color(v_coord, w_coord)
    };

    if no_intersection {
      IntersectionInfo::new_default()
    } else {
      // found intersection
      IntersectionInfo::new(color,intersect_distance,self.normal, q)
    }
  }

  // todo: we will want to clone a reference to the selected material into the
  // intersection_info instead of getting it directly from the shape
  // so we can allow front/back textures on triangles and other shapes.
  fn get_material(&self) -> Arc<Material> {
    self.front_material.clone()
  }

  fn calculate_bounding_planes(&self, unit_vec: PosVector) -> Bound {
    let mut min_d = unit_vec.dot_product(self.va);
    let mut max_d = unit_vec.dot_product(self.vb);

    if max_d < min_d {
      // swap
      let temp = max_d;
      max_d = min_d;
      min_d = temp;
    }

    let t = unit_vec.dot_product(self.vc);
    if t < min_d {
      min_d = t;
    } else {
      max_d = t;
    }

    Bound::new(min_d, max_d)
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
      IntersectionInfo::new(color,distance,normal, position)
    } else {
      IntersectionInfo::new_default()
    }
  }

  fn get_material(&self) -> Arc<Material> {
    self.material.clone()
  }

  fn calculate_bounding_planes(&self, unit_vec: PosVector) -> Bound {
    let cd = unit_vec.dot_product(self.position);
    Bound::new(cd + self.radius, cd - self.radius)
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
        IntersectionInfo::new(color,t,self.position,intersect_position)
      }
    }
  }

  fn get_material(&self) -> Arc<Material> {
    self.material.clone()
  }

  fn calculate_bounding_planes(&self, _unit_vec: PosVector) -> Bound {
    Bound::new(1.0, 1.0)
  }
}
