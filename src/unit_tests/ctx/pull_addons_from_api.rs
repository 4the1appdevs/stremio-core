use crate::constants::{OFFICIAL_ADDONS, PROFILE_STORAGE_KEY};
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Action, ActionCtx, Msg};
use crate::state_types::{EnvFuture, Environment, Runtime};
use crate::types::addons::{Descriptor, DescriptorFlags, Manifest};
use crate::types::api::{APIResult, Auth, CollectionResponse, User};
use crate::types::profile::Profile;
use crate::unit_tests::{default_fetch_handler, Env, Request, FETCH_HANDLER, REQUESTS, STORAGE};
use futures::future;
use semver::Version;
use std::any::Any;
use std::fmt::Debug;
use stremio_derive::Model;
use tokio::runtime::current_thread::run;

#[test]
fn actionctx_pulladdonsfromapi() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    let official_addon = OFFICIAL_ADDONS.first().unwrap().to_owned();
    Env::reset();
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    addons: vec![Descriptor {
                        manifest: Manifest {
                            version: Version::new(0, 0, 1),
                            ..official_addon.manifest.to_owned()
                        },
                        transport_url: "transport_url".to_owned(),
                        flags: DescriptorFlags {
                            extra: {
                                [("flag".to_owned(), serde_json::Value::Bool(true))]
                                    .iter()
                                    .cloned()
                                    .collect()
                            },
                            ..official_addon.flags.to_owned()
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons,
        vec![Descriptor {
            flags: DescriptorFlags {
                extra: {
                    [("flag".to_owned(), serde_json::Value::Bool(true))]
                        .iter()
                        .cloned()
                        .collect()
                },
                ..official_addon.flags.to_owned()
            },
            ..official_addon.to_owned()
        }],
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons
                    == vec![Descriptor {
                        flags: DescriptorFlags {
                            extra: {
                                [("flag".to_owned(), serde_json::Value::Bool(true))]
                                    .iter()
                                    .cloned()
                                    .collect()
                            },
                            ..official_addon.flags.to_owned()
                        },
                        ..official_addon.to_owned()
                    }]
            }),
        "addons updated successfully in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}

#[test]
fn actionctx_pulladdonsfromapi_with_user() {
    #[derive(Model, Debug, Default)]
    struct Model {
        ctx: Ctx<Env>,
    }
    fn fetch_handler(request: Request) -> EnvFuture<Box<dyn Any>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://api.strem.io/api/addonCollectionGet"
                && method == "POST"
                && body == "{\"type\":\"AddonCollectionGet\",\"authKey\":\"auth_key\",\"update\":true}" =>
            {
                Box::new(future::ok(Box::new(APIResult::Ok {
                    result: CollectionResponse {
                        addons: OFFICIAL_ADDONS.to_vec(),
                        last_modified: Env::now(),
                    },
                }) as Box<dyn Any>))
            }
            _ => default_fetch_handler(request),
        }
    }
    Env::reset();
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    let (runtime, _) = Runtime::<Env, Model>::new(
        Model {
            ctx: Ctx {
                profile: Profile {
                    auth: Some(Auth {
                        key: "auth_key".to_owned(),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: Env::now(),
                            date_registered: Env::now(),
                        },
                    }),
                    addons: vec![Descriptor {
                        manifest: Manifest {
                            id: "id".to_owned(),
                            version: Version::new(0, 0, 1),
                            name: "name".to_owned(),
                            contact_email: None,
                            description: None,
                            logo: None,
                            background: None,
                            types: vec![],
                            resources: vec![],
                            id_prefixes: None,
                            catalogs: vec![],
                            addon_catalogs: vec![],
                            behavior_hints: Default::default(),
                        },
                        transport_url: "transport_url".to_owned(),
                        flags: Default::default(),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        1000,
    );
    run(runtime.dispatch(&Msg::Action(Action::Ctx(ActionCtx::PullAddonsFromAPI))));
    assert_eq!(
        runtime.app.read().unwrap().ctx.profile.addons,
        OFFICIAL_ADDONS.to_vec(),
        "addons updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(PROFILE_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<Profile>(&data).unwrap().addons == OFFICIAL_ADDONS.to_vec()
            }),
        "addons updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().get(0).unwrap().url,
        "https://api.strem.io/api/addonCollectionGet".to_owned(),
        "addonCollectionGet request has been sent"
    );
}
