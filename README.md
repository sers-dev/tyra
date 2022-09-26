# Tyra (TYped Rust Actor)

`Tyra` is a cross-platform Typed Actor System Written in Rust.


## Current State

`0.8.0` is intended to be the last Release before `1.0.0` and it's declared ready for production
 - since `0.7.0` it's been used for a variety of internal non-public tools
 - documentation + examples + tests should be in a good state
 - API is finalized
 - performance is acceptable

So why's it not yet `1.0.0`?
 - more time to intensively test
   - fixing more bugs found from testing
 - finalizing dependencies

The current development status can be tracked in the [CHANGELOG.md](CHANGELOG.md)

## Clustering

Through the current implementation of the `SerializedMessage` it's proven that this system can be clustered.
Development will focus on implementing a proper clustering system once `1.0.0` has been released. 

[cargo run --example serialize](./examples/serialize.rs) to view/run the poc implementation 

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