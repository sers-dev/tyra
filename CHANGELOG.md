# WIP 0.0.3

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

