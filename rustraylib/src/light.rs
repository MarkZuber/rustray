use std::fmt;

use posvector::PosVector;
use color::ColorVector;

pub trait Light: fmt::Debug {
  fn get_position(&self) -> PosVector;
  fn get_color(&self) -> ColorVector;
}

#[derive(Debug)]
pub struct PointLight {
  position: PosVector,
  color: ColorVector,
}

impl PointLight {
  pub fn new(position: PosVector, color: ColorVector) -> PointLight {
    PointLight { position, color }
  }
}

impl Light for PointLight {
  fn get_position(&self) -> PosVector {
    self.position
  }
  fn get_color(&self) -> ColorVector {
    self.color
  }
}
