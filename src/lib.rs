pub mod addon_transport;
pub mod state_types;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::addon_transport::*;
    use crate::state_types::messages::*;
    use crate::state_types::models::common::Loadable;
    use crate::state_types::models::*;
    use crate::state_types::*;
    use crate::types::addons::{Descriptor, ResourceRef, ResourceRequest, ResourceResponse};
    use crate::types::MetaPreview;
    use futures::future::lazy;
    use futures::{future, Future};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use tokio::executor::current_thread::spawn;
    use tokio::runtime::current_thread::run;

    #[test]
    fn transport_manifests() {
        run(lazy(|| {
            let cinemeta_url = "https://v3-cinemeta.strem.io/manifest.json";
            let legacy_url = "https://opensubtitles.strem.io/stremioget/stremio/v1";
            let fut1 = AddonHTTPTransport::<Env>::from_url(&cinemeta_url)
                .manifest()
                .then(|res| {
                    if let Err(e) = res {
                        panic!("failed getting cinemeta manifest {:?}", e);
                    }
                    future::ok(())
                });
            let fut2 = AddonHTTPTransport::<Env>::from_url(&legacy_url)
                .manifest()
                .then(|res| {
                    if let Err(e) = res {
                        panic!("failed getting legacy manifest {:?}", e);
                    }
                    future::ok(())
                });
            fut1.join(fut2).map(|(_, _)| ())
        }));
    }

    #[test]
    fn get_videos() {
        run(lazy(|| {
            let transport_url = "https://v3-cinemeta.strem.io/manifest.json";
            AddonHTTPTransport::<Env>::from_url(&transport_url)
                .get(&ResourceRef::without_extra("meta", "series", "tt0386676"))
                .then(|res| {
                    match res {
                        Err(e) => panic!("failed getting metadata {:?}", e),
                        Ok(ResourceResponse::Meta { meta }) => {
                            //dbg!(&meta.videos);
                            assert!(meta.videos.len() > 0, "has videos")
                        }
                        _ => panic!("unexpected response"),
                    };
                    future::ok(())
                })
        }));
    }

    #[test]
    fn addon_collection() {
        run(lazy(|| {
            let collection_url = "https://api.strem.io/addonscollection.json";
            let req = Request::get(collection_url)
                .body(())
                .expect("builder cannot fail");
            Env::fetch_serde::<_, Vec<Descriptor>>(req).then(|res| {
                match res {
                    Err(e) => panic!("failed getting addon collection {:?}", e),
                    Ok(collection) => assert!(collection.len() > 0, "has addons"),
                };
                future::ok(())
            })
        }));
    }

    #[test]
    fn sample_storage() {
        let key = "foo".to_owned();
        let value = "fooobar".to_owned();
        // Notihng in the beginning
        assert!(Env::get_storage::<String>(&key).wait().unwrap().is_none());
        // Then set and read
        // with RwLock and BTreeMap, set_storage takes 73993042ns for 10000 iterations (or 74ms)
        //  get_storage takes 42076632 (or 42ms) for 10000 iterations
        assert_eq!(Env::set_storage(&key, Some(&value)).wait().unwrap(), ());
        assert_eq!(
            Env::get_storage::<String>(&key).wait().unwrap(),
            Some(value)
        );
    }

    #[test]
    fn stremio_derive() {
        // Implement some dummy Ctx and contents
        struct Ctx {};
        impl Update for Ctx {
            fn update(&mut self, _: &Msg) -> Effects {
                dummy_effect()
            }
        }
        struct Content {};
        impl UpdateWithCtx<Ctx> for Content {
            fn update(&mut self, _: &Ctx, _: &Msg) -> Effects {
                dummy_effect()
            }
        }

        use stremio_derive::Model;
        #[derive(Model)]
        struct Model {
            pub ctx: Ctx,
            pub one: Content,
            pub two: Content,
        }
        let mut m = Model {
            ctx: Ctx {},
            one: Content {},
            two: Content {},
        };
        let fx = m.update(&Msg::Action(Action::LoadCtx));
        assert!(fx.has_changed, "has changed");
        assert_eq!(fx.effects.len(), 3, "proper number of effects");
    }
    fn dummy_effect() -> Effects {
        Effects::one(Box::new(future::ok(Msg::Action(Action::LoadCtx))))
    }

    // Testing the CatalogsWithExtra model
    // and the Runtime type
    #[test]
    fn catalog_with_extra() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogsWithExtra,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // Run a single dispatch of a Load msg
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra {
            extra: vec![],
        }));
        run(runtime.dispatch(&msg));
        // since this is after the .run() has ended, this means all async effects
        // have processed
        {
            let state = &runtime.app.read().unwrap().catalogs;
            assert_eq!(state.catalogs.len(), 7, "groups is the right length");
            for g in state.catalogs.iter() {
                assert!(
                    match g.content {
                        Loadable::Ready(_) => true,
                        Loadable::Err(_) => true,
                        _ => false,
                    },
                    "group is Ready or Err"
                );
            }
        }

        // Now try the same, but with Search
        let extra = vec![("search".to_owned(), "grand tour".to_owned())];
        let msg = Msg::Action(Action::Load(ActionLoad::CatalogsWithExtra { extra }));
        run(runtime.dispatch(&msg));
        assert_eq!(
            runtime.app.read().unwrap().catalogs.catalogs.len(),
            5,
            "groups is the right length when searching"
        );
    }

    #[test]
    fn catalog_filtered() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            catalogs: CatalogFiltered<MetaPreview>,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        let req = ResourceRequest {
            base: "https://v3-cinemeta.strem.io/manifest.json".to_owned(),
            path: ResourceRef::without_extra("catalog", "movie", "top"),
        };
        let action = Action::Load(ActionLoad::CatalogFiltered(req.to_owned()));
        run(runtime.dispatch_with(|model| model.catalogs.update(&model.ctx, &action.into())));
        // Clone the state so that we don't keep a lock on .app
        let state = runtime.app.read().unwrap().catalogs.to_owned();
        assert!(state.types[0].is_selected, "first type is selected");
        assert!(state.catalogs[0].is_selected, "first catalog is selected");
        assert_eq!(state.types[0].type_name, "movie", "first type is movie");
        assert_eq!(state.types[1].type_name, "series", "second type is series");
        assert!(state.catalogs.len() > 3, "has catalogs");
        assert_eq!(state.selected, Some(req), "selected is right");
        match &state.content {
            Loadable::Ready(x) => assert_eq!(x.len(), 100, "right length of items"),
            x => panic!("item_pages[0] is not Ready, but instead: {:?}", x),
        }

        // Verify that pagination works
        let load_next = state
            .load_next
            .as_ref()
            .expect("there should be a next page")
            .to_owned();
        let action = Action::Load(ActionLoad::CatalogFiltered(load_next));
        run(runtime.dispatch(&action.into()));
        let state = &runtime.app.read().unwrap().catalogs.to_owned();
        assert!(state.types[0].is_selected, "first type is selected");
        assert!(
            state.catalogs[0].is_selected,
            "first catalog is still selected"
        );
        assert_eq!(
            state
                .selected
                .as_ref()
                .expect("there must be .selected")
                .path
                .get_extra_first_val("skip"),
            Some("100")
        );
        assert_eq!(
            state
                .load_next
                .as_ref()
                .expect("there must be .load_next")
                .path
                .get_extra_first_val("skip"),
            Some("200")
        );
        assert_eq!(
            state
                .load_prev
                .as_ref()
                .expect("there must be .load_prev")
                .path
                .get_extra_first_val("skip"),
            Some("0")
        );

        //dbg!(&state);

        let year_catalog = state
            .catalogs
            .iter()
            .find(|c| c.name == "By year")
            .expect("could not find year catalog");
        let action = Action::Load(ActionLoad::CatalogFiltered(year_catalog.load.to_owned()));
        run(runtime.dispatch(&action.into()));
        let state = &runtime.app.read().unwrap().catalogs;
        let selected = state.selected.as_ref().expect("should have selected");
        assert_eq!(
            selected.path.get_extra_first_val("skip"),
            None,
            "first page"
        );
        assert_eq!(selected.path.id, "year", "year catalog is loaded");
        // The year catalog is not seekable
        assert_eq!(state.load_prev, None);
        assert_eq!(state.load_next, None);
    }

    #[test]
    fn streams() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            meta_details: MetaDetails,
        }

        let app = Model::default();
        let (runtime, _) = Runtime::<Env, Model>::new(app, 1000);

        // If we login here with some dummy account, we can use this pretty nicely
        //let action = Action::UserOp(ActionUser::Login { email, password });
        //run(runtime.dispatch(&action.into()));

        // @TODO install some addons that provide streams
        let action = Action::Load(ActionLoad::MetaDetails {
            type_name: "series".to_string(),
            id: "tt0773262".to_string(),
            video_id: Some("tt0773262:6:1".to_string()),
        });
        run(runtime.dispatch(&action.into()));
        let state = &runtime.app.read().unwrap().meta_details;
        assert_eq!(state.streams_groups.len(), 2, "2 groups");
    }

    #[test]
    fn ctx_and_lib() {
        use stremio_derive::Model;
        #[derive(Model, Debug, Default)]
        struct Model {
            ctx: Ctx<Env>,
            lib_recent: LibRecent,
            notifs: Notifications,
        }
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);

        // Log into a user, check if library synced correctly
        // We are always support to start with LoadCtx, even though it is not needed here
        run(runtime.dispatch(&Msg::Action(Action::LoadCtx)));

        // if this user gets deleted, the test will fail
        // @TODO register a new user instead
        let login_msg: Msg = Action::UserOp(ActionUser::Login {
            email: "ctxandlib@stremio.com".into(),
            password: "ctxandlib".into(),
        })
        .into();
        run(runtime.dispatch(&login_msg));
        // @TODO test if the addon collection is pulled
        let model = &runtime.app.read().unwrap();
        let first_content = model.ctx.content.to_owned();
        let first_lib = if let LibraryLoadable::Ready(l) = &model.ctx.library {
            assert!(!l.items.is_empty(), "library has items");
            // LibRecent is "continue watching"
            assert!(!model.lib_recent.recent.is_empty(), "has recent items");
            l.to_owned()
        } else {
            panic!("library must be Ready")
        };

        // New runtime, just LoadCtx, to see if the ctx was persisted
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
        assert_eq!(runtime.app.read().unwrap().ctx.is_loaded, false);
        run(runtime.dispatch(&Msg::Action(Action::LoadCtx)));
        {
            let ctx = &runtime.app.read().unwrap().ctx;
            assert_eq!(&first_content, &ctx.content, "content is the same");
            if let LibraryLoadable::Ready(l) = &ctx.library {
                assert_eq!(&first_lib, l, "loaded lib is same as synced");
            } else {
                panic!("library must be Ready")
            }
            assert!(ctx.is_loaded);
        }

        // Update notifications
        {
            // ¯\_(ツ)_/¯
            // temporary hack (really) until last-videos catalog lands in upstream cinemeta
            // and gets updated for our user
            let addons: Vec<Descriptor> =
                serde_json::from_slice(include_bytes!("../stremio-official-addons/index.json"))
                    .expect("official addons JSON parse");
            runtime.app.write().unwrap().ctx.content.addons[0] = addons[0].clone();
            // we did unspeakable things, now dispatch the load action
            run(runtime.dispatch(&Action::Load(ActionLoad::Notifications).into()));
            // ...
            let model = &runtime.app.read().unwrap();
            assert_eq!(model.notifs.groups.len(), 1);
            let meta_items = match &model.notifs.groups[0].content {
                Loadable::Ready(x) => x,
                x => panic!("notifs group not ready, but instead: {:?}", x),
            };
            assert!(meta_items.len() > 1, "should have loaded multiple items");
            // No notifications, cause neither LibItem has .last_vid_released set
            assert!(meta_items.iter().all(|x| x.videos.len() == 0));
        }

        // Logout and expect everything to be reset
        let logout_action = Action::UserOp(ActionUser::Logout);
        run(runtime.dispatch(&logout_action.into()));
        {
            let model = &runtime.app.read().unwrap();
            assert!(model.ctx.content.auth.is_none(), "logged out");
            assert!(model.ctx.content.addons.len() > 0, "has addons");
            if let LibraryLoadable::Ready(l) = &model.ctx.library {
                assert!(l.items.is_empty(), "library must be empty");
                assert!(model.lib_recent.recent.is_empty(), "is empty");
            } else {
                panic!("library must be Ready")
            }
        }

        // Addon updating in anon mode works
        let zero_ver = semver::Version::new(0, 0, 0);
        {
            let addons = &mut runtime.app.write().unwrap().ctx.content.addons;
            addons[0].manifest.version = zero_ver.clone();
            addons[0].flags.extra.insert("foo".into(), "bar".into());
            assert_eq!(&addons[0].manifest.version, &zero_ver);
        }
        let update_action = Action::UserOp(ActionUser::PullAndUpdateAddons);
        run(runtime.dispatch(&update_action.into()));
        {
            let model = &runtime.app.read().unwrap();
            let first_addon = &model.ctx.content.addons[0];
            let expected_val = serde_json::Value::String("bar".into());
            assert_ne!(&first_addon.manifest.version, &zero_ver);
            assert_eq!(first_addon.flags.extra.get("foo"), Some(&expected_val));
        }

        // we will now add an item for the anon user
        let item = first_lib.items.values().next().unwrap().to_owned();
        run(runtime.dispatch(&Msg::Action(Action::UserOp(ActionUser::LibUpdate(item)))));
        // take a copy now so we can compare later
        let anon_lib = runtime.app.read().unwrap().ctx.library.to_owned();

        // we will load again to make sure it's persisted
        let (runtime, _) = Runtime::<Env, Model>::new(Model::default(), 1000);
        run(runtime.dispatch(&Msg::Action(Action::LoadCtx)));
        {
            let ctx = &runtime.app.read().unwrap().ctx;
            assert_eq!(anon_lib, ctx.library);
            if let LibraryLoadable::Ready(l) = &ctx.library {
                assert_eq!(l.items.len(), 1, "library has 1 item");
            } else {
                panic!("library must be Ready")
            }
        }
    }

    // Storage implementation
    // Uses reqwest (asynchronously) for fetch, and a BTreeMap storage
    use lazy_static::*;
    use std::collections::BTreeMap;
    use std::sync::RwLock;
    lazy_static! {
        static ref STORAGE: RwLock<BTreeMap<String, String>> = { Default::default() };
    }
    struct Env {}
    impl Environment for Env {
        fn fetch_serde<IN, OUT>(in_req: Request<IN>) -> EnvFuture<OUT>
        where
            IN: 'static + Serialize,
            OUT: 'static + DeserializeOwned,
        {
            let (parts, body) = in_req.into_parts();
            let method = reqwest::Method::from_bytes(parts.method.as_str().as_bytes())
                .expect("method is not valid for reqwest");
            let mut req = reqwest::r#async::Client::new().request(method, &parts.uri.to_string());
            // NOTE: both might be HeaderMap, so maybe there's a better way?
            for (k, v) in parts.headers.iter() {
                req = req.header(k.as_str(), v.as_ref());
            }
            // @TODO add content-type application/json
            // @TODO: if the response code is not 200, return an error related to that
            req = req.json(&body);
            let fut = req
                .send()
                .and_then(|mut res: reqwest::r#async::Response| res.json::<OUT>())
                .map_err(|e| e.into());
            Box::new(fut)
        }
        fn exec(fut: Box<dyn Future<Item = (), Error = ()>>) {
            spawn(fut);
        }
        fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<T>> {
            Box::new(future::ok(
                STORAGE
                    .read()
                    .unwrap()
                    .get(key)
                    .map(|v| serde_json::from_str(&v).unwrap()),
            ))
        }
        fn set_storage<T: Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()> {
            let mut storage = STORAGE.write().unwrap();
            match value {
                Some(v) => storage.insert(key.to_string(), serde_json::to_string(v).unwrap()),
                None => storage.remove(key),
            };
            Box::new(future::ok(()))
        }
    }
}
