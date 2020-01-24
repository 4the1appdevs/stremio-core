use super::fetch_api;
use crate::state_types::msg::MsgError;
use crate::state_types::Environment;
use crate::types::addons::Descriptor;
use crate::types::api::{APIRequest, AuthKey, SuccessResponse};
use futures::Future;

pub fn set_user_addons<Env: Environment + 'static>(
    auth_key: &AuthKey,
    addons: &[Descriptor],
) -> impl Future<Item = (), Error = MsgError> {
    fetch_api::<Env, _, SuccessResponse>(&APIRequest::AddonCollectionSet {
        auth_key: auth_key.to_owned(),
        addons: addons.to_owned(),
    })
    .map(|_| ())
}
