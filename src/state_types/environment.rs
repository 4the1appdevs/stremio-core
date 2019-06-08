use futures::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use crate::types::addons::TransportUrl;
use crate::addon_transport::{AddonHTTPTransport, AddonInterface};

pub use http::Request;

pub type EnvFuture<T> = Box<Future<Item = T, Error = Box<dyn Error>>>;
pub trait Environment {
    // https://serde.rs/lifetimes.html#trait-bounds
    fn fetch_serde<IN, OUT>(request: Request<IN>) -> EnvFuture<OUT>
    where
        IN: 'static + Serialize,
        OUT: 'static + DeserializeOwned;
    fn exec(fut: Box<Future<Item = (), Error = ()>>);
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<Option<T>>;
    fn set_storage<T: 'static + Serialize>(key: &str, value: Option<&T>) -> EnvFuture<()>;
    fn addon_transport(url: &TransportUrl) -> Box<dyn AddonInterface>
        where Self: Sized + 'static
    {
        Box::new(AddonHTTPTransport::<Self>::from_url(url))
    }
}
