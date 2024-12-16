# `miroir_glium`

Library enabling running ray reflection simulations using [`miroir`](../miroir_core) and visualising them in 2D/3D using OpenGL, through [`glium`](https://crates.io/crates/glium/).

## Examples

Examples of simulations you can run (and how to create them) can be found in the [`examples`](examples) directory. Use the following command to run one.

```shell
cargo run -r --example <example_name>
```

Where `<example_name>` is the name of the example's source file, without the trailing `.rs`.

Here are screenshots of some of the examples' visualisations:

[`trapped_with_sphere.rs`](examples/trapped_with_sphere.rs)
![trapped_in_sphere](https://github.com/AquaEBM/miroir/assets/79016373/53693e8c-993f-4919-979e-5c4ca0931ded)

[`cynlinder.rs`](examples/cylinder.rs)
![image](https://github.com/AquaEBM/miroir/assets/79016373/05abbd0d-7268-4bbe-af7b-b9e195bab3bc)

## Controls

- Use the WASD keys (or ZQSD) to move forward, left, backward, and right, respectively.
- Use the space bar to move up and the shift key to move down.
- Click and drag your mouse on the screen to look around, and rotate the camera.
- Use the right/left arrow key to increase/decrease camera rotation sensitivity.
- Use the up/down key to increase/decrease movement speed.

## Documentation

For more information on how to use this crate, check out the docs:

```shell
cargo doc --no-deps --open
```

To include this crate as a dependency in your project, just add it to the `[dependencies]` section of your `Cargo.toml` file.

```toml
# ...
[dependencies]
# ...

miroir_glium = { git = "https://github.com/AquaEBM/miroir" }
```

This crate already re-exports [`glium`](https://crates.io/crates/glium/) for convenience, and to avoid dependency synchronisation issues.

### TODOs

- 3D simulations lack any kind of lighting, hence, making viewing complex, curved, surfaces awkward. I am not (yet?) well-versed enough in 3D rendering to know how to implement this neatly.
- This uses a slightly older version of [`glium`](https://crates.io/crates/glium/), because we rely on [`glium_shapes`](https://crates.io/crates/glium_shapes) to create the vertices of a 3D sphere. Hand-roll our own implementation, bump the dependency, and adapt to API changes.
