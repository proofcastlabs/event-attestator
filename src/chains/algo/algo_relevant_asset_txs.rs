use derive_more::{Constructor, Deref};
use rust_algorand::{
    AlgorandAddress,
    AlgorandTransaction,
    AlgorandTransactionProof,
    AlgorandTransactionType,
    AlgorandTransactions,
};

use crate::{chains::algo::algo_submission_material::AlgoSubmissionMaterial, types::Result};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deref, Constructor)]
pub struct AlgoRelevantAssetTxs(pub Vec<AlgoRelevantAssetTx>);

impl AlgoRelevantAssetTxs {
    pub fn from_submission_material_for_assets_and_receivers(
        submission_material: &AlgoSubmissionMaterial,
        asset_ids: Vec<u64>,
        receivers: Vec<AlgorandAddress>,
    ) -> Result<Self> {
        Ok(Self::new(
            asset_ids
                .iter()
                .map(|asset_id| {
                    Ok(receivers
                        .iter()
                        .map(|receiver| {
                            AlgoRelevantAssetTx::from_submission_material(submission_material, *asset_id, receiver)
                        })
                        .collect::<Result<Vec<Vec<AlgoRelevantAssetTx>>>>()?
                        .concat())
                })
                .collect::<Result<Vec<Vec<AlgoRelevantAssetTx>>>>()?
                .concat(),
        ))
    }

    pub fn filter_out_invalid_txs(&self, submission_material: &AlgoSubmissionMaterial) -> Self {
        info!("✔ Filtering out invalid transaction proofs...");
        Self::new(
            self.iter()
                .filter(|relevant_tx| {
                    let result = relevant_tx.tx_proof.validate(&submission_material.block);
                    if result.is_ok() {
                        true
                    } else {
                        info!("✘ Proof filtered out because it's invalid: {}", relevant_tx.tx_proof);
                        false
                    }
                })
                .cloned()
                .collect::<Vec<AlgoRelevantAssetTx>>(),
        )
    }

    pub fn to_transactions(&self) -> AlgorandTransactions {
        AlgorandTransactions::new(
            self.iter()
                .map(|relevant_tx| relevant_tx.tx.clone())
                .collect::<Vec<AlgorandTransaction>>(),
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor)]
pub struct AlgoRelevantAssetTx {
    pub tx_index: u64,
    pub tx: AlgorandTransaction,
    pub tx_proof: AlgorandTransactionProof,
}

impl AlgoRelevantAssetTx {
    fn maybe_get_proof_from_submission_material_by_index(
        submission_material: &AlgoSubmissionMaterial,
        index: usize,
    ) -> Option<AlgorandTransactionProof> {
        debug!("✔ Maybe getting proof for tx at index {index}...");
        let filtered_proofs = submission_material
            .proofs
            .iter()
            .filter(|proof| proof.index == index as u64)
            .collect::<Vec<&AlgorandTransactionProof>>();
        if filtered_proofs.is_empty() {
            debug!("✘ No proof for for tx!");
            None
        } else {
            // NOTE: We're ignoring any duplicates, since that suggests malformed submission material.
            // Plus they get validated later anyway so no invalid ones can slip by anyway.
            info!("✔ Proof found for tx!");
            Some(filtered_proofs[0].clone())
        }
    }

    fn from_asset_transfer_txs(
        asset_id: u64,
        receiver: &AlgorandAddress,
        submission_material: &AlgoSubmissionMaterial,
        tx: &AlgorandTransaction,
        tx_index: usize,
    ) -> Option<Self> {
        let is_asset_transfer_txn = tx.txn_type == Some(AlgorandTransactionType::AssetTransfer);
        let maybe_proof = Self::maybe_get_proof_from_submission_material_by_index(submission_material, tx_index);
        let is_desired_asset = tx.transfer_asset_id == Some(asset_id);
        let is_to_redeem_address = tx.asset_receiver == Some(*receiver);
        let amount_is_greater_than_zero = match tx.asset_amount {
            Some(amount) => amount > 0,
            None => false,
        };
        let has_proof = maybe_proof.is_some(); // NOTE: This allows the expect below!
        debug!("Has proof: {}", has_proof);
        debug!("Is desired asset: {}", is_desired_asset);
        debug!("Is to redeem address: {}", is_to_redeem_address);
        debug!("Is asset transfer tx: {}", is_asset_transfer_txn);
        debug!("Amount is greater than zero: {}", amount_is_greater_than_zero);
        if is_asset_transfer_txn && has_proof && is_desired_asset && is_to_redeem_address && amount_is_greater_than_zero
        {
            Some(Self::new(
                tx_index as u64,
                tx.clone(),
                maybe_proof.expect("This to exist!"),
            ))
        } else {
            None
        }
    }

    fn from_submission_material_for_asset_transfer_txs(
        submission_material: &AlgoSubmissionMaterial,
        asset_id: u64,
        receiver: &AlgorandAddress,
    ) -> Result<Vec<Self>> {
        info!("✔ Getting `AlgoRelevantAssetTx` from submission material for asset transfer txs!");
        Ok(submission_material
            .block
            .get_transactions()?
            .iter()
            .enumerate()
            .filter_map(|(i, tx)| Self::from_asset_transfer_txs(asset_id, receiver, submission_material, tx, i))
            .collect::<Vec<Self>>())
    }

    fn from_submission_material_for_application_txs(
        submission_material: &AlgoSubmissionMaterial,
        asset_id: u64,
        receiver: &AlgorandAddress,
    ) -> Result<Vec<Self>> {
        info!("✔ Getting `AlgoRelevantAssetTx` from submission material for asset transfer txs!");
        Ok(submission_material
            .block
            .get_transactions()?
            .iter()
            .enumerate()
            .flat_map(|(i, tx)| {
                if tx.txn_type == Some(AlgorandTransactionType::ApplicationCall) && tx.inner_txs.is_some() {
                    tx.inner_txs
                        .as_ref()
                        .expect("Inner txs definitely exist here!")
                        .iter()
                        .filter_map(|inner_tx| {
                            Self::from_asset_transfer_txs(asset_id, receiver, submission_material, inner_tx, i)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            })
            .collect::<Vec<_>>())
    }

    fn from_submission_material(
        submission_material: &AlgoSubmissionMaterial,
        asset_id: u64,
        receiver: &AlgorandAddress,
    ) -> Result<Vec<Self>> {
        Ok([
            Self::from_submission_material_for_application_txs(submission_material, asset_id, receiver)?,
            Self::from_submission_material_for_asset_transfer_txs(submission_material, asset_id, receiver)?,
        ]
        .concat())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::chains::algo::test_utils::get_sample_submission_material_n;

    #[test]
    fn should_get_algo_relevant_asset_tx_from_app_call_txs() {
        let submission_material = get_sample_submission_material_n(11);
        let receiver = AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        let asset_id = 714666072;
        let result = AlgoRelevantAssetTx::from_submission_material_for_application_txs(
            &submission_material,
            asset_id,
            &receiver,
        )
        .unwrap()
        .len();
        let expected_result = 1;
        assert_eq!(result, expected_result);
    }
}
