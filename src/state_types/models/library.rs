use crate::state_types::*;
use crate::types::LibItem;
use crate::types::api::*;
use std::collections::HashMap;
use futures::{Future, future};
use super::{Auth, api_fetch};
use serde_derive::*;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};

const COLL_NAME: &str = "libraryItem";

/*
pub struct Library {
    // @TODO the state should be NotLoaded, Loading, Ready; so that when we dispatch LibSync we
    // can ensure we've waited for storage load first
    // perhaps wrap it on a Ctx level, and have an effect for loading from storage here; this
    // effect can either fail massively (CtxFatal) or succeed
    // when the user is logged out, we'll reset it to NotLoaded
    pub items: HashMap<String, LibItem>,
    pub last_videos: Vec<MetaDetail>,
}
*/
pub type LibraryIndex = HashMap<String, LibItem>;

#[derive(Debug, Deserialize)]
struct LibMTime(String, #[serde(with = "ts_milliseconds")] DateTime<Utc>);

// Implementing Auth here is a bit unconventional,
// but rust allows multiple impl blocks precisely to allow
// separation of concerns like this
impl Auth {
    pub fn lib_update(&mut self, items: &[LibItem]) {
        for item in items.iter() {
            self.lib.insert(item.id.to_owned(), item.to_owned());
        }
    }
    // @TODO rather than EnvFuture, use a Future that returns CtxError
    pub fn lib_sync<Env: Environment + 'static>(&self) -> impl Future<Item = Vec<LibItem>, Error = CtxError> {
        let local_lib = self.lib.clone();
        let key = self.key.clone();
        let api_req = APIRequest::DatastoreMeta {
            auth_key: key.clone(),
            collection: COLL_NAME.into(),
        };
        let ft = api_fetch::<Env, Vec<LibMTime>>(api_req).and_then(move |remote_mtimes| {
            let map_remote = remote_mtimes
                .into_iter()
                .map(|LibMTime(k, mtime)| (k, mtime))
                .collect::<HashMap<_, _>>();
            let to_pull_ids = map_remote
                .iter()
                .filter(|(k, v)| local_lib.get(*k).map_or(true, |item| item.mtime < **v))
                .map(|(k, _)| k.to_owned())
                .collect::<Vec<String>>();
            let to_push = local_lib
                .iter()
                .filter(|(id, item)| {
                    map_remote.get(*id).map_or(true, |date| *date < item.mtime)
                })
                .map(|(_, v)| v.to_owned())
                .collect::<Vec<LibItem>>();
            let push_req = APIRequest::DatastorePut {
                auth_key: key.clone(),
                collection: COLL_NAME.into(),
                changes: to_push
            };
            let pull_req = APIRequest::DatastoreGet {
                auth_key: key.clone(),
                collection: COLL_NAME.into(),
                all: false,
                ids: to_pull_ids
            };
            api_fetch::<Env, Vec<LibItem>>(pull_req)
                .join(api_fetch::<Env, SuccessResponse>(push_req))
                .map(|(items, _)| items)
        });
        Box::new(ft)
    }
    fn lib_push(&self, item: &LibItem) -> impl Future<Item = (), Error = CtxError> {
        unimplemented!();
        future::ok(())
    }
    fn lib_pull(&self, id: &str) -> impl Future<Item = Option<LibItem>, Error = CtxError> {
        unimplemented!();
        future::ok(None)
    }
}
