use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignedEventError {
    #[error("merkle proof error: {0}")]
    MerkleProof(#[from] crate::merkle_proof::MerkleProofError),

    #[error("common error: {0}")]
    Common(#[from] common::CommonError),
}
