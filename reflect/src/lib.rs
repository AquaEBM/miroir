extern crate alloc;

// re-export deps for convenience
pub mod mirror;
pub use nalgebra;
pub use rand;
pub use serde_json;

use core::iter;
use nalgebra::{SMatrix, SVector, Unit};
use std::error::Error;

use mirror::{JsonDes, JsonSer};

pub type Float = f64;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RayPath<const D: usize> {
    points: Vec<SVector<Float, D>>,
    loop_start: Option<usize>,
    divergence_direction: Option<Unit<SVector<Float, D>>>,
}

impl<const D: usize> RayPath<D> {
    pub fn all_points_raw(&self) -> &[SVector<Float, D>] {
        self.points.as_slice()
    }

    /// returns a pair (non_loop_points, loop_points)
    pub fn all_points(&self) -> (&[SVector<Float, D>], &[SVector<Float, D>]) {
        self.points
            .split_at(self.loop_start.unwrap_or(self.points.len()))
    }

    // name bikeshedding welcome

    pub fn non_loop_points(&self) -> &[SVector<Float, D>] {
        &self.points[..self.loop_start.unwrap_or(self.points.len())]
    }

    pub fn loop_points(&self) -> &[SVector<Float, D>] {
        self.loop_start
            .map(|index| &self.points[index..])
            .unwrap_or_default()
    }

    pub fn divergence_direction(&self) -> Option<&Unit<SVector<Float, D>>> {
        self.divergence_direction.as_ref()
    }

    pub fn push_point(&mut self, pt: SVector<Float, D>) {
        self.points.push(pt);
    }

    pub fn causes_loop_at(&self, pt: SVector<Float, D>, epsilon: Float) -> Option<usize> {
        self.points.split_last().and_then(|(last_pt, points)| {
            points.windows(2).enumerate().find_map(|(i, window)| {
                // ugly, but `slice::array_windows` is unstable
                let [this_pt, next_pt] = window else {
                    // because window.len() is always 2
                    unreachable!()
                };
                ((last_pt - this_pt).norm() <= epsilon && (pt - next_pt).norm() < epsilon)
                    .then_some(i)
            })
        })
    }

    /// Attempts to push a point to the path. If it causes an infinite loop, aborts,
    /// registers the section of the path that loops, and returns `false`
    pub fn try_push_point(&mut self, pt: SVector<Float, D>, epsilon: Float) -> bool {
        let maybe_loop_index = self.causes_loop_at(pt, epsilon);

        if let Some(loop_index) = maybe_loop_index {
            self.loop_start = Some(loop_index);
        } else {
            self.push_point(pt);
        }

        maybe_loop_index.is_none()
    }

    pub fn set_divergence_direction(&mut self, dir: Unit<SVector<Float, D>>) -> bool {
        let first_time = self.divergence_direction.is_none();
        self.divergence_direction = Some(dir);
        first_time
    }
}

pub struct Simulation<T, const D: usize> {
    pub rays: Vec<mirror::Ray<D>>,
    pub mirror: T,
}

impl<T: mirror::Random, const D: usize> mirror::Random for Simulation<T, D> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self
    where
        Self: Sized,
    {
        const MIN_NUM_RAYS: usize = 1;
        const MAX_NUM_RAYS: usize = 32;
        let num_rays = rng.gen_range(MIN_NUM_RAYS..MAX_NUM_RAYS);
        Self {
            rays: iter::repeat_with(|| mirror::Ray::random(rng))
                .take(num_rays)
                .collect(),
            mirror: T::random(rng),
        }
    }
}

impl<const D: usize, T: mirror::JsonDes> JsonDes for Simulation<T, D> {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        let mirror = T::from_json(json.get("mirror").ok_or("mirror field expected")?)?;

        let rays = util::map_json_array(
            json.get("rays").ok_or("ray field expected")?,
            mirror::Ray::from_json,
        )?;

        Ok(Self { mirror, rays })
    }
}

impl<const D: usize, T: mirror::JsonSer> JsonSer for Simulation<T, D> {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "dim": D,
            "rays": Vec::from_iter(self.rays.iter().map(mirror::Ray::to_json)),
            "mirror": self.mirror.to_json(),
        })
    }
}

impl<const D: usize, T: mirror::Mirror<D>> Simulation<T, D> {
    pub fn get_ray_paths(&self, reflection_limit: usize) -> Vec<RayPath<D>> {
        let mut intersections_scratch = vec![];
        self.rays
            .iter()
            .map(|ray| {
                let mut ray = *ray;
                let mut ray_path = RayPath::default();
                ray_path.push_point(ray.origin);

                for _n in 0..reflection_limit {
                    intersections_scratch.clear();
                    self.mirror.append_intersecting_points(
                        &ray,
                        util::List::new(&mut intersections_scratch),
                    );

                    if let Some((distance, tangent)) = intersections_scratch
                        .iter()
                        .filter_map(|tangent| {
                            let d = tangent
                                .try_ray_intersection(&ray)
                                .expect("a mirror returned a plane parallel to the ray: aborting");
                            (d > Float::EPSILON * 64.0).then_some((d, tangent))
                        })
                        .min_by(|(d1, _), (d2, _)| {
                            d1.partial_cmp(d2)
                                .expect("NaN found in intersection distances: aborting")
                        })
                    {
                        ray.advance(distance);
                        if !ray_path.try_push_point(ray.origin, Float::EPSILON * 16.0) {
                            break;
                        }
                        ray.reflect_dir(&tangent.direction)
                    } else {
                        ray_path.set_divergence_direction(ray.direction);
                        break;
                    }
                }
                ray_path
            })
            .collect()
    }
}

pub mod util {
    use alloc::collections::TryReserveError;

    use super::*;

    pub fn rand_vect<const D: usize>(
        rng: &mut (impl rand::Rng + ?Sized),
        max_coord_mag: Float,
    ) -> SVector<Float, D> {
        // the rng generates floats in 0.0..1.0, scale and translate the range accordingly

        SVector::<Float, D>::from_fn(|_, _| {
            (rng.gen::<Float>() - 0.5) * (max_coord_mag.abs() * 2.0)
        })
    }

    /// This is essentially `try_into` then `try_map` but the latter is nightly-only
    pub fn json_array_to_float_array<const D: usize>(
        json_array: &[serde_json::Value],
    ) -> Option<[Float; D]> {
        let array: &[serde_json::Value; D] = json_array.try_into().ok()?;

        let mut center_coords_array = [0.; D];
        for (coord, value) in center_coords_array.iter_mut().zip(array) {
            *coord = value.as_f64()? as Float;
        }
        Some(center_coords_array)
    }

    /// This is essentially `try_into` then `try_map` but the latter is nightly-only
    pub fn json_array_to_vector<const D: usize>(
        json_array: &[serde_json::Value],
    ) -> Option<SVector<Float, D>> {
        json_array_to_float_array(json_array).map(SVector::from)
    }

    pub fn map_json_array<T>(
        json: &serde_json::Value,
        map: impl FnMut(&serde_json::Value) -> Result<T, Box<dyn Error>>,
    ) -> Result<Vec<T>, Box<dyn Error>> {
        json.as_array()
            .ok_or("json value must be an array")?
            .iter()
            .map(map)
            .collect()
    }

    pub struct List<'a, T>(&'a mut Vec<T>);

    impl<'a, T> List<'a, T> {
        pub fn reborrow(&mut self) -> List<T> {
            List(self.0)
        }

        pub fn new(list: &'a mut Vec<T>) -> Self {
            Self(list)
        }

        pub fn capacity(&self) -> usize {
            self.0.capacity()
        }

        pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
            self.0.try_reserve(additional)
        }

        pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
            self.0.try_reserve_exact(additional)
        }

        pub fn reserve(&mut self, additional: usize) {
            self.0.reserve(additional)
        }

        pub fn reserve_exact(&mut self, additional: usize) {
            self.0.reserve_exact(additional)
        }

        pub fn push(&mut self, v: T) {
            self.0.push(v)
        }

        pub fn append(&mut self, vec: &mut Vec<T>) {
            self.0.append(vec)
        }

        pub fn extend_from_slice(&mut self, slice: &[T])
        where
            T: Clone,
        {
            self.0.extend_from_slice(slice)
        }
    }

    impl<'a, T> Extend<T> for List<'a, T> {
        fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            self.0.extend(iter)
        }
    }

    impl<'a, T> From<&'a mut Vec<T>> for List<'a, T> {
        fn from(value: &'a mut Vec<T>) -> Self {
            Self(value)
        }
    }
}
