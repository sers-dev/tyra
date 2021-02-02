# Tractor (Typed Rust Actor)

`Tractor` is a cross platform Typed Actor System Written in Rust.

## Configuration

[Default Config](src/config/default.toml)

There are two ways to configure Tractor
 - environment variables in the form of `TRACTOR_CONFIG_<KEY>` i.e. `TRACTOR_CONFIG_NAME=custom`
 - creating the configuration as mutable and overwriting the values in your code
   ```rust
   let mut actor_config = TractorConfig::new().unwrap();
   actor_config.actor.name = String::from("custom-name");
   ```

## Clustering

This crate is written with Clustering in mind from the get go. As soon as the core reaches a certain stability, we will start working on Cluster functionality

