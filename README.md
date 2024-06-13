# `reflect`

A minimal yet powerful library for ray reflection simulation in Rust.

Requires the latest stable version of the [Rust Compiler](https://www.rust-lang.org/tools/install).

Powered by [`nalgebra`](https://nalgebra.org/).

## Crates

The core of this project is the [`reflect`](reflect) crate, containing the main `Mirror` trait and primitives used to run simulations. This crate is `#[no_std]` and performs no allocations, making it possible to use it anywhere.

The [`reflect_mirrors`](reflect_mirrors) crate contains several example implementations of reflective surfaces that can be used in simulations. This is where you should look if you need an example of how to implement your own custom mirror shapes.

There are integrations extending this library with more functionality such as:

- [`reflect_glium`](reflect_glium) Which enables running and visualising 2D and 3D simulations using OpenGL.

...more to come soon.

Other third-party integrations can easily be created over the simple API of the [`reflect`](reflect) crate. It is advised to check it's documentation:

```shell
cargo doc -p reflect --no-deps --open
```

### Controls for `reflect_glium`

The [`reflect_glium`](reflect_glium) binary crate allows viewing simulations where you can move around and rotate the camera. Here are the controls:

- Use the WASD keys (or ZQSD) to move forward, left, backward, and right, respectively.
- Use the space bar to move up and the shift key to move down.
- Click and drag your mouse on the screen to look around, and rotate the camera.
- Use the right/left arrow key to increase/decrease camera rotation sensitivity.
- Use the up/down key to increase/decrease movement speed.

Currently, the ray's path is drawn in white, and the portion of the path that loops infinitely (if it exists) is drawn in pink.

Examples of simulations you can run (and how to create them) can be found in the [`reflect_glium/examples`](reflect_glium/examples) directory. Use the following command to run one.

```shell
cargo run -r -p reflect_glium --example <example_name>
```

where `<example_name>` is the name of the example's source file (without the trailing `.rs`)
