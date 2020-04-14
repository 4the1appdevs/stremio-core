use super::{fetch_api, update_library, update_profile, CtxError};
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY,
};
use crate::state_types::msg::{Action, ActionCtx, ActionLoad, Event, Internal, Msg};
use crate::state_types::{Effects, Environment, Update};
use crate::types::addons::Descriptor;
use crate::types::api::{
    APIRequest, Auth, AuthRequest, AuthResponse, CollectionResponse, DatastoreCmd, DatastoreReq,
    SuccessResponse,
};
use crate::types::profile::Profile;
use crate::types::{LibBucket, LibItem};
use derivative::Derivative;
use enclose::enclose;
use futures::Future;
use serde::Serialize;
use std::marker::PhantomData;
use std::ops::Deref;

pub type CtxStorageResponse = (Option<Profile>, Option<LibBucket>, Option<LibBucket>);
pub type CtxAuthResponse = (Auth, Vec<Descriptor>, Vec<LibItem>);

#[derive(Clone, Debug, PartialEq)]
pub enum CtxRequest {
    Storage,
    API(AuthRequest),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CtxStatus {
    Loading(CtxRequest),
    Ready,
}

#[derive(Derivative, Clone, PartialEq, Serialize)]
#[derivative(Default, Debug)]
pub struct Ctx<Env: Environment> {
    pub profile: Profile,
    // TODO StreamsBucket
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(skip)]
    pub library: LibBucket,
    #[serde(skip)]
    #[derivative(Default(value = "CtxStatus::Ready"))]
    status: CtxStatus,
    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::Ctx)) => {
                self.status = CtxStatus::Loading(CtxRequest::Storage);
                Effects::one(Box::new(
                    Env::get_storage(PROFILE_STORAGE_KEY)
                        .join3(
                            Env::get_storage(LIBRARY_RECENT_STORAGE_KEY),
                            Env::get_storage(LIBRARY_STORAGE_KEY),
                        )
                        .map_err(CtxError::from)
                        .then(|result| {
                            Ok(Msg::Internal(Internal::CtxStorageResult(Box::new(result))))
                        }),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                self.status = CtxStatus::Loading(CtxRequest::API(auth_request.to_owned()));
                Effects::one(Box::new(
                    fetch_api::<Env, _, _>(&APIRequest::Auth(auth_request.to_owned()))
                        .map(|AuthResponse { key, user }| Auth { key, user })
                        .and_then(|auth| {
                            fetch_api::<Env, _, _>(&APIRequest::AddonCollectionGet {
                                auth_key: auth.key.to_owned(),
                                update: true,
                            })
                            .map(|CollectionResponse { addons, .. }| addons)
                            .join(fetch_api::<Env, _, _>(&DatastoreReq {
                                auth_key: auth.key.to_owned(),
                                collection: LIBRARY_COLLECTION_NAME.to_owned(),
                                cmd: DatastoreCmd::Get {
                                    ids: vec![],
                                    all: true,
                                },
                            }))
                            .map(move |(addons, lib_items)| (auth, addons, lib_items))
                        })
                        .then(enclose!((auth_request) move |result| {
                            Ok(Msg::Internal(Internal::CtxAuthResult(auth_request, result)))
                        })),
                ))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let ctx_effects = {
                    let uid = self.profile.uid();
                    let session_effects = match &self.profile.auth {
                        Some(auth) => Effects::one(Box::new(
                            fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
                                auth_key: auth.key.to_owned(),
                            })
                            .map(enclose!((uid) move |_| {
                                Msg::Event(Event::SessionDeleted { uid })
                            }))
                            .map_err(enclose!((uid) move |error| {
                                Msg::Event(Event::Error {
                                    error,
                                    source: Box::new(Event::SessionDeleted { uid }),
                                })
                            })),
                        ))
                        .unchanged(),
                        _ => Effects::none().unchanged(),
                    };
                    Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                        .unchanged()
                        .join(session_effects)
                };
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, &msg);
                let library_effects =
                    update_library::<Env>(&mut self.library, &self.profile, &self.status, &msg);
                self.status = CtxStatus::Ready;
                ctx_effects.join(profile_effects).join(library_effects)
            }
            Msg::Internal(Internal::CtxStorageResult(result)) => {
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<Env>(&mut self.library, &self.profile, &self.status, msg);
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(CtxRequest::Storage) => {
                        self.status = CtxStatus::Ready;
                        match result.deref() {
                            Ok(_) => Effects::msg(Msg::Event(Event::CtxPulledFromStorage {
                                uid: self.profile.uid(),
                            }))
                            .unchanged(),
                            Err(error) => Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                source: Box::new(Event::CtxPulledFromStorage {
                                    uid: self.profile.uid(),
                                }),
                            }))
                            .unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                ctx_effects.join(profile_effects).join(library_effects)
            }
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => {
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, msg);
                let library_effects =
                    update_library::<Env>(&mut self.library, &self.profile, &self.status, msg);
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(CtxRequest::API(loading_auth_request))
                        if loading_auth_request == auth_request =>
                    {
                        self.status = CtxStatus::Ready;
                        match result {
                            Ok(_) => Effects::msg(Msg::Event(Event::UserAuthenticated {
                                uid: self.profile.uid(),
                                auth_request: auth_request.to_owned(),
                            }))
                            .unchanged(),
                            Err(error) => Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                source: Box::new(Event::UserAuthenticated {
                                    uid: self.profile.uid(),
                                    auth_request: auth_request.to_owned(),
                                }),
                            }))
                            .unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                ctx_effects.join(profile_effects).join(library_effects)
            }
            _ => {
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, &msg);
                let library_effects =
                    update_library::<Env>(&mut self.library, &self.profile, &self.status, &msg);
                profile_effects.join(library_effects)
            }
        }
    }
}
