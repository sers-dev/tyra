use crate::actor::actor_wrapper::ActorWrapper;
use crate::prelude::Actor;
use crate::system::actor_system::ActorSystem;
use std::panic::UnwindSafe;

/// Enables access to [ActorSystem] and [Actor] within [Handler](./trait.Handler.html) implementations
///
/// Also injected into [ActorFactory.new_actor](../prelude/trait.ActorFactory.html#tymethod.new_actor), so that it can be stored within the Actor
pub struct ActorContext<A>
where
    Self: Send + Sync,
    A: Actor,
{
    pub actor_ref: ActorWrapper<A>,
    pub system: ActorSystem,
}

impl<A> UnwindSafe for ActorContext<A> where A: Actor {}

impl<A> Clone for ActorContext<A>
where
    A: Actor,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            actor_ref: self.actor_ref.clone(),
        }
    }
}

impl<A> ActorContext<A>
where
    A: Actor,
{
    pub fn dummy() -> Self {
        let config = crate::prelude::TyraConfig::new().unwrap();
        let system = crate::prelude::ActorSystem::new(config);
        let (tx, _rx) = flume::unbounded();
        let mailbox = crate::actor::mailbox::Mailbox {
            msg_in: tx,
            is_stopped: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            is_sleeping: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        let address = crate::actor::actor_address::ActorAddress {
            remote: String::new(),
            system: String::new(),
            pool: String::new(),
            actor: String::new(),
        };
        let wakeup_manager = crate::system::wakeup_manager::WakeupManager::new();
        let internal_actor_manager = crate::system::internal_actor_manager::InternalActorManager::new();
        let actor_ref = crate::actor::actor_wrapper::ActorWrapper::new(
            mailbox,
            address,
            wakeup_manager,
            internal_actor_manager,
        );
        Self {
            system,
            actor_ref,
        }
    }
}
