use std::str::FromStr;

use rust_algorand::{AlgorandBlock, AlgorandBlockJson, AlgorandTransactionProof, AlgorandTransactionProofJson};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    chains::algo::algo_state::AlgoState,
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoSubmissionMaterial {
    block: AlgorandBlock,
    proofs: Vec<AlgorandTransactionProof>,
}

impl AlgoSubmissionMaterial {
    pub fn from_json(json: &AlgoSubmissionMaterialJson) -> Result<Self> {
        Ok(Self {
            block: AlgorandBlock::from_json(&json.block)?,
            proofs: json
                .proofs
                .iter()
                .map(|proof_json| Ok(AlgorandTransactionProof::from_json(proof_json)?))
                .collect::<Result<Vec<AlgorandTransactionProof>>>()?,
        })
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice::<AlgoSubmissionMaterialJson>(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }

    pub fn to_json(&self) -> Result<AlgoSubmissionMaterialJson> {
        Ok(AlgoSubmissionMaterialJson {
            block: self.block.to_json()?,
            proofs: self
                .proofs
                .iter()
                .map(|proof| proof.to_json())
                .collect::<Vec<AlgorandTransactionProofJson>>(),
        })
    }
}

impl FromStr for AlgoSubmissionMaterial {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        AlgoSubmissionMaterialJson::from_str(s).and_then(|ref json| Self::from_json(json))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoSubmissionMaterialJson {
    block: AlgorandBlockJson,
    proofs: Vec<AlgorandTransactionProofJson>,
}

impl FromStr for AlgoSubmissionMaterialJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

// FIXME so this needs to change and take into account the proofs now!
pub fn parse_algo_submission_material_and_put_in_state<'a, D: DatabaseInterface>(
    submission_material: &str,
    state: AlgoState<'a, D>,
) -> Result<AlgoState<'a, D>> {
    state.add_submitted_algo_block(&AlgorandBlock::from_str(submission_material)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::chains::algo::test_utils::get_sample_block_json_str_n;

    #[test]
    fn should_parse_submission_material_from_str() {
        let submission_material_str = get_sample_block_json_str_n(0);
        let result = AlgoSubmissionMaterial::from_str(&submission_material_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_serde_submission_material_to_and_from_json() {
        let submission_material_str = get_sample_block_json_str_n(0);
        let submission_material = AlgoSubmissionMaterial::from_str(&submission_material_str).unwrap();
        let json = submission_material.to_json().unwrap();
        let result = AlgoSubmissionMaterial::from_json(&json).unwrap();
        assert_eq!(result, submission_material)
    }

    #[test]
    fn should_serde_algo_submission_material_to_and_from_bytes() {
        let submission_material_str = get_sample_block_json_str_n(0);
        let submission_material = AlgoSubmissionMaterial::from_str(&submission_material_str).unwrap();
        let bytes = submission_material.to_bytes().unwrap();
        let result = AlgoSubmissionMaterial::from_bytes(&bytes).unwrap();
        //assert_eq!(result, submission_material);
    }
}
