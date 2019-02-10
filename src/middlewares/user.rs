use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use serde_derive::*;
use std::marker::PhantomData;
use std::rc::Rc;

// @TODO: auth_key, user, addons
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct UserData {
    auth_key: String,
    user: User,
    addons: Vec<Descriptor>,
}

#[derive(Default)]
pub struct UserMiddleware<T: Environment> {
    //id: usize,
    pub user: Option<String>,
    pub env: PhantomData<T>,
}
impl<T: Environment> UserMiddleware<T> {
    pub fn new() -> Self {
        UserMiddleware {
            user: None,
            env: PhantomData,
        }
    }
}
impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // @TODO Action::SyncAddons, Action::TryLogin
        if let Action::Load(action_load) = action {
            let action_load = action_load.to_owned();
            let req = Request::post("https://api.strem.io/api/addonCollectionGet")
                .body(CollectionRequest {})
                .unwrap();
            let fut = T::fetch_serde::<_, APIResult<CollectionResponse>>(req)
                .and_then(move |result| {
                    // @TODO err handling
                    match *result {
                        APIResult::Ok {
                            result: CollectionResponse { addons },
                        } => {
                            emit(&Action::LoadWithAddons(addons.to_vec(), action_load));
                        }
                        APIResult::Err { error } => {
                            // @TODO err handling
                            dbg!(error);
                        }
                    }
                    future::ok(())
                })
                .or_else(|_e| {
                    // @TODO err handling
                    future::err(())
                });
            T::exec(Box::new(fut));
        }
    }
}

// @TODO move those to types/api or something
#[derive(Deserialize, Clone, Debug)]
struct APIErr {
    message: String,
}
// @TODO
#[derive(Serialize, Clone)]
struct CollectionRequest {}
#[derive(Serialize, Deserialize)]
struct CollectionResponse {
    pub addons: Vec<Descriptor>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum APIResult<T> {
    Ok { result: T },
    Err { error: APIErr },
}
