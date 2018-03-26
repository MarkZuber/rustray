#[derive(Debug, Clone, Copy)]
pub struct ColorVector {
  pub r: f64,
  pub g: f64,
  pub b: f64,
}

impl ColorVector {

  pub fn new(r: f64, g: f64, b: f64) -> ColorVector {
    ColorVector { r, g, b }
  }

  fn clamp_val(val: f64) -> f64 {
    if val < 0.0 {
      0.0
    } else {
      if val > 1.0 {
        1.0
      } else {
        val
      }
    }
  }

  pub fn clamp(&self) -> ColorVector {
    ColorVector {
      r: ColorVector::clamp_val(self.r),
      g: ColorVector::clamp_val(self.g),
      b: ColorVector::clamp_val(self.b),
    }
  }

  pub fn multiply_by_scalar(&self, scalar: f64) -> ColorVector {
    ColorVector {
      r: self.r * scalar,
      g: self.g * scalar,
      b: self.b * scalar,
    }
  }

  pub fn add(&self, other: ColorVector) -> ColorVector {
    ColorVector {
      r: self.r + other.r,
      g: self.g + other.g,
      b: self.b + other.b,
    }
  }

  pub fn blend(&self, other: ColorVector, weight: f64) -> ColorVector {
    let temp = self.multiply_by_scalar(1.0 - weight);
    let temp2 = temp.add(other.multiply_by_scalar(1.0 - weight));
    temp2
  }
}
