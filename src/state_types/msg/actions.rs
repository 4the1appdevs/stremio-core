use crate::types::addons::*;
use crate::types::LibItem;
use serde_derive::*;
use std::collections::HashMap;

//
// Input actions: those are triggered by users
//
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    CatalogGrouped { extra: Vec<ExtraProp> },
    CatalogFiltered(ResourceRequest),
    Detail { type_name: String, id: String },
    Streams { type_name: String, id: String },
    // @TODO most of these values need content
    AddonCatalog,
    Notifications,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Remove { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "settings", content = "args")]
pub enum ActionSettings {
    Store(Box<HashMap<String, String>>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "userOp", content = "args")]
pub enum ActionUser {
    Login { email: String, password: String },
    Register { email: String, password: String },
    Logout,
    PullAndUpdateAddons,
    PushAddons,
    LibSync,
    LibUpdate(LibItem),
    // @TODO consider PullUser, PushUser?
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    LoadCtx,
    Load(ActionLoad),
    Settings(ActionSettings),
    AddonOp(ActionAddon),
    UserOp(ActionUser),
}
