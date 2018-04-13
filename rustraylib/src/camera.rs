use posvector::PosVector;

#[derive(Debug)]
pub struct Ray {
  position: PosVector,
  direction: PosVector,
}

impl Ray {
  pub fn new(position: PosVector, direction: PosVector) -> Ray {
    Ray { position, direction }
  }

  pub fn get_position(&self) -> PosVector {
    self.position
  }

  pub fn get_direction(&self) -> PosVector {
    self.direction
  }
}


#[derive(Debug, Clone)]
pub struct Camera {
  pub position: PosVector,
  pub look_at: PosVector,
  pub up: PosVector,
  pub fov: f64,

  a1: PosVector,  // vector x axis of screen
  a2: PosVector,  // vector y axis of screen
  a3: PosVector,  // vector into the screen
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

  pub fn get_position(&self) -> PosVector {
    self.position
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
