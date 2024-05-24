mod sphere;
mod plane;
mod cylinder;

pub use sphere::*;
pub use cylinder::*;
pub use plane::*;

use reflect::*;
use reflect_glium::*;
use nalgebra::{SVector, Unit};
use std::error::Error;

use reflect::mirror::*;