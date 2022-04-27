use derive_more::{Constructor, Deref};
use rust_algorand::{AlgorandAddress, AlgorandTransaction, AlgorandTransactionProof, AlgorandTransactions};

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
                            AlgoRelevantAssetTx::from_submission_material_for_asset_and_receiver(
                                submission_material,
                                *asset_id,
                                receiver,
                            )
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
        info!("✔ Maybe getting proof for tx at index {index}...");
        let filtered_proofs = submission_material
            .proofs
            .iter()
            .filter(|proof| proof.index == index as u64)
            .collect::<Vec<&AlgorandTransactionProof>>();
        if filtered_proofs.is_empty() {
            info!("✘ No proof for for tx!");
            None
        } else {
            // NOTE: We're ignoring any duplicates, since that suggests malformed submission material.
            // Plus they get validated later anyway so no invalid ones can slip by anyway.
            info!("✔ Proof found for tx!");
            Some(filtered_proofs[0].clone())
        }
    }

    fn from_submission_material_for_asset_and_receiver(
        submission_material: &AlgoSubmissionMaterial,
        asset_id: u64,
        receiver: &AlgorandAddress, // NOTE: Get from here: db_utils.get_redeem_address()?.to_string(),
    ) -> Result<Vec<Self>> {
        info!(
            "✔ Getting `AlgoRelevantAssetTx` from submission material for asset ID '{}' and receiver '{}'",
            asset_id, receiver
        );
        Ok(submission_material
            .block
            .get_transactions()?
            .iter()
            .enumerate()
            .filter_map(|(i, tx)| {
                let maybe_proof = Self::maybe_get_proof_from_submission_material_by_index(submission_material, i);
                let is_desired_asset = tx.transfer_asset_id == Some(asset_id);
                let is_to_redeem_address = tx.asset_receiver == Some(*receiver);
                let amount_is_gt_zero = match tx.asset_amount {
                    Some(amount) => amount > 0,
                    None => false,
                };
                let has_proof = maybe_proof.is_some(); // NOTE: This allows the `expect` below!
                if is_desired_asset && is_to_redeem_address && amount_is_gt_zero && has_proof {
                    Some(AlgoRelevantAssetTx::new(
                        i as u64,
                        tx.clone(),
                        maybe_proof.expect("We know this proof exists at this point!"),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<Self>>())
    }
}
