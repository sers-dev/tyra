# Tractor (Typed Rust Actor)

Goal would be to create a library in pure Rust, that gives us the most important features of akka and akka-cluster (although akka-cluster will probably be moved to a separate repository)

First Steps to achieve this goal:
 - "Merge" Axiom and Actix
   - Typed Actors from Actix
   - Behavior like Axiom 
      - no return values from sent messages
      - no interaction with actors without message sending
      - actors are not pinned to a thread but a thread pool
    
    
After a working Actor System is established:
 - benchmark and optimize until satisfying results are returned in performance
 - open source library
 - (blog post)
 - fix bugs
 - develop cluster features

Possible Libraries:
 - threadpool for executor instead of self build madness from axiom
 - flume/crossbeam for message channels instead of self build madness from axiom (also allow unbound channels and make them default like akka)





