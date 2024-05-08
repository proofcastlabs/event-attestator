use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignedEventVersion {
    V1,
}

impl Default for SignedEventVersion {
    fn default() -> Self {
        Self::V1
    }
}
