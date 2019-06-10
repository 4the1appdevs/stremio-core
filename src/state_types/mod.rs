mod environment;
pub use self::environment::*;

mod msg;
pub use self::msg::*;

mod effects;
pub use self::effects::*;

mod models;
pub use self::models::*;

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx {
    type Ctx;
    fn update(&mut self, ctx: &Self::Ctx, msg: &Msg) -> Effects;
}

use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use futures::future;
use futures::future::Future;
use msg::Internal::*;
// @TODO move loadable
// @TODO should this take &Descriptor too?
pub trait Group {
    fn new(req: ResourceRequest) -> Self;
    fn update(&mut self, res: &Result<ResourceResponse, EnvError>);
    fn addon_req(&self) -> &ResourceRequest;
}
pub fn addon_aggr_new<Env: Environment + 'static, G: Group>(
    addons: &[Descriptor],
    aggr_req: &AggrRequest,
) -> (Vec<G>, Effects) {
    let (effects, groups): (Vec<_>, Vec<_>) = aggr_req
        .plan(&addons)
        .into_iter()
        .map(|addon_req| (addon_get::<Env>(&addon_req), G::new(addon_req)))
        .unzip();
    (groups, Effects::many(effects))
}
pub fn addon_aggr_update<G: Group>(groups: &mut Vec<G>, msg: &Msg) -> Effects {
    match msg {
        Msg::Internal(AddonResponse(req, result)) => {
            if let Some(idx) = groups.iter().position(|g| g.addon_req() == req) {
                groups[idx].update(result);
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        _ => Effects::none().unchanged(),
    }
}
fn addon_get<Env: Environment + 'static>(req: &ResourceRequest) -> Effect {
    // we will need that, cause we have to move it into the closure
    let req = req.clone();
    Box::new(
        Env::addon_transport(&req.base)
            .get(&req.path)
            .then(move |res| match res {
                Ok(_) => future::ok(AddonResponse(req, Box::new(res)).into()),
                Err(_) => future::err(AddonResponse(req, Box::new(res)).into()),
            }),
    )
}

// CatalogFiltered
use crate::types::MetaPreview;
use serde_derive::*;
const UNEXPECTED_RESP_MSG: &str = "unexpected ResourceResponse";
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    Loading,
    Ready(R),
    Err(E),
}

use std::sync::Arc;
struct CatalogGroup(Arc<(ResourceRequest, Loadable<Vec<MetaPreview>, String>)>);
impl Group for CatalogGroup {
    fn new(req: ResourceRequest) -> Self {
        CatalogGroup(Arc::new((req, Loadable::Loading)))
    }
    fn update(&mut self, res: &Result<ResourceResponse, EnvError>) {
        let req = (self.0).0.to_owned();
        self.0 = Arc::new((
            req,
            match res {
                Ok(ResourceResponse::Metas { metas }) => Loadable::Ready(metas.to_owned()),
                Ok(_) => Loadable::Err(UNEXPECTED_RESP_MSG.to_string()),
                Err(e) => Loadable::Err(e.to_string()),
            },
        ));
    }
    fn addon_req(&self) -> &ResourceRequest {
        &(self.0).0
    }
}
