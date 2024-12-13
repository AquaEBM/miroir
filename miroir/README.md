# `miroir`

This crate is the core of this project, all the other crates depend on it.

To include it as a dependency in your project, just add it to the `[dependencies]` section of your `Cargo.toml` file.

```toml
# ...
[dependencies]
# ...

miroir = { git = "https://github.com/AquaEBM/miroir" }
```

This crate already re-exports [`nalgebra`](https://crates.io/crates/nalgebra), for convenience reasons, and to avoid dependency synchronisation issues.

## Overview

The main idea behind this crate is the following one-method trait: (some parts omitted)

```rust
pub trait Mirror {
    fn add_tangents(&self, ctx: &mut SimulationCtx);
}
```

In this method, `self` must report to `ctx` the distance(s) a given ray (accessible with `ctx.ray()`) must travel to reach a point of intersection with `self`, as well as the direction space(s) of the tangent(s) to `self` at said point(s).

This trait is object safe, and automatically implemented for arrays, slices, (mutable) references, `{Box/Rc/Arc/Vec}`s (when the `alloc` feature is enabled) and tuples if the underlying type(s) are also `Mirror`s, making combining, seperating, sharing, and composing mirrors easy and intuitive.

The `Ray` struct has a method `ray.closest_intersection(&mirror, ..)` that queries `mirror` and finds the closest one of said tangents.

Finally, the `RayPath` struct is an iterator of `Ray`s, built from a ray and a mirror, that calls the aforementioned method, moves the ray forward to the closest tangent, reflects it's direction w.r.t. the tangents direction space, then yields it, repeatedly, unitl no intersections between the ray and the mirror are found.

## Documentation

For more information on how to use this crate, check out the docs:

```shell
cargo doc --no-deps --open
```
