use std::sync::Arc;

use material::{BaseMaterial, SolidMaterial, ChessboardMaterial};
use light::{Light, PointLight};
use shapes::{Shape, SphereShape, PlaneShape };
use posvector::PosVector;
use color::ColorVector;

#[derive(Debug)]
pub struct Background {
  pub color: ColorVector,
  pub ambience: f64,
}

impl Background {
  pub fn new(color: ColorVector, ambience: f64) -> Background {
    Background { color, ambience }
  }
}


#[derive(Debug)]
pub struct Scene {
  pub background: Background,
  pub shapes: Vec<Box<Shape>>,
  pub lights: Vec<Box<Light>>,

  pub render_diffuse: bool,
  pub render_reflection: bool,
  pub render_refraction: bool,
  pub render_shadow: bool,
  pub render_highlights: bool,
}

unsafe impl Send for Scene {}
unsafe impl Sync for Scene {}


impl Scene {
  pub fn new_marbles_scene(
    background_color: ColorVector,
    background_ambience: f64,
    sphere_radius: f64,
    sphere_distance_increment: f64,
    num_spheres_per_axis: i32,
    show_plane: bool,
    plane_pos: PosVector,
    plane_d_val: f64,
  ) -> Scene {
    let background = Background::new(background_color, background_ambience);

    let blue_material = SolidMaterial::new(2.0, 0.2, 0.0, 0.0, ColorVector::new(0.0, 0.0, 0.9));
    let red_material = SolidMaterial::new(2.0,0.2,0.0,0.0,ColorVector::new(0.9, 0.0, 0.0));
    let green_material = SolidMaterial::new(2.0,0.2,0.0,0.0,ColorVector::new(0.0, 0.9, 0.0));

    let max_axis = sphere_distance_increment * num_spheres_per_axis as f64;

    let mut shapes: Vec<Box<Shape>> = Vec::new();

    let mut x = 0.0;
    let mut id = 1;
    while x <= max_axis {
      shapes.push(Box::new(SphereShape {
        position: PosVector::new(x, 0.0, 0.0),
        radius: sphere_radius,
        material: Arc::new(blue_material),
        id,
      }));
      x += sphere_distance_increment;
      id = id + 1;
    }

    let mut y = 0.0;
    while y <= max_axis {
      shapes.push(Box::new(SphereShape {
        position: PosVector::new(0.0, y, 0.0),
        radius: sphere_radius,
        material: Arc::new(green_material),
        id,
      }));
      y += sphere_distance_increment;
      id = id + 1;
    }

    let mut z = 0.0;
    while z <= max_axis {
      shapes.push(Box::new(SphereShape {
        position: PosVector::new(0.0, 0.0, z),
        radius: sphere_radius,
        material: Arc::new(red_material),
        id,
      }));
      z += sphere_distance_increment;
      id = id + 1;
    }

    let chess_mat = ChessboardMaterial {
      color_even: ColorVector::new(0.8, 0.8, 0.8),
      color_odd: ColorVector::new(0.0, 0.0, 0.0),
      material: BaseMaterial {
        reflection: 0.2,
        transparency: 0.0,
        gloss: 1.0,
        refraction: 0.0,
      },
      scale: 15.0,
    };

    if show_plane {
      shapes.push(Box::new(PlaneShape {
        position: plane_pos,
        d_val: plane_d_val,
        material: Arc::new(chess_mat),
        id,
      }));
      id = id + 1;
    }

    let mut lights: Vec<Box<Light>> = Vec::new();
    lights.push(Box::new(PointLight::new(
      PosVector::new(-5.0, 10.0, 10.0),
      ColorVector::new(0.8, 0.8, 0.8)
    )));

    lights.push(Box::new(PointLight::new(
      PosVector::new(5.0, 10.0, 10.0),
      ColorVector::new(0.8, 0.8, 0.8)
    )));

    let scene = Scene {
      background,
      shapes,
      lights,
      render_diffuse: true,
      render_reflection: true,
      render_refraction: true,
      render_shadow: true,
      render_highlights: true,
    };

    scene
  }
}
