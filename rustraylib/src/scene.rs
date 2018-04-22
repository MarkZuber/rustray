use std::sync::Arc;
use std::collections::HashMap;
use elapsed::measure_time;

use material::{BaseMaterial, ChessboardMaterial, Material, SolidMaterial};
use light::{Light, PointLight};
use shapes::{BoundingBox, PlaneShape, Shape, SphereShape};
use posvector::PosVector;
use color::ColorVector;
use tracer::IntersectionInfo;
use camera::Ray;

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
pub struct CompiledShape {
  shape: Arc<Box<Shape>>,
  id: u32,
  bbox: Arc<Box<BoundingBox>>,
}

impl CompiledShape {
  pub fn new(shape: Arc<Box<Shape>>, id: u32) -> CompiledShape {
    let bbox = Arc::new(Box::new(BoundingBox::from_shape(&shape)));

    CompiledShape { shape, id, bbox }
  }

  pub fn get_position(&self) -> PosVector {
    self.shape.get_position()
  }

  pub fn intersect(&self, ray: &Ray) -> IntersectionInfo {
    let mut intersect_info = self.shape.intersect(ray);
    intersect_info.element_id = self.get_id();
    intersect_info
  }

  pub fn get_material(&self) -> Arc<Material> {
    self.shape.get_material()
  }

  pub fn get_id(&self) -> u32 {
    self.id
  }

  pub fn get_bounding_box(&self) -> Arc<Box<BoundingBox>> {
    self.bbox.clone()
  }
}

#[derive(Debug)]
pub struct CompiledLight {
  pub light: Arc<Box<Light>>,
  pub id: u32
}

impl CompiledLight {
  pub fn new(light: Arc<Box<Light>>, id: u32) -> CompiledLight {
    CompiledLight { light, id }
  }
}

impl CompiledLight {
  pub fn get_position(&self) -> PosVector {
    self.light.get_position()
  }
  pub fn get_color(&self) -> ColorVector {
    self.light.get_color()
  }
}

#[derive(Debug)]
pub struct Scene {
  pub background: Background,
  pub shapes: HashMap<u32, Box<CompiledShape>>,
  pub lights: HashMap<u32, Box<CompiledLight>>,
}

unsafe impl Send for Scene {}
unsafe impl Sync for Scene {}

impl Scene {
  pub fn new(background: Background, shapes: Vec<Box<Shape>>, lights: Vec<Box<Light>>) -> Scene {
    let mut compiled_shapes: HashMap<u32, Box<CompiledShape>> = HashMap::new();
    let mut compiled_lights: HashMap<u32, Box<CompiledLight>> = HashMap::new();

    let (elapsed, _) = measure_time(|| {
      let mut current_shape_id: u32 = 1;


      for shape in shapes {
        compiled_shapes.insert(current_shape_id, Box::new(CompiledShape::new(
          Arc::new(shape),
          current_shape_id,
        )));
        current_shape_id = current_shape_id + 1;
      }

      let mut current_light_id: u32 = 1;

      for light in lights {
        compiled_lights.insert(current_light_id, Box::new(CompiledLight::new(
          Arc::new(light),
          current_light_id
        )));
        current_light_id = current_light_id + 1;
      }
    });
    println!("compile time = {:?}ms", elapsed.millis());

    Scene {
      background,
      shapes: compiled_shapes,
      lights: compiled_lights,
    }
  }

  pub fn get_shape(&self, id: &u32) -> Option<&Box<CompiledShape>> {
    self.shapes.get(id)
  }

  pub fn get_light(&self, id: &u32) -> Option<&Box<CompiledLight>> {
    self.lights.get(id)
  }
}

pub fn new_basic_scene() -> Scene {
  let background = Background::new(ColorVector::new(0.0, 0.0, 0.0), 0.2);

  let mut shapes: Vec<Box<Shape>> = Vec::new();

  // right most sphere: purple
  shapes.push(Box::new(SphereShape {
    position: PosVector::new(2.5, 5.0, 1.0),
    radius: 0.75,
    material: Arc::new(SolidMaterial::new(
      0.0,
      0.0,
      0.0,
      0.0,
      ColorVector::new(1.0, 0.0, 1.0),
    )),
    id: 1,
  }));

  // left most sphere: red
  shapes.push(Box::new(SphereShape {
    position: PosVector::new(3.5, 1.25, 1.5),
    radius: 1.0,
    material: Arc::new(SolidMaterial::new(
      0.0,
      0.0,
      0.0,
      0.0,
      ColorVector::new(1.0, 1.0, 0.0),
    )),
    id: 2,
  }));

  // middle sphere: cyan
  shapes.push(Box::new(SphereShape {
    position: PosVector::new(2.0, 3.0, 1.0),
    radius: 1.0,
    material: Arc::new(SolidMaterial::new(
      0.0,
      0.0,
      0.0,
      0.0,
      ColorVector::new(0.0, 1.0, 1.0),
    )),
    id: 3,
  }));

  // bottom plane:  green
  shapes.push(Box::new(PlaneShape {
    position: PosVector::new(0.0, 0.0, 1.0),
    d_val: 0.0,
    material: Arc::new(SolidMaterial::new(
      0.0,
      0.0,
      0.0,
      0.0,
      ColorVector::new(0.0, 1.0, 0.0),
    )),
    id: 4,
  }));

  // right plane:  blue
  shapes.push(Box::new(PlaneShape {
    position: PosVector::new(1.0, 0.0, 0.0),
    d_val: 0.0,
    material: Arc::new(SolidMaterial::new(
      0.0,
      0.0,
      0.0,
      0.0,
      ColorVector::new(0.0, 0.0, 1.0),
    )),
    id: 5,
  }));

  // left plane: red
  shapes.push(Box::new(PlaneShape {
    position: PosVector::new(0.0, 1.0, 0.0),
    d_val: 0.0,
    material: Arc::new(SolidMaterial::new(
      0.0,
      0.0,
      0.0,
      0.0,
      ColorVector::new(1.0, 0.0, 0.0),
    )),
    id: 6,
  }));

  let mut lights: Vec<Box<Light>> = Vec::new();
  lights.push(Box::new(PointLight::new(
    PosVector::new(100.0, 60.0, 40.0),
    ColorVector::new(1.0, 1.0, 1.0),
  )));

  Scene::new(background, shapes, lights)
}

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
  let red_material = SolidMaterial::new(2.0, 0.2, 0.0, 0.0, ColorVector::new(0.9, 0.0, 0.0));
  let green_material = SolidMaterial::new(2.0, 0.2, 0.0, 0.0, ColorVector::new(0.0, 0.9, 0.0));

  let max_axis = sphere_distance_increment * num_spheres_per_axis as f64;

  let mut shapes: Vec<Box<Shape>> = Vec::new();

  let mut x = 0.0;
  let mut id = 1;
  while x <= max_axis {
    shapes.push(Box::new(SphereShape {
      position: PosVector::new(x, 0.0, 0.0),
      radius: sphere_radius,
      material: Arc::new(red_material),
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
      material: Arc::new(blue_material),
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

    // todo: need to move this into scene
    // so we're not manually managing this
    // id = id + 1;
  }

  let mut lights: Vec<Box<Light>> = Vec::new();
  lights.push(Box::new(PointLight::new(
    PosVector::new(-5.0, 10.0, 10.0),
    ColorVector::new(0.4, 0.4, 0.4),
  )));

  lights.push(Box::new(PointLight::new(
    PosVector::new(5.0, 10.0, 10.0),
    ColorVector::new(0.4, 0.4, 0.4),
  )));

  Scene::new(background, shapes, lights)
}
