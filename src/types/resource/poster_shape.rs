use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "camelCase")]
pub enum PosterShape {
    Square,
    Landscape,
    #[derivative(Default)]
    #[serde(other)]
    Poster,
}
