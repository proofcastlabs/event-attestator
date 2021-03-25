use crate::{
    metadata::Metadata,
    types::{Bytes, Result},
};

pub trait ToMetadata {
    fn to_metadata(&self) -> Result<Metadata>;
    fn to_metadata_bytes(&self) -> Result<Bytes>;
}
