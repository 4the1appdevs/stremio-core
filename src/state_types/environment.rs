use futures::Future;
use serde::de::DeserializeOwned;
use std::error::Error;
pub type EnvFuture<T> = Box<Future<Item = Box<T>, Error = Box<Error>>>;
pub trait Environment {
    // https://serde.rs/lifetimes.html#trait-bounds
    fn fetch_serde<T: 'static + DeserializeOwned>(url: &str) -> EnvFuture<T>;
    fn exec(fut: Box<Future<Item = (), Error = ()>>);
    fn get_storage<T: 'static + DeserializeOwned>(key: &str) -> EnvFuture<T>;
    fn set_storage<T: 'static + DeserializeOwned>(key: &str, value: &T) -> EnvFuture<()>;
}
