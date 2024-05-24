use reflect::{
    mirror::{
        self,
        JsonDes,
        JsonType,
    },
    serde_json, util, Simulation,
};

use reflect_glium;

use reflect_mirrors::*;

use std::{collections::HashMap, error::Error, format as f, fs::File, sync::OnceLock};

trait SimulationMirror<const D: usize>: mirror::Mirror<D> + reflect_glium::OpenGLRenderable {}

impl<const D: usize, T: mirror::Mirror<D> + reflect_glium::OpenGLRenderable + ?Sized> SimulationMirror<D>
    for T
{
}

impl<const D: usize> JsonType for dyn SimulationMirror<D> {
    fn json_type() -> String {
        "dynamic".into()
    }
}

fn boxed<'a, const D: usize, T: SimulationMirror<D> + 'a>(
    mirror: T,
) -> Box<dyn SimulationMirror<D> + 'a> {
    Box::new(mirror)
}

type MirrorDeserializer<const D: usize> =
    fn(&serde_json::Value) -> Result<Box<dyn SimulationMirror<D>>, Box<dyn Error>>;

fn deserialize_boxed<const D: usize>(
    json: &serde_json::Value,
    deserializers: &HashMap<String, MirrorDeserializer<D>>,
) -> Result<Box<dyn SimulationMirror<D>>, Box<dyn Error>> {
    let mirror_type = json
        .get("type")
        .ok_or("Missing mirror type")?
        .as_str()
        .ok_or("type must be a string")?;

    let mirror_json = json.get("mirror").ok_or("Missing mirror data")?;

    let deserializer = deserializers
        .get(mirror_type.trim_start_matches("[]"))
        .ok_or(f!("invalid_mirror_type: {mirror_type}"))?;

    if mirror_type.starts_with("[]") {
        util::map_json_array(mirror_json, deserializer).map(boxed)
    } else {
        deserializer(mirror_json)
    }
}

impl mirror::JsonDes for Box<dyn SimulationMirror<2>> {
    /// Deserialize a new 2D simulation mirror object from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```json
    /// {
    ///     "type": "string",
    ///     "mirror": // <layout depends on the value at "type">
    /// }
    /// ```
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        static DESERIALIZERS: OnceLock<HashMap<String, MirrorDeserializer<2>>> = OnceLock::new();

        #[rustfmt::skip]
        let deserializers = DESERIALIZERS.get_or_init(|| HashMap::from([
            (
                // recurse
                <dyn SimulationMirror<2>>::json_type(),
                (|value| Box::<dyn SimulationMirror<2>>::from_json(value).map(boxed)) as MirrorDeserializer<2>,
            ),
            (
                PlaneMirror::<2>::json_type(),
                |value| PlaneMirror::<2>::from_json(value).map(boxed),
            ),
            (
                EuclideanSphereMirror::<2>::json_type(),
                |value| EuclideanSphereMirror::<2>::from_json(value).map(boxed),
            ),
        ]));

        deserialize_boxed(json, deserializers)
    }
}

// copy paste lol
impl mirror::JsonDes for Box<dyn SimulationMirror<3>> {
    /// Deserialize a new 3D simulation mirror object from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```json
    /// {
    ///     "type": "string",
    ///     "mirror": // <layout depends on the value at "type">
    /// }
    /// ```
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        static DESERIALIZERS: OnceLock<HashMap<String, MirrorDeserializer<3>>> = OnceLock::new();

        #[rustfmt::skip]
        let deserializers = DESERIALIZERS.get_or_init(|| HashMap::from([
            (
                // recurse
                <dyn SimulationMirror<3>>::json_type(),
                (|json| Box::<dyn SimulationMirror<3>>::from_json(json).map(boxed)) as MirrorDeserializer<3>,
            ),
            (
                PlaneMirror::<3>::json_type(),
                |json| PlaneMirror::<3>::from_json(json).map(boxed),
            ),
            (
                EuclideanSphereMirror::<3>::json_type(),
                |json| EuclideanSphereMirror::<3>::from_json(json).map(boxed),
            ),
            (
                CylindricalMirror::json_type(),
                |json| CylindricalMirror::from_json(json).map(boxed)
            )
        ]));

        deserialize_boxed(json, deserializers)
    }
}

fn run_simulation(reflection_cap: usize, json: &serde_json::Value) -> Result<(), Box<dyn Error>> {
    let dim = json
        .get("dim")
        .ok_or(r#"invalid json: expected a "dim" field"#)?
        .as_u64()
        .ok_or(r#""dim" field must be a number"#)?;

    match dim {
        2 => Simulation::<Box<dyn SimulationMirror<2>>, 2>::from_json(json)
            .map(|sim| reflect_glium::run_2d(sim, reflection_cap)),
        3 => Simulation::<Box<dyn SimulationMirror<3>>, 3>::from_json(json)
            .map(|sim| reflect_glium::run_3d(sim, reflection_cap)),
        _ => Err("dimension must be 2 or 3".into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("expected a file path as a first argument.")?;

    let max_num_reflections = args
        .next()
        .map(|arg| arg.parse().expect("expected a number as second argument"))
        .unwrap_or(1000);

    run_simulation(
        max_num_reflections,
        &serde_json::from_reader(File::open(file_path)?)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detection() {
        let simulation =
            Simulation::<Box<dyn SimulationMirror<3>>, 3>::from_json(&serde_json::json!(
                {
                    "mirror": {
                        "type": "[]plane",
                        "mirror": [
                            {
                                "center": [1., 0., 0.],
                                "basis": [
                                    [0., 1., 0.],
                                    [0., 0., 1.],
                                ],
                                "bounds": [1.,1.],
                            },
                            {
                                "center": [-1., 0., 0.],
                                "basis": [
                                    [0., 1., 0.],
                                    [0., 0., 1.],
                                ],
                                "bounds": [1.,1.],
                            }
                        ],
                    },
                    "rays": [
                        {
                            "origin": [0., 0., 0.],
                            "direction": [1., 0., 0.],
                        }
                    ],
                }
            ))
            .unwrap();

        let path = simulation.get_ray_paths(100);
        assert!(path.first().unwrap().all_points_raw().len() == 4);
    }

    #[test]
    fn test_no_loop_detection() {
        let simulation = Simulation::<Box<dyn SimulationMirror<2>>, 2>::from_json(
            &include_str!("../../assets/diamond_of_hell.json")
                .parse()
                .expect("invalid json in assets/diamond_of_hell.json"),
        )
        .unwrap();

        let path = simulation.get_ray_paths(100);
        assert!(path.first().unwrap().all_points_raw().len() == 101);
    }
}
