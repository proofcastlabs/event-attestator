use crate::{
    types::Result,
    metadata::Metadata,
};

pub trait ToMetadata {
    fn to_metadata(&self) -> Result<Metadata>;
}
