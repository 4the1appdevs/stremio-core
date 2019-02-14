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
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn middlewares() {
        // to make sure we can't use 'static
        inner_middlewares();
    }
    fn inner_middlewares() {
        // @TODO: Fix: the assumptions we are testing against are pretty much based on the current
        // official addons; e.g. assuming 6 groups, or 4 groups when searching
        // @TODO test what happens with no handlers
        let container = Rc::new(RefCell::new(Container::with_reducer(
            CatalogGrouped::new(),
            &catalogs_reducer,
        )));
        let container_ref = container.clone();
        let chain = Chain::new(
            vec![
                Box::new(UserMiddleware::<Env>::new()),
                Box::new(AddonsMiddleware::<Env>::new()),
                Box::new(ContainerHandler::new(0, container)),
            ],
            Box::new(move |action| {
                // @TODO: test if this is actually progressing properly
                if let Action::NewState(_) = action {
                    //println!("new state {:?}", container_ref.borrow().get_state());
                }
            }),
        );

        // this is the dispatch operation
        let action = &Action::Load(ActionLoad::CatalogGrouped { extra: vec![] });
        chain.dispatch(action);

        // since the Env implementation works synchronously, this is OK
        let state = container_ref.borrow().get_state().to_owned();
        assert_eq!(state.groups.len(), 6, "groups is the right length");
        for g in state.groups.iter() {
            assert!(
                match g.1 {
                    Loadable::Ready(_) => true,
                    Loadable::Message(_) => true,
                    _ => false,
                },
                "group is Ready or Message"
            );
        }
        if !state.groups.iter().any(|g| {
            if let Loadable::Ready(_) = g.1 {
                true
            } else {
                false
            }
        }) {
            panic!("there are no items that are Ready in state {:?}", state);
        }

        // Now try to Search
        let extra = vec![("search".to_owned(), "grand tour".to_owned())];
        let action = &Action::Load(ActionLoad::CatalogGrouped { extra });
        chain.dispatch(action);
        let state = container_ref.borrow().get_state().to_owned();
        assert_eq!(
            state.groups.len(),
            4,
            "groups is the right length when searching"
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
            // @TODO: if the response code is not 200, return an error related to that
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
            match fut.wait() {
                Ok(_) => {}
                Err(e) => dbg!(e),
            }
        }
        fn get_storage<T: 'static + DeserializeOwned>(_key: &str) -> EnvFuture<Option<Box<T>>> {
            Box::new(future::ok(None))
        }
        fn set_storage<T: 'static + Serialize>(_key: &str, _value: Option<&T>) -> EnvFuture<()> {
            Box::new(future::err("unimplemented".into()))
        }
    }
}
