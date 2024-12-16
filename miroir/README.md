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
    fn closest_intersection(&self, ray: &Ray) -> Option<Intersection>;
}
```

In this method, one must return the first point of intersection with `self`, that `ray` would meet when traveling straight forward.

This trait is object safe, and automatically implemented for arrays, slices, (mutable) references, `{Box/Rc/Arc/Vec}`s (when the `alloc` feature is enabled) and tuples if the underlying type(s) are also `Mirror`s, making combining, seperating, and sharing mirrors easy and intuitive.

## Documentation

For more information on how to use this crate, check out the docs:

```shell
cargo doc --no-deps --open
```
