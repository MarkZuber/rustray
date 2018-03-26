use color::ColorVector;
use std::fmt;

pub trait Material: fmt::Debug {
  fn get_color(&self, u: f64, v: f64) -> ColorVector;
  fn has_texture(&self) -> bool;
  fn get_gloss(&self) -> f64;
  fn get_reflection(&self) -> f64;
  fn get_refraction(&self) -> f64;
  fn get_transparency(&self) -> f64;
}

#[derive(Debug, Copy, Clone)]
pub struct BaseMaterial {
  pub gloss: f64,
  pub reflection: f64,
  pub refraction: f64,
  pub transparency: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct SolidMaterial {
  pub material: BaseMaterial,
  pub color: ColorVector,
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
  pub material: BaseMaterial,
  pub color_even: ColorVector,
  pub color_odd: ColorVector,
  pub scale: f64,
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

impl ChessboardMaterial {
  /// <summary>
  /// wraps any value up in the inteval [-1,1] in a rotational manner
  /// e.g. 1.7 -> -0.3
  /// e.g. -1.1 -> 0.9
  /// e.g. -2.3 -> -0.3
  /// </summary>
  /// <param name="t"></param>
  /// <returns></returns>
  // fn wrap_up(t: f64) -> f64 {
  //   let mut x = t % 2.0;
  //   if x < -1.0 {
  //       x = x + 2.0;
  //   }
  //   if x >= 1.0 {
  //       x = x - 2.0;
  //   }
  //   x
  // }

  fn wrap_up_scale(t: f64, scale: f64) -> f64 {
    let mut x = t % scale;
    if x < -scale / 2.0 {
      x = x + scale;
    }
    if x >= scale / 2.0 {
      x = x - scale;
    }
    x
  }
}

impl Material for ChessboardMaterial {
  fn get_color(&self, u: f64, v: f64) -> ColorVector {
    // let t = ChessboardMaterial::wrap_up(u) * ChessboardMaterial::wrap_up(v);
    let t = ChessboardMaterial::wrap_up_scale(u, self.scale)
      * ChessboardMaterial::wrap_up_scale(v, self.scale);
    if t < 0.0 {
      self.color_even.clone()
    } else {
      self.color_odd.clone()
    }
  }

  fn has_texture(&self) -> bool {
    true
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
