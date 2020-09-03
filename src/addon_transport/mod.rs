use crate::state_types::{EnvFuture, Environment};
use crate::types::addon::*;
use core::pin::Pin;
use futures::future;
use http::Request;
use std::marker::PhantomData;
use url::Url;

mod legacy;
use self::legacy::AddonLegacyTransport;

pub const MANIFEST_PATH: &str = "/manifest.json";
pub const LEGACY_PATH: &str = "/stremio/v1";

pub trait AddonInterface {
    fn get(&self, path: &ResourceRef) -> EnvFuture<ResourceResponse>;
    fn manifest(&self) -> EnvFuture<Manifest>;
}

pub struct AddonHTTPTransport<T: Environment> {
    env: PhantomData<T>,
    transport_url: Url,
}
impl<T: Environment> AddonHTTPTransport<T> {
    pub fn from_url(url: &Url) -> Self {
        AddonHTTPTransport {
            env: PhantomData,
            transport_url: url.to_owned(),
        }
    }
}
impl<T: Environment> AddonInterface for AddonHTTPTransport<T> {
    fn get(&self, path: &ResourceRef) -> EnvFuture<ResourceResponse> {
        if self.transport_url.path().ends_with(LEGACY_PATH) {
            return AddonLegacyTransport::<T>::from_url(&self.transport_url).get(&path);
        }

        if !self.transport_url.path().ends_with(MANIFEST_PATH) {
            return Pin::new(Box::new(future::err(
                format!("transport_url must end in {}", MANIFEST_PATH).into(),
            )));
        }

        let url = self
            .transport_url
            .as_str()
            .replace(MANIFEST_PATH, &path.to_string());
        let r = Request::get(&url).body(()).expect("builder cannot fail");
        T::fetch::<_, ResourceResponse>(r)
    }
    fn manifest(&self) -> EnvFuture<Manifest> {
        if self.transport_url.path().ends_with(LEGACY_PATH) {
            return AddonLegacyTransport::<T>::from_url(&self.transport_url).manifest();
        }

        let r = Request::get(self.transport_url.as_str())
            .body(())
            .expect("builder cannot fail");
        T::fetch::<_, Manifest>(r)
    }
}
