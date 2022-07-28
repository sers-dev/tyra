struct Template {}

impl Actor for Template {}

impl Handler<SerializedMessage> for Template {
    fn handle(&mut self, _msg: SerializedMessage, _context: &ActorContext<Self>) {}
}

impl Handler<TestMsg> for RemoteActor {
    fn handle(&mut self, msg: TestMsg, _context: &ActorContext<Self>) {
        println!("{}", msg.content);
    }
}

struct RemoteActorFactory {}

impl ActorFactory<RemoteActor> for RemoteActorFactory {
    fn new_actor(&self, context: ActorContext<RemoteActor>) -> RemoteActor {
        RemoteActor { ctx: context }
    }
}