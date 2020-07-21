# rsiup

Experimental and incomplete dynamic Rust bindings to the
[IUP](https://www.tecgraf.puc-rio.br/iup/) library.

An example which shows the bindings in use including callbacks is
[helloiup](https://github.com/mark-summerfield/helloiup).

To try the example, download this library and the example and put them
side-by-side in parallel directories, e.g., `parent/iup/` and
`parent/helloiup`. Then `cd` into `helloiup` and do `cargo run --release`.
This will fail the first time due to missing libraries. Copy (or on Unix
soft-link) the `iup/iup` directory to `helloiup/target/release/` and this
time it should build and run. Or, if you have Python 3 installed, use the
`run.py` script. (If it doesn't build make sure you changed the `rsiup`
directory to `iup` _or_ fix the `path` used in `helloiup`'s `Cargo.toml`
file. Note also that the provided `.so`s and `.dll`s are for 64-bit
systems.)

**I've reached a dead-end since I can't work out how to dynamically load the
IUP IM library**

## License

rsiup is free open source software (FOSS) licensed under the Apache-2.0
license: see LICENSE.
