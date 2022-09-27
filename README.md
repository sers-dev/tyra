# Tyra (TYped Rust Actor)

`Tyra` is a cross-platform Typed Actor System Written in Rust.

## Current State

`0.8.0` is intended to be the last minor release before `1.0.0` where it's officially declared ready for production
 - since `0.7.0` it's been used for a variety of internal non-public tools
 - documentation + examples + tests should be in a good state
 - API is finalized
 - performance is acceptable

So why's it not yet `1.0.0`?
 - more time to intensively test
   - fixing more bugs found from testing
 - finalizing dependencies

The current development status can be tracked in the [CHANGELOG.md](CHANGELOG.md)

## Configuration

See [default.toml](./src/config/default.toml) for a list of all configuration parameters and their defaults.

Configuration can be adjusted by providing a `./config/tyra.toml`.

## Clustering

Through the current implementation of the `SerializedMessage` it's proven that this system can be clustered.
Development will focus on implementing a proper clustering system once `1.0.0` has been released. 

[cargo run --example serialize](./examples/serialize.rs) to view/run the poc implementation 

## Benchmarks

There are a few different benchmarks within [examples](./examples), prefixed with `benchmark_`

The most interesting things to look at are specifically (ordered from fastest to slowest):
- [cargo run --release --example benchmark_single_actor_process_after_send_single_thread](benchmark_single_actor_process_after_send_single_thread)
- [cargo run --release --example benchmark_single_actor_process_after_send](benchmark_single_actor_process_after_send)
- [cargo run --release --example benchmark_single_actor](benchmark_single_actor)
  - probably the most common scenario
- [cargo run --release --example benchmark_single_actor_single_thread](benchmark_single_actor_single_thread)
  - roughly as fast as the `benchmark_single_actor` but some executions are much slower


#### Don't just trust random benchmarks posted somewhere and do your own!
Nonetheless, it makes sense to post some generic data that gives at least some insight into the performance:
 - Release: `0.8.0`
 - CPU: Ryzen 1700 with 8 cores and 16 threads running at 3GHz
 - tyra config:
   - `general.default_mailbox_size: 0` (setting a fixed mailbox_size would further increase performance)
   - `general.default_message_throughput: 15` (obviously could also be optimized for single actor usage, but would probably not be a fair measurement)
   - `thread_pool.config.default.threads_max: 10` (overwritten by `_single_thread` tests)
 - running `benchmark_single_actor` 10mil messages always within at least 3.3 seconds
   - send+receive 10mil messages in at least 3.3 seconds
     - ~3mil messages per second
 - running `benchmark_single_actor_process_after_send_single_thread`
   - send 10mil messages in at least 700ms
     - ~14mil messages per second
   - receive 10mil messages in at least 750ms
     - ~13mil messages per second
 - running `benchmark_single_actor_process_after_send`
   - send 10mil messages in at least 700ms
     - ~14mil messages per second
   - receive 10mil messages in at least 1750ms
     - ~5.7mil messages per second


## Documentation

[docs.rs](https://docs.rs/tyra/) or generate your own with `cargo doc`

## Quickstart

This code can be found in [examples/quickstart.rs](./examples/quickstart.rs) and can be executed with `cargo run --example quickstart`

```rust
use tyra::prelude::*;
use std::error::Error;
use std::time::Duration;

// define an `ActorMessage` that can be sent to `Actors` that implement the corresponding `Handler<T>`
struct TestMessage {}
impl TestMessage {
    pub fn new() -> Self {
        Self {}
    }
}
impl ActorMessage for TestMessage {}

// define the actual `Actor` that should process messages
struct TestActor {}
impl TestActor {
    pub fn new() -> Self {
        Self {}
    }
}
impl Actor for TestActor {}

// define a factory that creates the `Actor` for us
struct TestActorFactory {}
impl TestActorFactory {
    pub fn new() -> Self {
        Self {}
    }
}
impl ActorFactory<TestActor> for TestActorFactory {
    fn new_actor(&mut self, _context: ActorContext<TestActor>) -> Result<TestActor, Box<dyn Error>> {
        Ok(TestActor::new())
    }
}

// implement our message for the `Actor`
impl Handler<TestMessage> for TestActor {
    fn handle(&mut self, _msg: TestMessage, context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        println!("HELLO WORLD!");
        context.system.stop(Duration::from_millis(1000));
        Ok(ActorResult::Ok)
    }
}

fn main() {
    // generate config
    let actor_config = TyraConfig::new().unwrap();
    // start system with config
    let actor_system = ActorSystem::new(actor_config);
    // create actor on the system
    let actor = actor_system.builder().spawn("test", TestActorFactory::new()).unwrap();
    // send a message to the actor
    actor.send(TestMessage::new()).unwrap();
    // wait for the system to stop
    std::process::exit(actor_system.await_shutdown());
}
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