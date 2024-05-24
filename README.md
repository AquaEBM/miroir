# ✨ reflect ✨

Light ray reflection simulation with 3D rendering.

Built with [Rust](https://www.rust-lang.org/), using [nalgebra](https://nalgebra.org/) for general computation, and [glium](https://github.com/glium/glium) for graphical rendering.

GUI app built with [Flutter](https://flutter.dev/)

This project is split into four main parts:

1. 📚 The library which contains the core simulation engine, and API for creating new mirrors (`mirror_verse`).
2. 🏃‍♂️ A program that runs a visualisation of a simulation, from it's JSON representation. (see `assets`)
3. 🔀 A program that generates simulations randomly and serialises them to JSON files.
4. 🖥️ A Flutter GUI app enabling users to run these tools without the terminal.

## GUI

First, make sure you have [Rust](https://www.rust-lang.org/) and [Flutter](https://flutter.dev/) installed on your machine.

### 🛠️ Compilation

Build the Rust project and move the emitted executables into the Flutter project's `assets` directory:

#### Windows

```shell
cargo build -r
copy "target\release\generate_random_simulation_3d.exe mirror_verse_ui\assets"
copy "target\release\run_simulation_json_3d.exe mirror_verse_ui\assets"
```

#### For Linux/MacOS

```shell
cargo build -r && \
cp target/release/generate_random_simulation_3d mirror_verse_ui/assets && \
cp target/release/run_simulation_json_3d mirror_verse_ui/assets
```

### 🚀 Running the UI

```shell
cd mirror_verse_ui
flutter run --release
```

## CLI

Installing [Flutter](https://flutter.dev/) is not necessary to run/generate simulations from the command line.

### 🔬 Running a simulation from a JSON file

```shell
cargo run --release -p run_sim_json "<path/to/simulation.json>" [max_reflection_count]
```

#### Controls

You can use the following controls during the visualisation of a simulation:

- Use the ZQSD keys (or WASD) to move forward, backward, left, and right, respectively.
- Use the space bar to move up and the shift key to move down.
- Click and drag your mouse on the screen to look around, and rotate the camera.
- Use the right/left arrow key to increase/decrease camera movement sensitivity.
- Use the up/down key to increase/decrease physical movement speed.

### 🔄 Generating a random simulation

```shell
cargo run --release -p gen_rand_sim "<path/to/output.json>" [dimension, default=2] [num_mirrors, default=12] [num_rays, default=4]
```

## Contributing

There are many ways to contribute to this project:

- Creating unit tests
- Adding documentation
- Raising an issue on Github for bugfixes or suggestions
- Adding new mirror shapes to the library!

### Implementing new Mirror Shapes

First, it is advised to check the documentation, specifically the `mirror_verse` module, using:

```shell
cargo doc --no-deps -p mirror_verse --open
```

Most mirror implementations, with all of their functionalities, are simple and compact (< 200 sloc), so you can easily browse the already implemented mirrors in the `mirror_verse/src/mirror` directory, if you need examples.

## Note

This project is currently undergoing (yet another hehe) major refactor, in which the core `Mirror` trait and simulation engine will be seperated into their own crate `mirror_verse` from other exposed functionalities, that will, then, be seen as extensions/integrations, in order to allow other users to seemlessly extend this crate into the broader Rust ecosystem.
