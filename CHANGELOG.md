# X.X.X

- configurable message throughput per actor, with global default
- refactor + cleanup configuration
- allow dynamic creation of thread_pools through config
  - automatically start all configured thread_pools

# 0.0.1

- initial release. Each published release on https://crates.io/crates/tyractorsaur will be tracked in GitHub releases 
- core functionality is working
  - create system and spawn pools 
  - create actors and add them into pools of the existing system 
  - send messages to created actors
  - retrieve and handle messages in actor

