use crate::state_types::models::ctx::{CtxAuthResponse, CtxError, CtxStorageResponse};
use crate::state_types::models::streaming_server::Settings as StreamingServerSettings;
use crate::state_types::EnvError;
use crate::types::addons::{Descriptor, Manifest, ResourceRequest, ResourceResponse, TransportUrl};
use crate::types::api::{AuthKey, AuthRequest};
use crate::types::LibItem;
use url::Url;

//
// Those messages are meant to be dispatched and hanled only inside stremio-core crate
//
#[derive(Debug)]
pub enum Internal {
    // Result for pull profile and library from storage.
    CtxStorageResult(Box<Result<CtxStorageResponse, CtxError>>),
    // Result for authenticate to API.
    CtxAuthResult(AuthRequest, Result<CtxAuthResponse, CtxError>),
    // Result for pull addons from API.
    AddonsAPIResult(AuthKey, Result<Vec<Descriptor>, CtxError>),
    // Result for sync library items with API. Returns newer items that needs to be updated.
    LibrarySyncResult(AuthKey, Result<Vec<LibItem>, CtxError>),
    // Dispatched when library item needs to be updated in the memory, storage and API.
    UpdateLibraryItem(LibItem),
    // Dispatched when some of auth, addons or settings changed.
    ProfileChanged(bool),
    // Dispatched when library changes with a flag if its already persisted.
    LibraryChanged(bool),
    // Result for loading streaming server
    StreamingServerSettingsResult(Url, Result<StreamingServerSettings, EnvError>),
    // Result for updating streaming server
    StreamingServerUpdateSettingsResult(Url, Result<(), EnvError>),
    ResourceRequestResult(ResourceRequest, Box<Result<ResourceResponse, EnvError>>),
    ManifestRequestResult(TransportUrl, Result<Manifest, EnvError>),
}
