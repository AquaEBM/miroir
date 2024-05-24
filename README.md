# `reflect`

A minimal yet powerful library for ray reflection simulation in Rust.

Requires the latest stable version of the [Rust Compiler](https://www.rust-lang.org/tools/install)

Uses [nalgebra](https://nalgebra.org/) for general computation.

## Crates

The core of this project is the `reflect` crate, containing the main `Mirror` trait and simulation logic

The `reflect_mirrors` crate contains several example implementations of reflective surfaces that can be used in simulations. This is where you should look if you need an example of how to implement your own custom mirror shapes.

There are integrations extending this library with more functionality such as:

- `reflect_glium` Which enables running and visualising 2D and 3D simulations using OpenGL.
- `reflect_json` Which enables serialisation/deserialisation of simulation data with the JSON format. Some example simulations in their JSON representation can be found in the `assets` directory.
- `reflect_random` Which exposes a simple trait for generating reflective surfaces randomly, for quick and dirty testing.

Other third-party integrations can easily be created over the simple API of the `reflect` crate. It is advised to check it's documentation:

```shell
cargo doc -p reflect --no-deps --open
```

The binary crate `gen_rand_sim` can generate random simulations and serialise to json, using `reflect_json` and `reflect_random`:

```shell
cargo run -r -p gen_rand_sim "<path/to/file.json>" [dimensions=2] [num_mirrors=12] [num_rays=4]
```

The binary crate `run_sim_json` can deserialise, run, then view simulations using `reflect_glium` and `reflect_json`:

```shell
cargo run -r -p run_sim_json "<path/to/simulation.json>" [max_reflection_count=1000]
```

## Flutter GUI App

The `mirror_verse_ui` GUI App, built with [Flutter](https://flutter.dev/), serves as a simple way to view the simulation JSON files in the `assets` directory, run them with `run_sim_json`, as well as generate ones randomly with `gen_rand_sim`, all in one single place. Here's how to run it:

First, build the binaries and move them to the Flutter `assets` directory:

- Linux/MacOS

```shell
cargo build -r && \
cp target/release/generate_random_simulation_3d mirror_verse_ui/assets && \
cp target/release/run_simulation_json_3d mirror_verse_ui/assets
```

- Windows:

```shell
cargo build -r
copy "target\release\generate_random_simulation_3d.exe mirror_verse_ui\assets"
copy "target\release\run_simulation_json_3d.exe mirror_verse_ui\assets"
```

You can now run the app with:

```shell
cd mirror_verse_ui
flutter run --release
```

### Controls for `reflect_glium`

The `reflect_glium` binary crate (which is run by the `mirror_verse_ui`) allows viewing simulations where you can move around and rotate the camera. Here are the controls:

- Use the WASD keys (or ZQSD) to move forward, left, backward, and right, respectively.
- Use the space bar to move up and the shift key to move down.
- Click and drag your mouse on the screen to look around, and rotate the camera.
- Use the right/left arrow key to increase/decrease camera rotation sensitivity.
- Use the up/down key to increase/decrease movement speed.
