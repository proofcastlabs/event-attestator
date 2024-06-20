use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignedEventError {
    #[error("merkle proof error: {0}")]
    MerkleProof(#[from] crate::MerkleError),

    #[error("common error: {0}")]
    Common(#[from] common::CommonError),

    #[error("network id error: {0}")]
    NetworkId(#[from] common_network_ids::NetworkIdError),

    #[error("metadata chain id error: {0}")]
    MetadataChainId(#[from] common_metadata::MetadataChainIdError),

    #[error("event id error: {0}")]
    EventIdError(#[from] EventIdError),
}

#[derive(Debug, Error)]
pub enum EventIdError {
    #[error("common error: {0}")]
    Common(#[from] common::CommonError),

    #[error("encoded event is none")]
    EncodedEventIsNone,

    #[error("from hex error: {0}")]
    FromHexError(#[from] hex::FromHexError),
}
