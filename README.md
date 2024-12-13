# `miroir`

A minimal yet powerful set of libraries for simulating ray reflection on arbitrary, user-defined shapes (curves in 2D, surfaces in 3D, etc...), in Rust.

Requires the latest stable version of the [Rust Compiler](https://www.rust-lang.org/tools/install).

Powered by [`nalgebra`](https://nalgebra.org/).

## Crates

The core of this project is the [`miroir`](miroir_core) crate, containing the main `Mirror` trait and primitives used to run simulations. This crate is `#[no_std]` and performs no allocations, making it possible to use it anywhere.

The [`miroir_shapes`](miroir_shapes) crate contains several example implementations of basic shapes that can be used in simulations. This is where you should look if you need an example of how to implement your own custom mirror shapes.

There are integrations extending the [`miroir`](miroir_core) library with more functionality such as:

- [`miroir_glium`](miroir_glium), which enables running and visualising 2D and 3D simulations using OpenGL.
- [`miroir_numworks`](miroir_numworks), which enables running 2D simulations and viewing them on the screen of a [Numworks Graphing Calculator](https://www.numworks.com/). This serves mainly as an example of [`miroir`](miroir_core) being used in a bare-metal environment.

Check out the READMEs in the respective folders for more information.
