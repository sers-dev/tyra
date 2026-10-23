#[macro_export]
macro_rules! setup_router {
    ($system:expr, $router_type:ident, $name:expr, $count:expr, $actor_factory:expr) => {{
        use $crate::router::AddActorMessage;

        let router_factory = <$router_type<_>>::factory();
        let router = $system.builder().spawn(format!("{}-router", $name), router_factory);

        match router {
            Ok(router) => {
                let mut success: Result<(), Box<dyn std::error::Error>> = Ok(());
                for i in 0..$count {
                    let actor = $system.builder().spawn(format!("{}-{}", $name, i), $actor_factory);
                    match actor {
                        Ok(actor) => {
                            if let Err(e) = router.send(AddActorMessage::new(actor)) {
                                success = Err(Box::from(e));
                                break;
                            }
                        }
                        Err(e) => {
                            success = Err(Box::from(e));
                            break;
                        }
                    }
                }
                match success {
                    Ok(_) => Ok(router),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(Box::from(e)),
        }
    }};
}
