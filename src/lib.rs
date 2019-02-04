pub mod middlewares;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use self::middlewares::*;
    use self::state_types::*;
    use super::*;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    #[test]
    fn middlewares() {
        // to make sure we can't use 'static
        t_middlewares();
    }
    fn t_middlewares() {
        // @TODO test what happens with no handlers
        let container = std::rc::Rc::new(std::cell::RefCell::new(Container::with_reducer(
            CatalogGrouped::new(),
            &catalogs_reducer,
        )));
        let container_ref = container.clone();
        let chain = Chain::new(
            vec![
                Box::new(UserMiddleware::<Env>::new()),
                Box::new(CatalogMiddleware::<Env>::new()),
                Box::new(ContainerHandler::new(0, container)),
                // @TODO: reducers multiplexer middleware
            ],
            Box::new(move |action| {
                // @TODO: test if this is actually progressing properly
                if let Action::NewState(_) = action {
                    //println!("new state {:?}", container_ref.borrow().get_state());
                }
            }),
        );

        // this is the dispatch operation
        let action = &Action::Load(ActionLoad::Catalog);
        chain.dispatch(action);

        // since the Env implementation works synchronously, this is OK
        assert_eq!(
            container_ref.borrow().get_state().groups.len(),
            6,
            "groups is the right length"
        );
    }

    struct Env;
    impl Environment for Env {
        fn fetch_serde<IN, OUT>(in_req: Request<IN>) -> EnvFuture<Box<OUT>>
        where
            IN: 'static + Serialize,
            OUT: 'static + DeserializeOwned,
        {
            /*
            // Can't work for now, as it needs + Send
            req.send()
                .and_then(|mut res: reqwest::r#async::Response| {
                    res.json::<OUT>()
                })
                .map(|res| Box::new(res))
                .map_err(|e| e.into());
            Box::new(fut)
            */
            let (parts, body) = in_req.into_parts();
            let method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
                .expect("method is not valid for reqwest");
            let mut req = reqwest::Client::new().request(method, &parts.uri.to_string());
            // NOTE: both might be HeaderMap, so maybe there's a better way?
            for (k, v) in parts.headers.iter() {
                req = req.header(k.as_str(), v.as_ref());
            }
            // @TODO add content-type application/json
            req = req.json(&body);
            Box::new(match req.send() {
                Err(e) => future::err(e.into()),
                Ok(mut resp) => match resp.json() {
                    Err(e) => future::err(e.into()),
                    Ok(resp) => future::ok(Box::new(resp)),
                },
            })
        }
        fn exec(fut: Box<Future<Item = (), Error = ()>>) {
            fut.wait().unwrap();
        }
        fn get_storage<T: 'static + DeserializeOwned>(_key: &str) -> EnvFuture<Option<Box<T>>> {
            Box::new(future::err("unimplemented".into()))
        }
        fn set_storage<T: 'static + Serialize>(_key: &str, _value: &T) -> EnvFuture<()> {
            Box::new(future::err("unimplemented".into()))
        }
    }
}
