#![no_std]

mod cylinder;
mod simplex;
mod sphere;

pub use cylinder::*;
pub use simplex::*;
pub use sphere::*;

use miroir::*;
use na::{ComplexField, RealField, SVector, Unit};
