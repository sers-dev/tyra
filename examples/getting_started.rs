use std::error::Error;
use std::process::exit;
use std::time::Duration;
use tyra::prelude::*;

///////////
//Message//
///////////
struct TemplateMessage {
    id: usize
}

impl TemplateMessage {
    pub fn new(id: usize) -> Self {
        Self {
            id
        }
    }
}

impl ActorMessage for TemplateMessage {}

/////////
//Actor//
/////////
struct TemplateActor {}

impl TemplateActor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for TemplateActor {}

///////////
//Factory//
///////////
struct TemplateActorFactory {}

impl TemplateActorFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl ActorFactory<TemplateActor> for TemplateActorFactory {
    fn new_actor(&mut self, _context: ActorContext<TemplateActor>) -> Result<TemplateActor, Box<dyn Error>> {
        Ok(TemplateActor::new())
    }
}

///////////
//Handler//
///////////
impl Handler<TemplateMessage> for TemplateActor {
    fn handle(&mut self, _msg: TemplateMessage, _context: &ActorContext<Self>) -> Result<ActorResult, Box<dyn Error>> {
        println!("SERS: {}", _msg.id);
        Ok(ActorResult::Ok)
    }
}


/////////
//Usage//
/////////

fn main() {
    // first we need to get the config
    // this could be made mutable to add some dynamic config options
    let actor_config = TyraConfig::new().unwrap();
    // then we create the actual actor system using the config
    let actor_system = ActorSystem::new(actor_config);

    // afterwards we can create an actor within the system through the use of the factory
    let actor = actor_system.builder().spawn("template", TemplateActorFactory::new()).unwrap();

    // now we can send implemented messages to the actor
    actor.send(TemplateMessage::new(1));
    actor.send_after(TemplateMessage::new(2), Duration::from_secs(5));
    actor.send_after(TemplateMessage::new(3), Duration::from_secs(4));
    actor.send_after(TemplateMessage::new(4), Duration::from_secs(3));
    actor.send_after(TemplateMessage::new(5), Duration::from_secs(2));
    actor.send_after(TemplateMessage::new(6), Duration::from_secs(1));
    actor.send(TemplateMessage::new(7));

    //expected order:
    //1, 7, 6, 5, 4, 3, 2

    // finally we ask the system to stop, which will in turn ask all actors to stop
    // exit code indicates if actor system was able to properly finish within the timeout
    //actor_system.stop(Duration::from_secs(1));
    exit(actor_system.await_shutdown());
}