use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Getters)]
pub struct MongoConfig {
    pub enabled: bool,
    pub uri_str: String,
    pub database: String,
    pub collection: String,
}
