#![cfg_attr(not(feature = "miroir_glium"), no_std)]

mod cylinder;
mod simplex;
mod sphere;

pub use cylinder::*;
pub use simplex::*;
pub use sphere::*;

#[cfg(feature = "miroir_numworks")]
use miroir_numworks::{*, eadk::kandinsky::*};

#[cfg(feature = "miroir_glium")]
use miroir_glium::*;

use miroir::*;
use nalgebra::{SVector, Unit, ComplexField, RealField};
use num_traits::AsPrimitive;
