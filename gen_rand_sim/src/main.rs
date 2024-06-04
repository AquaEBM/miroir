use core::{iter, ops::Deref};
use std::{env, error::Error, fs::File};

use reflect_json::{serde_json, JsonSer};
use reflect_mirrors::*;
use reflect_random::*;

trait JsonTypeDyn {
    fn json_type_dyn(&self) -> String;
}

impl<T: reflect_json::JsonType + ?Sized> JsonTypeDyn for T {
    fn json_type_dyn(&self) -> String {
        Self::json_type()
    }
}

trait JsonSerDyn: JsonSer + JsonTypeDyn {}

impl<T: JsonSer + JsonTypeDyn> JsonSerDyn for T {}

struct Dynamic<T, const D: usize>(T);

impl Random for Dynamic<Box<dyn JsonSerDyn>, 2> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        Self(match rng.gen_range(0usize..2) {
            0 => Box::new(PlaneMirror::<2>::random(rng)) as Box<dyn JsonSerDyn>,
            1 => Box::new(EuclideanSphereMirror::<2>::random(rng)),
            _ => unreachable!(),
        })
    }
}

impl Random for Dynamic<Box<dyn JsonSerDyn>, 3> {
    fn random(rng: &mut (impl rand::Rng + ?Sized)) -> Self {
        Self(match rng.gen_range(0usize..3) {
            0 => Box::new(PlaneMirror::<3>::random(rng)) as Box<dyn JsonSerDyn>,
            1 => Box::new(EuclideanSphereMirror::<3>::random(rng)),
            2 => Box::new(CylindricalMirror::random(rng)),
            _ => unreachable!(),
        })
    }
}

impl<T: Deref, const D: usize> JsonSer for Dynamic<T, D>
where
    T::Target: JsonTypeDyn + JsonSer,
{
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": self.0.deref().json_type_dyn(),
            "data": self.0.deref().to_json(),
        })
    }
}

impl<T, const D: usize> reflect_json::JsonType for Dynamic<T, D> {
    fn json_type() -> String {
        "dynamic".into()
    }
}

pub fn gen_rand_mirrors<T: Random>(n: usize, rng: &mut (impl rand::Rng + ?Sized)) -> Vec<T> {
    iter::repeat_with(|| T::random(rng)).take(n).collect()
}

fn generate_random_simulation(
    dim: usize,
    num_mirrors: usize,
    num_rays: usize,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    if dim == 2 {

        Ok(reflect_json::serialize_simulation(
            &Dynamic::<_, 2>(gen_rand_mirrors::<Dynamic<Box<dyn JsonSerDyn>, 2>>(
                num_mirrors,
                &mut rng,
            )),
            iter::repeat_with(|| reflect::Ray::<2>::random(&mut rng)).take(num_rays)
        ))
    } else if dim == 3 {
        Ok(reflect_json::serialize_simulation(
            &Dynamic::<_, 3>(gen_rand_mirrors::<Dynamic<Box<dyn JsonSerDyn>, 3>>(
                num_mirrors,
                &mut rng,
            )),
            iter::repeat_with(|| reflect::Ray::<3>::random(&mut rng)).take(num_rays)
        ))
    } else {
        Err("dimension must be 2 or 3".into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("please provide a path to serialize the simulation json data")?;

    let dim = args.next().and_then(|arg| arg.parse().ok()).unwrap_or(2);

    let num_mirrors = args.next().and_then(|arg| arg.parse().ok()).unwrap_or(12);

    let num_rays = args.next().and_then(|arg| arg.parse().ok()).unwrap_or(4);

    let json = generate_random_simulation(dim, num_mirrors, num_rays)?;

    serde_json::to_writer_pretty(File::create(file_path)?, &json)?;

    Ok(())
}
