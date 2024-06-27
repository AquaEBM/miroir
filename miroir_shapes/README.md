# `miroir_shapes`

Library extending [`miroir`](../miroir_core) with some rudimentary shapes you can easily use in simulations.

To include this crate as a dependency in your project, just add it to the `[dependencies]` section of your `Cargo.toml` file.

```toml
# ...
[dependencies]
# ...

miroir = { git = "https://github.com/AquaEBM/miroir" }
# ^^^^ Make sure to include this dependency as well
miroir_shapes = { git = "https://github.com/AquaEBM/miroir" }
```

## Currently implemented shapes

- (Hyper)Spheres, in any dimension `n`.
- `n-1`-Simplexes in any dimension `n` (i. e. line segments in the plane, triangles in space, tetrahedrons in 4D space, etc...)
- Cylinders (open and right), represented as a line segment (two points) and a radius, in 3D space.
