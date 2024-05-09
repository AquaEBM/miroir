use std::{env, error::Error, fs::File};

use mirror_verse::{rand, serde_json};

const DIM: usize = 3;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("please provide a path to serialize the simulation json data")?;

    let mut rng = rand::thread_rng();

    let random_simulation = mirror_verse::Simulation::<
        Vec<Box<dyn mirror_verse::mirror::Mirror<DIM>>>,
        DIM,
    >::random(&mut rng);

    let json = random_simulation.to_json()?;

    serde_json::to_writer_pretty(File::create_new(file_path)?, &json)?;

    Ok(())
}