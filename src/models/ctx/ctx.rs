use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY,
};
use crate::models::ctx::{
    fetch_api, migrate_storage_schema, update_library, update_profile, CtxError,
};
use crate::runtime::msg::{Action, ActionCtx, Event, Internal, Msg};
use crate::runtime::{Effect, Effects, Environment, Update};
use crate::types::api::{
    APIRequest, AuthRequest, AuthResponse, CollectionResponse, DatastoreCommand, DatastoreRequest,
    SuccessResponse,
};
use crate::types::library::LibBucket;
use crate::types::profile::{Auth, Profile};
use derivative::Derivative;
use enclose::enclose;
use futures::{future, FutureExt, TryFutureExt};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::marker::PhantomData;

#[derive(PartialEq)]
pub enum CtxRequest {
    Storage,
    API(AuthRequest),
}

#[derive(Derivative, PartialEq)]
#[derivative(Default)]
pub enum CtxStatus {
    Loading(CtxRequest),
    #[derivative(Default)]
    Ready,
}

#[derive(Derivative, Serialize)]
#[derivative(Default)]
pub struct Ctx<Env: Environment> {
    pub profile: Profile,
    // TODO StreamsBucket
    // TODO SubtitlesBucket
    // TODO SearchesBucket
    #[serde(serialize_with = "serialize_lib_bucket")]
    pub library: LibBucket,
    #[serde(skip)]
    pub status: CtxStatus,
    #[serde(skip)]
    pub env: PhantomData<Env>,
}

impl<Env: Environment + 'static> Update for Ctx<Env> {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Ctx(ActionCtx::PullFromStorage)) => {
                self.status = CtxStatus::Loading(CtxRequest::Storage);
                Effects::one(pull_ctx_from_storage::<Env>()).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Authenticate(auth_request))) => {
                self.status = CtxStatus::Loading(CtxRequest::API(auth_request.to_owned()));
                Effects::one(authenticate::<Env>(auth_request)).unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Logout)) => {
                let uid = self.profile.uid();
                let session_effects = match &self.profile.auth {
                    Some(auth) => Effects::one(delete_session::<Env>(&auth.key)).unchanged(),
                    _ => Effects::none().unchanged(),
                };
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<Env>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    &msg,
                );
                self.status = CtxStatus::Ready;
                Effects::msg(Msg::Event(Event::UserLoggedOut { uid }))
                    .unchanged()
                    .join(session_effects)
                    .join(profile_effects)
                    .join(library_effects)
            }
            Msg::Internal(Internal::CtxStorageResult(result)) => {
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<Env>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    msg,
                );
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(CtxRequest::Storage) => {
                        self.status = CtxStatus::Ready;
                        match result {
                            Ok(_) => Effects::msg(Msg::Event(Event::CtxPulledFromStorage {
                                uid: self.profile.uid(),
                            }))
                            .unchanged(),
                            Err(error) => Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                source: Box::new(Event::CtxPulledFromStorage {
                                    uid: Default::default(),
                                }),
                            }))
                            .unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                profile_effects.join(library_effects).join(ctx_effects)
            }
            Msg::Internal(Internal::CtxAuthResult(auth_request, result)) => {
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, msg);
                let library_effects = update_library::<Env>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    msg,
                );
                let ctx_effects = match &self.status {
                    CtxStatus::Loading(CtxRequest::API(loading_auth_request))
                        if loading_auth_request == auth_request =>
                    {
                        self.status = CtxStatus::Ready;
                        match result {
                            Ok(_) => Effects::msg(Msg::Event(Event::UserAuthenticated {
                                auth_request: auth_request.to_owned(),
                            }))
                            .unchanged(),
                            Err(error) => Effects::msg(Msg::Event(Event::Error {
                                error: error.to_owned(),
                                source: Box::new(Event::UserAuthenticated {
                                    auth_request: auth_request.to_owned(),
                                }),
                            }))
                            .unchanged(),
                        }
                    }
                    _ => Effects::none().unchanged(),
                };
                profile_effects.join(library_effects).join(ctx_effects)
            }
            _ => {
                let profile_effects = update_profile::<Env>(&mut self.profile, &self.status, &msg);
                let library_effects = update_library::<Env>(
                    &mut self.library,
                    self.profile.auth.as_ref().map(|auth| &auth.key),
                    &self.status,
                    &msg,
                );
                profile_effects.join(library_effects)
            }
        }
    }
}

fn pull_ctx_from_storage<Env: Environment + 'static>() -> Effect {
    migrate_storage_schema::<Env>()
        .and_then(|_| {
            future::try_join3(
                Env::get_storage(PROFILE_STORAGE_KEY),
                Env::get_storage(LIBRARY_RECENT_STORAGE_KEY),
                Env::get_storage(LIBRARY_STORAGE_KEY),
            )
        })
        .map_err(CtxError::from)
        .map(|result| Msg::Internal(Internal::CtxStorageResult(result)))
        .boxed_local()
        .into()
}

fn authenticate<Env: Environment + 'static>(auth_request: &AuthRequest) -> Effect {
    fetch_api::<Env, _, _>(&APIRequest::Auth(auth_request.to_owned()))
        .map_ok(|AuthResponse { key, user }| Auth { key, user })
        .and_then(|auth| {
            future::try_join(
                fetch_api::<Env, _, _>(&APIRequest::AddonCollectionGet {
                    auth_key: auth.key.to_owned(),
                    update: true,
                })
                .map_ok(|CollectionResponse { addons, .. }| addons),
                fetch_api::<Env, _, _>(&DatastoreRequest {
                    auth_key: auth.key.to_owned(),
                    collection: LIBRARY_COLLECTION_NAME.to_owned(),
                    command: DatastoreCommand::Get {
                        ids: vec![],
                        all: true,
                    },
                }),
            )
            .map_ok(move |(addons, lib_items)| (auth, addons, lib_items))
        })
        .map(enclose!((auth_request) move |result| {
            Msg::Internal(Internal::CtxAuthResult(auth_request, result))
        }))
        .boxed_local()
        .into()
}

fn delete_session<Env: Environment + 'static>(auth_key: &str) -> Effect {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::Logout {
        auth_key: auth_key.to_owned(),
    })
    .map(enclose!((auth_key.to_owned() => auth_key) move |result|
        match result {
            Ok(_) => Msg::Event(Event::SessionDeleted { auth_key }),
            Err(error) => Msg::Event(Event::Error {
                error,
                source: Box::new(Event::SessionDeleted { auth_key }),
            }),
        }
    ))
    .boxed_local()
    .into()
}

fn serialize_lib_bucket<S>(lib_bucket: &LibBucket, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct LibItemProjection {
        pub removed: bool,
        pub temp: bool,
    }
    let mut map = serializer.serialize_map(Some(lib_bucket.items.len()))?;
    for lib_item in lib_bucket.items.values() {
        map.serialize_entry(
            &lib_item.id,
            &LibItemProjection {
                removed: lib_item.removed,
                temp: lib_item.temp,
            },
        )?;
    }
    map.end()
}
