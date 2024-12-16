# `miroir_numworks`

Library enabling running ray reflection simulations in 2D using [`miroir`](../miroir_core) and viewing them on the screen of your [Numworks Graphing Calculator](https://www.numworks.com/), using [`eadk_rs`](https://github.com/AquaEBM/eadk_rs).

## Examples

Examples of simulations you can run (and how to create them) can be found in the [`examples`](examples) directory. Use the following commands to run one.

First, build the app with the following command:

```shell
cargo rustc build -r --example <example_name> --target=thumbv7em-none-eabihf -- -Clink-arg=--relocatable -Clink-arg=-no-gc-sections
```

Where `<example_name>` is the name of the example's source file, without the trailing `.rs`.

Then, check out the last step in the instructions in [`eadk_rs`](https://github.com/AquaEBM/eadk_rs) to upload the binary to your calculator (Note that, since this crate is part of a workspace, the `target` folder will be at the root of the whole repository).

Here are some videos of some simulations being run on the calculator:

[`trapped_circle.rs`](examples/trapped_circle.rs) (Slowed down, with a timing parameter, for visibility)
https://github.com/AquaEBM/miroir/assets/79016373/e7fd62c3-1bdd-4d6c-a17b-de3517f60b39

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

miroir_numworks = { git = "https://github.com/AquaEBM/miroir", features = ["alloc"] }
#          only add this if you have an allocator configured ^^^^^^^^^^^^^^^^^^^^^^
```

This crate already re-exports [`miroir`](../miroir_core) and [`eadk_rs`](https://github.com/AquaEBM/eadk_rs) for convenience, and to avoid dependency synchronisation issues.
