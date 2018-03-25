#[derive(Debug, Clone, Copy)]
pub struct PosVector {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl PosVector {
  pub fn subtract(&self, other: PosVector) -> PosVector {
    PosVector {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
    }
  }

  pub fn cross(&self, other: PosVector) -> PosVector {
    PosVector {
      x: self.y * other.z - self.z * other.y,
      y: self.z * other.x - self.x * other.z,
      z: self.x * other.y - self.y * other.x,
    }
  }

  pub fn dot_product(&self, other: PosVector) -> f64 {
    (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
  }

  pub fn magnitude(&self) -> f64 {
    self.magnitude_squared().sqrt()
  }

  pub fn magnitude_squared(&self) -> f64 {
    (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
  }

  pub fn divide_by_scalar(&self, scalar: f64) -> PosVector {
    PosVector {
      x: self.x / scalar,
      y: self.y / scalar,
      z: self.z / scalar,
    }
  }

  pub fn multiply_by_scalar(&self, scalar: f64) -> PosVector {
    PosVector {
      x: self.x * scalar,
      y: self.y * scalar,
      z: self.z * scalar,
    }
  }

  pub fn normalize(&self) -> PosVector {
    self.divide_by_scalar(self.magnitude())
  }

  pub fn add(&self, other: PosVector) -> PosVector {
    PosVector {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
    }
  }
}
