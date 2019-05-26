//use crate::addon_transport::AddonTransport;
use crate::addon_transport::AddonInterface;
use crate::state_types::{EnvFuture, Environment, Handler, DispatcherFn, Msg};
use crate::types::addons::{Manifest, ResourceRef, ResourceResponse};
use crate::types::LibItem;
use futures::future::Shared;
use futures::future::join_all;
use futures::{future, Future};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use std::rc::Rc;
use std::collections::HashSet;

#[derive(Clone)]
pub struct SharedIndex(Arc<RwLock<Vec<LibItem>>>);

impl SharedIndex {
    fn from_items(items: Vec<LibItem>) -> Self {
        SharedIndex(Arc::new(RwLock::new(items)))
    }
    // @TODO: get_resources
    fn get_types(&self) -> Vec<String> {
        let idx = self.0.read().expect("unable to read idx");
        let types: HashSet<_> = idx.iter().map(|x| x.type_name.to_owned()).collect();
        types.into_iter().collect()
    }
}

pub struct LibAddon<T: Environment + 'static> {
    pub idx_loader: Shared<EnvFuture<SharedIndex>>,
    auth_key: String,
    env: PhantomData<T>,
}

impl<Env: Environment + 'static> LibAddon<Env> {
    pub fn with_authkey(auth_key: String) -> Self {
        let key = format!("library:{}", auth_key.chars().take(6).collect::<String>());
        // flatten cause it's an Option<Vec<...>>
        let idx_loader = Env::get_storage::<Vec<String>>(&key)
            .and_then(|keys| join_all(keys.into_iter().flatten().map(|k| Env::get_storage::<LibItem>(&k))))
            .map(|items| SharedIndex::from_items(items.into_iter().flatten().collect()));
        LibAddon {
            idx_loader: Future::shared(Box::new(idx_loader)),
            auth_key,
            env: PhantomData,
        }
    }
}
impl<T: Environment + 'static> Clone for LibAddon<T> {
    fn clone(&self) -> Self {
        LibAddon {
            idx_loader: self.idx_loader.clone(),
            auth_key: self.auth_key.clone(),
            env: PhantomData,
        }
    }
}

// @TODO sync pipeline
// @TODO videos pipeline

impl<T: Environment + 'static> Handler for LibAddon<T> {
    fn handle(&self, _action: &Msg, _emit: Rc<DispatcherFn>) {
        unimplemented!()
    }
}

impl<T: Environment + 'static> AddonInterface for LibAddon<T> {
    fn get(&self, _: &ResourceRef) -> EnvFuture<ResourceResponse> {
        unimplemented!()
    }
    fn manifest(&self) -> EnvFuture<Manifest> {
        Box::new(
            self.idx_loader
                .clone()
                .and_then(|idx| {
                    future::ok(Manifest {
                        id: "org.stremio.libitem".into(),
                        name: "Library".into(),
                        version: semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
                        // @TODO dynamic
                        resources: vec![],
                        types: idx.get_types(),
                        catalogs: vec![],
                        contact_email: None,
                        background: None,
                        logo: None,
                        id_prefixes: None,
                        description: None,
                    })
                })
                // @TODO fix by making the idx_loader result in a Cloneable error
                // this is not trivial to fix, as the underlying error is also a Box<dyn Error>
                .map_err(|_| "index loading error".into()),
        )
    }
}
