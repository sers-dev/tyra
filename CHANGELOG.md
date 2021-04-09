# WIP 0.0.6

- add ability to stop actor system
  - clean stop, sends SystemStopMessage to ALL actors
    - handling of SystemStopMessage can be defined per Actor
    - default handling sends an ActorStopMessage to simply stop the Actor as soon as possible
  - if actors did not exit until timeout is reached, will force stop everything
  - this comes with a slight performance decrease, as we have to read the necessary AtomicBool in the core iteration, to know when to stop
- system internal messages will now call functions defined in ActorTrait
  - we can't implement a handler for these messages that can be overridden, but we can call trait functions for the ActorTrait to give the enduser the ability to configure the behavior
- finally made actor wakeup failsafe and increase time before running actors go back to sleep   

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

- initial release. Each published release on https://crates.io/crates/tyractorsaur will be tracked in GitHub releases 
- core functionality is working
  - create system and spawn pools 
  - create actors and add them into pools of the existing system 
  - send messages to created actors
  - retrieve and handle messages in actor

