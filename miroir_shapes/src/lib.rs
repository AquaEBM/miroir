#![cfg_attr(not(feature = "glium"), no_std)]

mod cylinder;
mod simplex;
mod sphere;

pub use cylinder::*;
pub use simplex::*;
pub use sphere::*;

#[cfg(any(feature = "numworks", feature = "glium"))]
use num_traits::AsPrimitive;

#[cfg(feature = "numworks")]
use miroir_numworks::{self, eadk::kandinsky, ToPoint};

#[cfg(feature = "glium")]
use miroir_glium::*;

use miroir::*;
use na::{SVector, Unit, ComplexField, RealField};
