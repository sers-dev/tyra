# Tyractorsaur (TYped Rust ACTOR "SAUR")

`Tyractorsaur` is a cross platform Typed Actor System Written in Rust.

## Current State

It is `NOT` production ready.
`Tyractorsaur` is in very early development steps. The core functionality of creating actors, sending, receiving messages as well as waiting for the System to stop are already implemented.


## Configuration

[Default Config](src/config/default.toml)

There are two ways to configure Tyractorsaur
 - environment variables in the form of `TYRACTORSAUR_CONFIG_<KEY>` i.e. `TYRACTORSAUR_CONFIG_NAME=custom`
 - creating the configuration as mutable and overwriting the values in your code
   ```rust
   let mut actor_config = TyractorsaurConfig::new().unwrap();
   actor_config.actor.name = String::from("custom-name");
   ```

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.