use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use tyra::prelude::{ActorFactory, ActorMessage, ActorSystem, ActorContext, Handler, TyraConfig, ActorMessageDeserializer};

#[derive(Clone)]
struct SleepMsg {}

impl ActorMessage for SleepMsg {}

#[derive(Clone)]
struct SleepActor {
    counter: usize,
}

impl ActorMessageDeserializer for SleepActor {}

impl Handler<SleepMsg> for SleepActor {
    fn handle(&mut self, _msg: SleepMsg, _context: &ActorContext<Self>) {
        self.counter += 1;
        //if self.counter == 1 {
        sleep(Duration::from_secs(3));
        //}
        //if self.counter % 1000000 == 0 {
        println!("Received SERS: {}", self.counter);
        //}
    }
}

struct SleepActorFactory {
    counter: usize,
}

impl ActorFactory<SleepActor> for SleepActorFactory {
    fn new_actor(&self, _context: ActorContext<SleepActor>) -> SleepActor {
        SleepActor {
            counter: self.counter,
        }
    }
}

fn main() {
    let actor_config = TyraConfig::new().unwrap();
    let actor_system = ActorSystem::new(actor_config);

    let hw = SleepActorFactory {
        counter: 0,
    };
    let x = actor_system
        .builder()
        .set_mailbox_unbounded()
        .spawn("hello-world", hw).unwrap();
    x.send(SleepMsg {
    });

    sleep(Duration::from_secs(1));

    x.send(SleepMsg {
    });

    sleep(Duration::from_secs(1));

    x.send(SleepMsg {
    });
    //loop {
    //    //sleep(Duration::from_micros(1));
    //    x.send(SleepMsg {
    //        text: String::from("sers+2"),
    //    });
    //    //println!("SEND NOW");
    //    //sleep(Duration::from_micros(400));
    //}

    actor_system.stop(Duration::from_secs(10));
    exit(actor_system.await_shutdown());
}
