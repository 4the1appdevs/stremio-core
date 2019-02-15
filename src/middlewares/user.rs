use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use lazy_static::*;
use serde_derive::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use enclose::*;

const USER_DATA_KEY: &str = "userData";

lazy_static! {
    static ref DEFAULT_ADDONS: Vec<Descriptor> =
        serde_json::from_slice(include_bytes!("../../stremio-official-addons/index.json"))
            .expect("official addons JSON parse");
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Auth {
    key: String,
    user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserData {
    auth: Option<Auth>,
    addons: Vec<Descriptor>,
}
impl Default for UserData {
    fn default() -> Self {
        UserData {
            auth: None,
            addons: DEFAULT_ADDONS.to_owned(),
        }
    }
}

#[derive(Default)]
pub struct UserMiddleware<T: Environment> {
    //id: usize,
    state: Rc<RefCell<Option<UserData>>>,
    api_url: String,
    env: PhantomData<T>,
}
impl<T: Environment> UserMiddleware<T> {
    pub fn new() -> Self {
        UserMiddleware {
            state: Rc::new(RefCell::new(None)),
            api_url: "https://api.strem.io".to_owned(),
            env: PhantomData,
        }
    }

    fn load(&self) -> EnvFuture<UserData> {
        let current_state = self.state.borrow().to_owned();
        if let Some(ud) = current_state {
            return Box::new(future::ok(ud));
        }

        let state = self.state.clone();
        let fut = T::get_storage(USER_DATA_KEY).and_then(move |result: Option<Box<UserData>>| {
            let ud: UserData = *result.unwrap_or_default();
            let _ = state.replace(Some(ud.to_owned()));
            future::ok(ud)
        });
        Box::new(fut)
    }

    fn handle_user_op(&self, user_op: &ActionUser, emit: Rc<DispatcherFn>) {
        // @TODO actions that do not require auth Login, Register, Logout
        // @TODO actions that do require auth PullAddons (persist if same auth key), PushAddons
        // Login/Register follow the same code (except the method); and perhaps Register will take
        // extra (gdpr, etc.)
        // Logout is a separate operation, it clears the UD and sets it to default; perhaps it
        // should first clear the UD, THEN clear the session, to make it logout even w/o a conn
        // PushAddons just pushes and sends a UserMError if it fails
        // PullAddons will set the storage (if the authkey has not changed), and emits AddonsChanged
        if let ActionUser::PullAddons = user_op {
            // @TODO if we have auth_key
            // @TODO check if auth_key has changed
            let req = Request::post(format!("{}/api/addonCollection", &self.api_url))
                .body(CollectionRequest {})
                .expect("failed to build API request");
            let fut = T::fetch_serde::<_, APIResult<CollectionResponse>>(req)
                .and_then(|result| {
                    match *result {
                        APIResult::Ok {
                            result: CollectionResponse { addons },
                        } => {
                            // @TODO: other than storage, we should replace the in-mem thing too
                            T::set_storage(USER_DATA_KEY, Some(&UserData { auth: None, addons }))
                        },
                        // @TODO should this error be handled better?
                        APIResult::Err { error } => Box::new(future::err(error.message.into())),
                    }
                })
                .or_else(enclose!((emit) move |e| {
                    // @TODO better err handling?
                    // there are a few types of errors here: network errors, deserialization
                    // errors, API errors
                    emit(&Action::UserMError(e.to_string()));
                    future::err(())
                }));
            T::exec(Box::new(fut));
        }
    }
}

impl<T: Environment> Handler for UserMiddleware<T> {
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // @TODO: add/remove addons; consider AddonsChanged
        if let Action::Load(action_load) = action {
            let action_load = action_load.to_owned();
            let fut = self
                .load()
                .and_then(enclose!((emit) move |ud| {
                    emit(&Action::LoadWithAddons(ud.addons.to_vec(), action_load));
                    future::ok(())
                }))
                .or_else(enclose!((emit) move |e| {
                    // @TODO consider that this error is fatal, while the others are not
                    // perhaps consider a recovery strategy here?
                    emit(&Action::UserMFatal(e.to_string()));
                    future::err(())
                }));
            T::exec(Box::new(fut));
        }

        if let Action::UserOp(user_op) = action {
            self.handle_user_op(user_op, emit);
        }
    }
}
