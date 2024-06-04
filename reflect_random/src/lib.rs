use reflect::*;

use nalgebra::{SVector, Unit};

use core::iter;
pub use rand;

pub trait Random: Sized {
    /// Generate a randomized version of this mirror using the provided `rng`
    ///
    /// This method must not fail. If creating a mirror is faillible, keep trying until success
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self;
}

impl<const D: usize> Random for Ray<D> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        let origin = rand_vect(rng, 7.0);

        let direction = loop {
            if let Some(v) = Unit::try_new(rand_vect(rng, 1.0), Float::EPSILON * 8.0) {
                break v;
            }
        };
        Self { origin, direction }
    }
}

pub fn random_simulation<const D: usize, M: Mirror<D> + Random>(rng: &mut (impl rand::Rng + ?Sized)) -> (M, Vec<Ray<D>>) {
    const MIN_NUM_RAYS: usize = 1;
    const MAX_NUM_RAYS: usize = 32;
    let num_rays = rng.gen_range(MIN_NUM_RAYS..MAX_NUM_RAYS);

    (
        M::random(rng),
        iter::repeat_with(|| Ray::random(rng))
            .take(num_rays)
            .collect(),
    )
}

pub fn rand_vect<const D: usize>(
    rng: &mut (impl rand::Rng + ?Sized),
    max_coord_mag: Float,
) -> SVector<Float, D> {
    // the rng generates floats in 0.0..1.0, scale and translate the range accordingly

    SVector::<Float, D>::from_fn(|_, _| (rng.gen::<Float>() - 0.5) * (max_coord_mag.abs() * 2.0))
}
