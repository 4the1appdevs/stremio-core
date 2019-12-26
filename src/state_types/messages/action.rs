use crate::state_types::models::{Settings, StreamingServerSettings};
use crate::types::addons::{Descriptor, ExtraProp, ResourceRef, ResourceRequest, TransportUrl};
use crate::types::api::GDPRConsent;
use crate::types::{LibItem, MetaPreview};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "load", content = "args")]
pub enum ActionLoad {
    CatalogsWithExtra {
        extra: Vec<ExtraProp>,
    },
    CatalogFiltered(ResourceRequest),
    LibraryFiltered {
        type_name: String,
        sort_prop: Option<String>,
    },
    MetaDetails {
        type_name: String,
        id: String,
        video_id: Option<String>,
    },
    AddonDetails {
        transport_url: String,
    },
    Notifications,
    Player {
        transport_url: String,
        type_name: String,
        id: String,
        video_id: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "settings", content = "args")]
pub enum ActionSettings {
    // Although we load the streaming server settings together with the context
    // there is also a way to reload it separately in cases this is necessary.
    LoadStreamingServer,
    StoreStreamingServer(Box<StreamingServerSettings>),
    Store(Box<Settings>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "addonOp", content = "args")]
pub enum ActionAddon {
    Uninstall { transport_url: TransportUrl },
    Install(Box<Descriptor>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "userOp", content = "args")]
pub enum ActionUser {
    Login {
        email: String,
        password: String,
    },
    Register {
        email: String,
        password: String,
        gdpr_consent: GDPRConsent,
    },
    Logout,
    PullAndUpdateAddons,
    PushAddons,
    LibSync,
    LibUpdate(LibItem),
    AddToLibrary {
        meta_item: MetaPreview,
        now: DateTime<Utc>,
    },
    RemoveFromLibrary {
        id: String,
        now: DateTime<Utc>,
    },
    // @TODO consider PullUser, PushUser?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "playerOp", content = "args")]
pub enum ActionPlayer {
    TimeChanged {
        time: u64,
        duration: u64
    },
    Ended
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum Action {
    LoadCtx,
    Load(ActionLoad),
    Settings(ActionSettings),
    AddonOp(ActionAddon),
    UserOp(ActionUser),
    PlayerOp(ActionPlayer),
    Unload,
}
