use derivative::Derivative;
use serde::Serialize;

#[derive(Derivative, Clone, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(tag = "type", content = "content")]
pub enum Loadable<R, E> {
    #[derivative(Default)]
    Loading,
    Ready(R),
    Err(E),
}
