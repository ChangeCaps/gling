use serde::{Deserialize, Serialize};

pub mod v1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Data {
    V1(v1::Data),
}

impl Default for Data {
    fn default() -> Self {
        Self::V1(Default::default())
    }
}
