use crate::{metadata::Metadata, types::Result};

pub trait ToMetadata {
    fn to_metadata(&self) -> Result<Metadata>;
}
