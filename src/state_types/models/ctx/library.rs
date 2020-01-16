use super::UserDataLoadable;
use crate::constants::{
    LIBRARY_COLLECTION_NAME, LIBRARY_RECENT_COUNT, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
};
use crate::state_types::messages::{
    Action, ActionCtx, ActionLibrary, Event, Internal, Msg, MsgError,
};
use crate::state_types::models::common::fetch_api;
use crate::state_types::{Effects, Environment};
use crate::types::api::{Auth, DatastoreCmd, DatastoreReq, DatastoreReqBuilder, SuccessResponse};
use crate::types::{LibBucket, LibItem, LibItemModified, LibItemState, UID};
use chrono::Datelike;
use derivative::Derivative;
use futures::future::Either;
use futures::{future, Future};
use lazysort::SortedBy;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub enum LibraryRequest {
    Storage,
    API,
}

#[derive(Derivative, Debug, Clone, PartialEq)]
#[derivative(Default)]
pub enum LibraryLoadable {
    #[derivative(Default)]
    NotLoaded,
    Loading(UID, LibraryRequest),
    Ready(LibBucket),
}

impl LibraryLoadable {
    pub fn update<Env: Environment + 'static>(
        &mut self,
        user_data: &UserDataLoadable,
        msg: &Msg,
    ) -> Effects {
        let uid_changed = match (user_data.auth(), &self) {
            (None, LibraryLoadable::Loading(_, _))
            | (None, LibraryLoadable::Ready(_))
            | (Some(_), LibraryLoadable::NotLoaded) => true,
            (Some(auth), LibraryLoadable::Loading(uid, _))
            | (Some(auth), LibraryLoadable::Ready(LibBucket { uid, .. })) => {
                uid.ne(&UID(Some(auth.user.id.to_owned())))
            }
            _ => false,
        };
        if uid_changed {
            *self = LibraryLoadable::NotLoaded;
        };
        let library_effects = match msg {
            Msg::Event(Event::UserDataRetrivedFromStorage) => {
                let uid = UID(user_data.auth().map(|auth| auth.user.id));
                *self = LibraryLoadable::Loading(uid.to_owned(), LibraryRequest::Storage);
                Effects::one(Box::new(
                    Env::get_storage(LIBRARY_RECENT_STORAGE_KEY)
                        .join(Env::get_storage(LIBRARY_STORAGE_KEY))
                        .map(move |(recent_bucket, other_bucket)| {
                            Msg::Internal(Internal::LibraryStorageResponse(
                                uid,
                                Box::new(recent_bucket),
                                Box::new(other_bucket),
                            ))
                        })
                        .map_err(|error| Msg::Event(Event::Error(MsgError::from(error)))),
                ))
            }
            Msg::Event(Event::UserLoggedIn)
            | Msg::Event(Event::UserRegistered)
            | Msg::Event(Event::UserLoggedOut) => match user_data.auth() {
                Some(auth) => {
                    let uid = UID(Some(auth.user.id.to_owned()));
                    *self = LibraryLoadable::Loading(uid.to_owned(), LibraryRequest::API);
                    let library_request = DatastoreReq {
                        auth_key: auth.key.to_owned(),
                        collection: LIBRARY_COLLECTION_NAME.to_owned(),
                        cmd: DatastoreCmd::Get {
                            ids: vec![],
                            all: true,
                        },
                    };
                    Effects::one(Box::new(
                        fetch_api::<Env, _, _>(&library_request)
                            .map(move |items| {
                                Msg::Internal(Internal::LibraryAPIResponse(Box::new(
                                    LibBucket::new(uid, items),
                                )))
                            })
                            .map_err(|error| Msg::Event(Event::Error(error))),
                    ))
                }
                _ => {
                    *self = LibraryLoadable::Ready(LibBucket::default());
                    Effects::none()
                }
            },
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::SyncWithAPI))) => {
                match (user_data.auth(), &self) {
                    (Some(auth), LibraryLoadable::Ready(bucket))
                        if bucket.uid.eq(&UID(Some(auth.user.id.to_owned()))) =>
                    {
                        Effects::one(Box::new(
                            lib_sync::<Env>(auth, bucket.to_owned())
                                .map(|bucket| {
                                    Msg::Internal(Internal::LibrarySyncResponse(Box::new(bucket)))
                                })
                                .map_err(|error| Msg::Event(Event::Error(error))),
                        ))
                        .unchanged()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::Add { meta_item, now }))) => {
                let mut lib_item = LibItem {
                    id: meta_item.id.to_owned(),
                    type_name: meta_item.type_name.to_owned(),
                    name: meta_item.name.to_owned(),
                    poster: meta_item.poster.to_owned(),
                    poster_shape: meta_item.poster_shape.to_owned(),
                    logo: meta_item.logo.to_owned(),
                    background: None,
                    year: if let Some(released) = &meta_item.released {
                        Some(released.year().to_string())
                    } else if let Some(release_info) = &meta_item.release_info {
                        Some(release_info.to_owned())
                    } else {
                        None
                    },
                    ctime: Some(now.to_owned()),
                    mtime: now.to_owned(),
                    removed: false,
                    temp: false,
                    state: LibItemState::default(),
                };
                if let Some(LibItem { ctime, state, .. }) = self.get_item(&meta_item.id) {
                    lib_item.state = state.to_owned();
                    if let Some(ctime) = ctime {
                        lib_item.ctime = Some(ctime.to_owned());
                    };
                };
                Effects::msg(Msg::Action(Action::Ctx(ActionCtx::Library(
                    ActionLibrary::Update(Box::new(lib_item)),
                ))))
                .unchanged()
            }
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::Remove { id, now }))) => {
                match &mut self {
                    LibraryLoadable::Ready(bucket) => {
                        if let Some(mut lib_item) = bucket.items.get(id).cloned() {
                            lib_item.mtime = now.to_owned();
                            lib_item.removed = true;
                            Effects::msg(Msg::Action(Action::Ctx(ActionCtx::Library(
                                ActionLibrary::Update(Box::new(lib_item)),
                            ))))
                            .unchanged()
                        } else {
                            Effects::none().unchanged()
                        }
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Action(Action::Ctx(ActionCtx::Library(ActionLibrary::Update(lib_item)))) => {
                match &mut self {
                    LibraryLoadable::Ready(bucket) => {
                        let uid = UID(user_data.auth().map(|auth| auth.user.id));
                        let persist_library_effects = Effects::one(Box::new(
                            update_and_persist::<Env>(
                                bucket,
                                LibBucket::new(uid, vec![lib_item.deref().to_owned()]),
                            )
                            .map(move |_| Msg::Event(Event::LibraryPersisted))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                        ));
                        if let Some(auth) = user_data.auth() {
                            persist_library_effects.join(Effects::one(Box::new(
                                lib_push::<Env>(auth, &lib_item)
                                    .map(|_| Msg::Event(Event::LibraryPushed))
                                    .map_err(|error| Msg::Event(Event::Error(error))),
                            )))
                        } else {
                            persist_library_effects
                        }
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::LibraryStorageResponse(uid, recent_bucket, other_bucket)) => {
                match &self {
                    LibraryLoadable::Loading(loading_uid, LibraryRequest::Storage)
                        if loading_uid.eq(uid) =>
                    {
                        let mut bucket = LibBucket::new(uid.to_owned(), vec![]);
                        if let Some(recent_bucket) = recent_bucket.deref().to_owned() {
                            bucket.merge(recent_bucket)
                        };
                        if let Some(other_bucket) = other_bucket.deref().to_owned() {
                            bucket.merge(other_bucket);
                        };
                        *self = LibraryLoadable::Ready(bucket);
                        Effects::none()
                    }
                    _ => Effects::none().unchanged(),
                }
            }
            Msg::Internal(Internal::LibraryAPIResponse(bucket)) => match &self {
                LibraryLoadable::Loading(loading_uid, LibraryRequest::API)
                    if loading_uid.eq(&bucket.uid) =>
                {
                    *self = LibraryLoadable::Ready(bucket.deref().to_owned());
                    Effects::none()
                }
                _ => Effects::none().unchanged(),
            },
            Msg::Internal(Internal::LibrarySyncResponse(sync_bucket)) => match &mut self {
                LibraryLoadable::Ready(bucket) if bucket.uid.eq(&sync_bucket.uid) => {
                    Effects::msg(Msg::Event(Event::LibrarySynced)).join(Effects::one(Box::new(
                        update_and_persist::<Env>(&mut bucket, sync_bucket.deref().to_owned())
                            .map(move |_| Msg::Event(Event::LibraryPersisted))
                            .map_err(|error| Msg::Event(Event::Error(error))),
                    )))
                }
                _ => Effects::none().unchanged(),
            },
            _ => Effects::none().unchanged(),
        };
        if uid_changed || library_effects.has_changed {
            Effects::msg(Msg::Internal(Internal::LibraryChanged)).join(library_effects)
        } else {
            library_effects
        }
    }
    pub fn get_item(&self, id: &str) -> Option<&LibItem> {
        match self {
            LibraryLoadable::Ready(bucket) => bucket.items.get(id),
            _ => None,
        }
    }
}

fn datastore_req_builder(auth: &Auth) -> DatastoreReqBuilder {
    DatastoreReqBuilder::default()
        .auth_key(auth.key.to_owned())
        .collection(LIBRARY_COLLECTION_NAME.to_owned())
        .clone()
}

fn lib_sync<Env: Environment + 'static>(
    auth: &Auth,
    local_lib: LibBucket,
) -> impl Future<Item = LibBucket, Error = MsgError> {
    // @TODO consider asserting if uid matches auth
    let builder = datastore_req_builder(auth);
    let meta_req = builder.clone().with_cmd(DatastoreCmd::Meta {});

    fetch_api::<Env, Vec<LibItemModified>, _>(&meta_req).and_then(move |remote_mtimes| {
        let map_remote = remote_mtimes
            .into_iter()
            .map(|LibItemModified(k, mtime)| (k, mtime))
            .collect::<std::collections::HashMap<_, _>>();
        // IDs to pull
        let ids = map_remote
            .iter()
            .filter(|(k, v)| {
                local_lib
                    .items
                    .get(*k)
                    .map_or(true, |item| item.mtime < **v)
            })
            .map(|(k, _)| k.clone())
            .collect::<Vec<String>>();
        // Items to push
        let LibBucket { items, uid } = local_lib;
        let changes = items
            .into_iter()
            .filter(|(id, item)| {
                map_remote.get(id).map_or(true, |date| *date < item.mtime) && item.should_push()
            })
            .map(|(_, v)| v)
            .collect::<Vec<LibItem>>();

        let get_fut = if ids.is_empty() {
            Either::A(future::ok(vec![]))
        } else {
            Either::B(fetch_api::<Env, Vec<LibItem>, _>(
                &builder
                    .clone()
                    .with_cmd(DatastoreCmd::Get { ids, all: false }),
            ))
        };

        let put_fut = if changes.is_empty() {
            Either::A(future::ok(()))
        } else {
            Either::B(
                fetch_api::<Env, SuccessResponse, _>(
                    &builder.clone().with_cmd(DatastoreCmd::Put { changes }),
                )
                .map(|_| ()),
            )
        };

        get_fut
            .join(put_fut)
            .map(move |(items, _)| LibBucket::new(uid, items))
    })
}

fn lib_push<Env: Environment + 'static>(
    auth: &Auth,
    item: &LibItem,
) -> impl Future<Item = (), Error = MsgError> {
    let push_req = datastore_req_builder(auth).with_cmd(DatastoreCmd::Put {
        changes: vec![item.to_owned()],
    });

    fetch_api::<Env, SuccessResponse, _>(&push_req).map(|_| ())
}

fn update_and_persist<Env: Environment + 'static>(
    bucket: &mut LibBucket,
    new_bucket: LibBucket,
) -> impl Future<Item = (), Error = MsgError> {
    let recent_iter = bucket
        .items
        .values()
        .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
        .take(LIBRARY_RECENT_COUNT);
    let are_new_items_in_recent = new_bucket
        .items
        .keys()
        .all(move |id| recent_iter.any(|item| item.id.eq(id)));
    bucket.merge(new_bucket);
    if bucket.items.len() <= LIBRARY_RECENT_COUNT {
        Either::A(
            Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(bucket))
                .join(Env::set_storage(LIBRARY_STORAGE_KEY, None))
                .map(|(_, _)| ())
                .map_err(|error| MsgError::from(error)),
        )
    } else {
        let (recent_bucket, other_bucket) = bucket.split_by_recent();
        if are_new_items_in_recent {
            Either::B(Either::A(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .map_err(|error| MsgError::from(error)),
            ))
        } else {
            Either::B(Either::B(
                Env::set_storage(LIBRARY_RECENT_STORAGE_KEY, Some(&recent_bucket))
                    .join(Env::set_storage(LIBRARY_STORAGE_KEY, Some(&other_bucket)))
                    .map(|(_, _)| ())
                    .map_err(|error| MsgError::from(error)),
            ))
        }
    }
}
