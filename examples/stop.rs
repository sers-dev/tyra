use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use tyractorsaur::prelude::{
    Actor, ActorFactory, ActorMessage, ActorSystem, ActorContext, Handler, TyractorsaurConfig,
};

#[derive(Clone)]
struct TestMsg {}

impl ActorMessage for TestMsg {}

#[derive(Clone)]
struct StopActor {
    ctx: ActorContext<Self>,
}

impl Actor for StopActor {
    fn pre_start(&mut self) {
        println!("PRE START")
    }
    fn post_stop(&mut self) {
        self.ctx.system.stop(Duration::from_secs(1));
        println!("POST STOP");
    }
}

impl Handler<TestMsg> for StopActor {
    fn handle(&mut self, _msg: TestMsg, context: &ActorContext<Self>) {
        context.actor_ref.send(TestMsg {});
        println!("Message received!");
        sleep(Duration::from_millis(100));
    }
}

struct StopActorFactory {}

impl ActorFactory<StopActor> for StopActorFactory {
    fn new_actor(&self, context: ActorContext<StopActor>) -> StopActor {
        StopActor { ctx: context }
    }
}

fn main() {
    let actor_config = TyractorsaurConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = StopActorFactory {};
    let x = actor_system
        .builder("hello-world")
        .set_mailbox_size(7)
        .build(hw);
    // this is obviously handled, because it's the actor is still running
    x.send(TestMsg {});
    sleep(Duration::from_millis(700));

    x.stop();
    // this is still handled, because the actor has not handled the stop Message yet
    x.send(TestMsg {});
    sleep(Duration::from_millis(200));
    // this is no longer handled, because the actor has stopped by now
    x.send(TestMsg {});

    let result = actor_system.await_shutdown();

    exit(result);
}
