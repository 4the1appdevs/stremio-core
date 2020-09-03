use crate::state_types::models::ctx::CtxError;
use crate::state_types::Environment;
use crate::types::api::{APIMethodName, APIResult};
use futures::{future, Future, TryFutureExt};
use http::Request;
use serde::{Deserialize, Serialize};

pub fn fetch_api<Env, REQ, RESP>(api_request: &REQ) -> impl Future<Output = Result<RESP, CtxError>>
where
    Env: Environment + 'static,
    REQ: APIMethodName + Clone + Serialize + 'static,
    for<'de> RESP: Deserialize<'de> + 'static,
{
    let url = Env::api_url()
        .join("api/")
        .expect("api url builder cannot fail")
        .join(api_request.method_name())
        .expect("api url builder cannot fail");
    let request = Request::post(url.as_str())
        .body(api_request.to_owned())
        .expect("fetch_api request builder cannot fail");
    Env::fetch::<_, _>(request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => future::ready(Ok(result)),
            APIResult::Err { error } => future::ready(Err(CtxError::from(error))),
        })
}
