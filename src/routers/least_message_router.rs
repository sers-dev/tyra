use crate::actor::actor_wrapper::ActorWrapper;
use crate::actor::context::ActorContext;
use crate::actor::handler::Handler;
use crate::prelude::{Actor, ActorFactory, ActorMessage, ActorResult};
use crate::routers::add_actor_message::AddActorMessage;
use crate::routers::remove_actor_message::RemoveActorMessage;
use crate as tyra;
use log::error;
use std::error::Error;

#[derive(Actor, ActorFactory)]
pub struct LeastMessageRouter<A>
where
    A: Actor + 'static,
{
    #[tyra(default = "0")]
    next_route_index: usize,
    min_mailbox_size: usize,
    #[tyra(default = "Vec::new()")]
    route_to: Vec<ActorWrapper<A>>,
    #[tyra(default = "false")]
    can_route: bool,
}

impl<A> LeastMessageRouter<A>
where
    A: Actor + 'static,
{
}

impl<A> Handler<AddActorMessage<A>> for LeastMessageRouter<A>
where
    A: Actor + 'static,
{
    fn handle(
        &mut self,
        msg: AddActorMessage<A>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        self.route_to.push(msg.actor);
        self.can_route = true;
        return Ok(ActorResult::Ok);
    }
}

impl<A> Handler<RemoveActorMessage<A>> for LeastMessageRouter<A>
where
    A: Actor + 'static,
{
    fn handle(
        &mut self,
        msg: RemoveActorMessage<A>,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if let Some(pos) = self
            .route_to
            .iter()
            .position(|x| x.get_address() == msg.actor.get_address())
        {
            self.route_to.remove(pos);
        }
        if self.route_to.len() == 0 {
            self.can_route = false
        }
        return Ok(ActorResult::Ok);
    }
}

impl<A, M> Handler<M> for LeastMessageRouter<A>
where
    A: Actor + Handler<M> + 'static,
    M: ActorMessage + 'static,
{
    fn handle(
        &mut self,
        msg: M,
        _context: &ActorContext<Self>,
    ) -> Result<ActorResult, Box<dyn Error>> {
        if !self.can_route {
            return Ok(ActorResult::Ok);
        }

        let mut target = self.route_to.get(self.next_route_index).unwrap();
        let mut mailbox_size = target.get_mailbox_size();
        let target_len = self.route_to.len();
        for i in 0..target_len {
            if mailbox_size >= self.min_mailbox_size {
                break;
            }
            let potential_target = self.route_to.get(i).unwrap();
            let size = potential_target.get_mailbox_size();
            if size < mailbox_size {
                mailbox_size = size;
                target = potential_target;
            }
        }

        self.next_route_index += 1;
        if self.next_route_index >= target_len {
            self.next_route_index = 0;
        }

        let result = target.send(msg);
        if result.is_err() {
            error!(
                "Could not forward message to target {}",
                target.get_address().actor
            )
        }
        return Ok(ActorResult::Ok);
    }
}
