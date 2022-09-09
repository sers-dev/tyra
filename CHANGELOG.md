# WIP 0.7.0
 
 - actors can now delay message processing by going into a sleep state
   - can be achieved through `actor.sleep(duration)` on the `ActorWrapper` or by returning `ActorState::Sleep(duration)` from within the actor
 - fixed a bug where an actor with a limited mailbox can get stuck on shutdown
 - added `send_timeout()`
 - added return `Result<(), ActorSendError>` to message sending functions
 - actually enforce configured actor limits per thread_pool
 - added `get_available_actor_count_for_pool` that allows user to retrieve how many actors can still be spawned on a given thread_pool
 - added `ShardedRouter` that will consistently route messages to the targets
   - added `get_id` to `Message` trait
   - shards are reset whenever an actor is added/removed to the router
   - shard count is equal to the amount of targets times 5
 - `ActorFactory::new_actor` now returns a result
   - result of type error will result in same behavior as a panic

# 0.6.0

 - add error handling
   - `ActorBuilder.spawn()` now returns a Result, containing either the `ActorWrapper<A>` or an `ActorError`
   - added `ActorResult` that is returned by all relevant `Actor` functions
     - configures how the Actor should proceed
   - Proper panic handling now in all parts of the `Actor` and `Handler<M>` and `ActorFactory<A>`
     - panic now triggers `Actor.on_panic` providing the source of panic and allows User to determine how to proceed
       - `Actor.on_panic` is allowed to panic once and will be re-triggered in that case. If another panic happens in the retry, the Actor will be stopped
     - handling a panic within `ActorFactory<A>.new_actor()` by returning `ActorResult::Restart()` in `Actor.on_panic` can trigger a restart loop that will block the thread until `ActorFactory<A>.new_actor()` was successful
 - replaced `RestartPolicy` with `ActorResult`

# 0.5.0

- added `send_after` to `ActorWrapper<A>` to allow sending of delayed messages
  - delayed messages are handled through system internal DelayActor
  - system internal Actors are running on the `tyra` Threadpool. It is not recommended to re-use or re-configure that thread pool in any way

# 0.4.0

- added `serialize` example
- added `router_benchmark` example
- added `bulk_router_benchmark` example
- added `BulkActorMessage` and `BulkRouterMessage`
  - all implemented Messages `M` automatically support being sent through a `BulkActorMessage<M>` wrapper
- env configuration now done with `TYRA` prefix instead of `TYRACTOSAUR`
- fix serialized message handling
  - SerializedMessages are now properly sent through the mailbox and will follow same rules as any other message
  - serialized messages are now handled by the exact same object as any other message instead of a copy
- reworked `spawn()` behavior of `ActorBuilder`
  - will now return an `Option<ActorRef>` to the correct actor, even if it was not built by the same `ActorBuilder` beforehand, as long as the type matches
  - returns None only if the type of the Actor does not match the expected Actor type of the ActorBuilder
- added `getting_started` example

# 0.3.0

- renamed to `tyra`
  - equal to `tyractorsaur` release `0.2.0`
  - all releases before `0.2.0` will still be available as `tyractorsaur`, but will not be migrated to `tyra`
  - all releases after this one will only be available as `tyra`

# 0.2.0

- add documentation link to metadata
- update dependencies
  - upgrade `config` to `0.11.0`
  - remove explicit `lexical-core`, since documentation should now be able to be generated without it

# 0.1.1

- pin `lexical-core` to `0.7.6` to fix doc generation on nightlyfor `docs.rs`

# 0.1.0

All releases before this one set the groundwork for the actor framework and can be considered as Alpha releases.
Starting with this release the framework can be considered to be in Beta, which is why the most changes between `0.1.0` and `0.0.8` consist of refactoring, documentation and API cleanup.

Although a lot of work went into Refactoring of the public facing API, please do not yet consider the current API to be stable and expect some slight changes throughout the next Releases

- cleanup
- refactoring
- the following User facing resources have been renamed:
  - `ActorTrait` -> `Actor`
  - `MessageTrait` -> `ActorMessage`
  - `ActorRef` -> `ActorWrapper`
  - `ActorProps` -> `ActorFactory`
  - `ActorHandler` -> `Executor`
- moved routers from `mod prelude` to `mod router`
- add documentation
- fixed a bug where `ActorSystem.send_to_address()` was not correctly executed
- refactor `ActorBuilder` to `ActorBuilder<A>`
  - stores Actors that have already been created
    - creating the same Actor with a Builder twice, will create a single Actor and return the `ActorWrapper<A>` for both
    - returns `None` if the ActorAddress exists, but was not created by the same Builder

# WIP 0.0.8

- implement `clone()` for `ActorRef`
- ActorProps now receive the `Context<A>` as parameter when creating a new Actor
  - this way the User can store the `Context<Self>` inside his Actor Struct
- removed `Context<A>` from ActorTrait functions
  - if you need to Access the Context within any of these functions, consider Storing it as part of your Actor Struct
- remove default behavior for `SystemStopMessage`
  - This was done, because we lost the reference to `Context<A>` in the Trait function
- ActorStopMessage and SystemStopMessage are no longer directly accessible by the User
  - to stop an actor use `.stop();`, this ensures that the actor is also removed from the list of running actors
- replace magic in thread pool config
  - thread_pools are now configured with `min`, `max`, and `factor`
- Added round robin router with usage example 

- Remoting:
  - All necessary Remoting preparations are included in this Release. With these changes Remoting is in itself a possibility, although at this point in time no Remoting Library is provided
  - Added `SerializedMessage`
  - Added `fn handle_serialized_message(&self, msg: SerializedMessage)` to Actor Trait
  - System now Stores `dyn ActorTrait` per `ActorAddress`
  - System now provides `send_to_addrress()` which takes an `ActorAddress` and a `SerializedMessage` to call `handle_serialized_message`
  

# 0.0.7

- Actors are now created through ActorProps
  - this was necessary to remove the `actor_backup` from the `ActorHandler` and to rework the System to no longer make use of `.clone()` calls on the Actor
- Remove `Clone` Trait from Actors and Messages
  - this is only possible because of the introduction of the ActorProps

# 0.0.6

- add ability to stop actor system
  - clean stop, sends SystemStopMessage to ALL actors
    - handling of SystemStopMessage can be defined per Actor
    - default handling sends an ActorStopMessage to simply stop the Actor as soon as possible
  - if actors did not exit until timeout is reached, will force stop everything
  - this comes with a slight performance decrease, as we have to read the necessary AtomicBool in the core iteration, to know when to stop
- system internal messages will now call functions defined in ActorTrait
  - we can't implement a handler for these messages that can be overridden, but we can call trait functions for the ActorTrait to give the enduser the ability to configure the behavior
- finally made actor wakeup failsafe and increase time before running actors go back to sleep
- allow Actors to hold any ActorRef by explicitly implementing UnwindSafe for ActorRef 

# 0.0.5

- Actors can now be stopped by calling stop on an ActorRef
  - this sends a "StopMessage" to the Actor
  - all messages queued until the stop message is handled in the actor will still be executed
- fix ActorAddress generation used for Wakeup Calls  
- inject context into message handling
  - this allows the actor to interact with itself
  - this allows the actor to interact with the actor system

# 0.0.4

- Actor Wakeup: properly handle wakeup de-duplication
- added pre_start and post_stop to Actors

# 0.0.3

- fix ActorRef send mutability
- fix a Bug where message handlers could be executed manually on an ActorRef
- Actors now have a sleeping state
  - New Actors automatically start in this state until a message is received
  - Sleeping Actors do not use any CPU
  - After Wakeup, there's a hardcoded minimum interval of 1 second before an Actor is put to Sleep again

# 0.0.2

- added configurable actor restart mechanism
- performance increase by moving RwLock from Actor to ActorRef
- configurable message throughput per actor, with global default
- refactor + cleanup configuration
- allow dynamic creation of thread_pools through config
  - automatically start all configured thread_pools
- keep dependencies up2date
  - hard pin version for dependencies that do not have regular updates

# 0.0.1

- initial release. Each published release on https://crates.io/crates/tyra will be tracked in GitHub releases 
- core functionality is working
  - create system and spawn pools 
  - create actors and add them into pools of the existing system 
  - send messages to created actors
  - retrieve and handle messages in actor

