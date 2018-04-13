use std::sync::Arc;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

use scene::{Background, Scene};
use camera::Camera;
use renderer::RenderData;
use shapes::*;
use material::*;
use light::*;
use color::ColorVector;
use posvector::PosVector;

pub struct NffParserResult {
  pub scene: Scene,
  pub render_data: RenderData,
  pub camera: Camera,
}

fn as_f64(s: &str) -> f64 {
  s.parse::<f64>().unwrap()
}

fn as_u32(s: &str) -> u32 {
  s.parse::<u32>().unwrap()
}

enum LookingFor {
  Instruction,
  ViewpointFrom,
  ViewpointAt,
  ViewpointUp,
  ViewpointAngle,
  ViewpointHither,
  ViewpointResolution,
  Polygon
}

// see: http://www.fileformat.info/format/nff/egff.htm
pub fn parse_nff_file(file_path: &str, num_threads: u32, ray_trace_depth: u32) -> NffParserResult {
  let mut shapes: Vec<Box<Shape>> = Vec::new();
  let mut lights: Vec<Box<Light>> = Vec::new();
  let mut camera_from = PosVector::new_default();
  let mut camera_at = PosVector::new_default();
  let mut camera_up = PosVector::new_default();
  let mut resolution_x = 1000;
  let mut resolution_y = 1000;

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


  // bottom plane:  green
  shapes.push(Box::new(PlaneShape {
    position: PosVector::new(0.0, 0.0, 1.0),
    d_val: 0.0,
    // material: Arc::new(SolidMaterial::new(0.0, 0.0, 0.0, 0.0, ColorVector::new(0.0, 1.0, 0.0))),
    material: Arc::new(chess_mat),
    id: 0,
  }));

  let mut background = Background::new(ColorVector::new(0.0, 0.0, 0.0), 0.0);

  let mut looking_for = LookingFor::Instruction;
  let mut current_material = SolidMaterial::new(0.0, 0.0, 0.0, 0.0, ColorVector::new(0.0, 0.0, 0.0));

  let mut current_shape_id = 1;
  let mut current_item_counter = 0;

  let f = File::open(file_path).unwrap();
  let file = BufReader::new(&f);
  for (num, line) in file.lines().enumerate() {
    let l = line.unwrap();

    match looking_for {
      LookingFor::Instruction => {
        let vec: Vec<&str> = l.split(" ").collect();

        let instruction = vec[0];

        if instruction == "b" {
          // background color
          // println!("reading background: {}", num);
          background = Background::new(
            ColorVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3])),
            0.0,
          );
        } else if instruction == "v" {
          // viewpoint location
          // println!("reading v: {}", num);
          looking_for = LookingFor::ViewpointFrom;
        } else if instruction == "l" {
          // println!("reading light: {}", num);
          // positional light

          let mut color_vec = ColorVector::new(1.0, 1.0, 1.0);

          if vec.len() == 7 {
            color_vec = ColorVector::new(
              as_f64(vec[4]),
              as_f64(vec[5]),
              as_f64(vec[6]),
            );
          }

          lights.push(Box::new(PointLight::new(
            PosVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3])), // start at 1 to skip instruction
            color_vec
          )));
        } else if instruction == "f" {
          // println!("reading f: {}", num);
          // object material properties
          // "f" red green blue Kd Ks Shine T index_of_refraction
          // Kd Diffuse component
          // Ks Specular
          // Shine Phong cosine power for highlights
          // T Transmittance (fraction of contribution of the transmitting ray).
          // Usually, 0 <= Kd <= 1 and 0 <= Ks <= 1, though it is not required that Kd + Ks = 1. Note that transmitting objects (T > 0) are considered to have two sides for algorithms that need these (normally, objects have one side).

          // todo: i don't think i'm assigning the correct values into my solidmaterial yet
          current_material = SolidMaterial::new(
            as_f64(vec[6]),
            as_f64(vec[5]),
            as_f64(vec[8]),
            as_f64(vec[7]),
            // as_f64(vec[6]),
            // as_f64(vec[4]),
            // as_f64(vec[5]),
            // as_f64(vec[7]),
            ColorVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3])),
          );
        } else if instruction == "c" {
          // cone or cylinder
          // println!("reading c: {}", num);
        } else if instruction == "s" {
          // println!("reading sphere: {}", num);
          // sphere
          // println!("{}, {}, {}", as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3]));
          shapes.push(Box::new(SphereShape {
            position: PosVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3])),
            radius: as_f64(vec[4]),
            material: Arc::new(current_material),
            id: current_shape_id,
          }));
          current_shape_id = current_shape_id + 1;
        } else if instruction == "p" {
          // println!("reading polygon: {}", num);
          // polygon
          current_item_counter = as_u32(vec[1]);
          looking_for = LookingFor::Polygon;
        } else if instruction == "pp" {
          // println!("reading polygon patch: {}", num);
          // polygon patch
        } else if instruction == "#" {
          // println!("reading comment: {}", num);
          // comment
        }
      }
      LookingFor::Polygon => {
        if current_item_counter > 0 {
          current_item_counter = current_item_counter - 1;
          // todo: parse polygon
        }

        if current_item_counter == 0 {
          looking_for = LookingFor::Instruction;
        }
      }
      LookingFor::ViewpointFrom => {
        // println!("reading viewpoint from: {}", num);
        let vec: Vec<&str> = l.split(" ").collect();
        // let instruction = vec[0];
        // todo: assert instruction == "from"
        camera_from = PosVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3]));
        looking_for = LookingFor::ViewpointAt;
      }
      LookingFor::ViewpointAt => {
        // println!("reading viewpoint at: {}", num);
        let vec: Vec<&str> = l.split(" ").collect();
        // let instruction = vec[0];
        // todo: assert instruction == "at"
        camera_at = PosVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3]));
        looking_for = LookingFor::ViewpointUp;
      }
      LookingFor::ViewpointUp => {
        // println!("reading viewpoint up: {}", num);
        let vec: Vec<&str> = l.split(" ").collect();
        // let instruction = vec[0];
        // todo: assert instruction == "at"
        camera_up = PosVector::new(as_f64(vec[1]), as_f64(vec[2]), as_f64(vec[3]));
        looking_for = LookingFor::ViewpointAngle;
      }
      LookingFor::ViewpointAngle => {
        // println!("reading viewpoint angle: {}", num);
        // todo: implement
        looking_for = LookingFor::ViewpointHither;
      }
      LookingFor::ViewpointHither => {
        // println!("reading viewpoint hither: {}", num);
        // todo: implement
        looking_for = LookingFor::ViewpointResolution;
      }
      LookingFor::ViewpointResolution => {
        // println!("reading viewpoint resolution: {}", num);
        let vec: Vec<&str> = l.split(" ").collect();
        // let instruction = vec[0];

        resolution_x = as_u32(vec[1]);
        resolution_y = as_u32(vec[2]);
        looking_for = LookingFor::Instruction;
      }
      _ => {}
    }
  }

  NffParserResult {
    scene: Scene {
      background,
      shapes,
      lights,
      render_diffuse: true,
      render_reflection: true,
      render_refraction: true,
      render_shadow: true,
      render_highlights: true,
    },
    render_data: RenderData {
      width: resolution_x,
      height: resolution_y,
      ray_trace_depth,
      num_threads,
      thread_per_line: true,
    },
    camera: Camera::new(camera_from, camera_at, camera_up, 50.0),
  }
}
