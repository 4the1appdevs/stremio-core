use crate::constants::API_URL;
use crate::models::ctx::CtxError;
use crate::runtime::Env;
use crate::types::api::{APIMethodName, APIResult};
use futures::{future, Future, TryFutureExt};
use http::Request;
use serde::{Deserialize, Serialize};

pub fn fetch_api<E, REQ, RESP>(api_request: &REQ) -> impl Future<Output = Result<RESP, CtxError>>
where
    E: Env,
    REQ: APIMethodName + Clone + Serialize,
    for<'de> RESP: Deserialize<'de> + 'static,
{
    let url = API_URL
        .join("api/")
        .expect("url builder failed")
        .join(api_request.method_name())
        .expect("url builder failed");
    let request = Request::post(url.as_str())
        .body(api_request.to_owned())
        .expect("request builder failed");
    E::fetch::<_, _>(request)
        .map_err(CtxError::from)
        .and_then(|result| match result {
            APIResult::Ok { result } => future::ok(result),
            APIResult::Err { error } => future::err(CtxError::from(error)),
        })
}
