//! Tyra is a [configurable](./prelude/struct.TyraConfig.html) typed actor framework
//!
//! An [Actor](./prelude/trait.Actor.html) is an object with which you can only interact by sending predefined [messages](./prelude/trait.ActorMessage.html)
//!
//! Furthermore Actors are bound to a thread pool and can be moved between executions to any of the threads of said pool.
//!
//! [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam) and [flume](https://github.com/zesterer/flume) are used for all internal-messaging
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use tyra::prelude::*;
//! use std::process::exit;
//! use std::time::Duration;
//! use std::error::Error;
//!
//! // define message
//! #[derive(Hash)]
//! struct FooBar {}
//! impl ActorMessage for FooBar {}
//!
//! // define actor
//! struct HelloWorld {}
//! impl Actor for HelloWorld {}
//!
//! // setup required Factory
//! #[derive(Clone)]
//! struct HelloWorldFactory {}
//! impl ActorFactory<HelloWorld> for HelloWorldFactory {
//!     fn new_actor(&mut self, _context: ActorContext<HelloWorld>) -> Result<HelloWorld, Box<dyn Error>> {
//!         Ok(HelloWorld {})
//!     }
//! }
//!
//! // each supported message has its own Handler implementation
//! // this is where the actual work is done
//! impl Handler<FooBar> for HelloWorld {
//!     fn handle(&mut self, _msg: FooBar, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
//!         println!("Message Received!");
//!         Ok(ActorResult::Ok)
//!     }
//! }
//!
//! fn main() {
//!     // create a new actor system with the default config
//!     let actor_config = TyraConfig::new().unwrap();
//!     let actor_system = ActorSystem::new(actor_config);
//!
//!     // create an actor and send it a message
//!     let factory = HelloWorldFactory {};
//!     let actor = actor_system
//!         .builder()
//!         .spawn("hello-world", factory)
//!         .unwrap();
//!     actor.send(FooBar {}).unwrap();
//!
//!     // cleanup
//!     actor.stop().unwrap();
//!     actor_system.stop(Duration::from_millis(5000));
//!     exit(actor_system.await_shutdown());
//! }
//! ```
//!
//! # Architecture
//!
//! ## User POV
//!
//! ```text
//!                             ┌──────────────────────┐
//!                             │                      │
//!                             │      TyraConfig      │
//!                             │                      │
//!                             └──────────┬───────────┘
//!                                        │
//!                                ┌───────▼───────┐
//!                                │               │
//!               ┌────────────────┤  ActorSystem  ├─────────────────┐
//!               │                │               │                 │
//!               │                └───────┬───────┘                 │
//!               │                        │                         │
//!      ┌────────▼─────────┐     ┌────────▼─────────┐     ┌─────────▼────────┐
//!      │                  │     │                  │     │                  │
//!      │  ActorBuilder A  │     │  ActorBuilder B  │     │  ActorBuilder N  │
//!      │                  │     │                  │     │                  │
//!      └┬─────────────────┘     └┬─────────────────┘     └┬─────────────────┘
//!       │                        │                        │
//!       │ ┌────────────┐         │  ┌────────────┐        │  ┌────────────┐
//!       │ │            │         │  │            │        │  │            │
//!       ├─►  Actor A1  │         ├──►  Actor B1  │        ├──►  Actor N1  │
//!       │ │            │         │  │            │        │  │            │
//!       │ └────────────┘         │  └────────────┘        │  └────────────┘
//!       │                        │                        │
//!       │ ┌────────────┐         │  ┌────────────┐        │  ┌────────────┐
//!       │ │            │         │  │            │        │  │            │
//!       ├─►  Actor A2  │         ├──►  Actor B2  │        ├──►  Actor N2  │
//!       │ │            │         │  │            │        │  │            │
//!       │ └────────────┘         │  └────────────┘        │  └────────────┘
//!       │                        │                        │
//!       │ ┌────────────┐         │  ┌────────────┐        │  ┌────────────┐
//!       │ │            │         │  │            │        │  │            │
//!       └─►  Actor An  │         └──►  Actor Bn  │        └──►  Actor Nn  │
//!         │            │            │            │           │            │
//!         └────────────┘            └────────────┘           └────────────┘
//!
//! ```
//!
mod actor;
mod config;
mod message;
mod routers;
mod system;
mod net;

/// core components
pub mod prelude {
    pub use crate::actor::prelude::*;
    pub use crate::config::prelude::*;
    pub use crate::message::prelude::*;
    pub use crate::system::prelude::*;
    pub use crate::net::prelude::*;
}

/// collection of different router implementations
pub mod router {
    pub use crate::routers::prelude::*;
}
