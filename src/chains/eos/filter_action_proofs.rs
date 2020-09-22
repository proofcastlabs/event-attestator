use std::str::FromStr;
use eos_primitives::Checksum256;
use eos_primitives::{
    ActionName as EosActionName,
    AccountName as EosAccountName,
};
use crate::{
    types::Result,
    chains::eos::{
        eos_constants::REDEEM_ACTION_NAME,
        eos_merkle_utils::verify_merkle_proof,
        eos_utils::convert_bytes_to_checksum256,
        eos_types::{
            RedeemInfo,
            RedeemInfos,
            ActionProof,
            ActionProofs,
            ProcessedTxIds,
        },
    },
};

pub fn filter_proofs_with_wrong_action_mroot(
    action_mroot: &Checksum256,
    action_proofs: &[ActionProof],
) -> Result<ActionProofs> {
    let filtered = action_proofs
        .iter()
        .filter(|proof_data|
            proof_data.action_proof[proof_data.action_proof.len() - 1] ==
            action_mroot.to_string()
        )
        .cloned()
        .collect::<ActionProofs>();
    debug!("Num proofs before: {}", action_proofs.len());
    debug!("Num proofs after : {}", filtered.len());
    Ok(filtered)
}

pub fn filter_out_proofs_for_other_accounts(
    action_proofs: &[ActionProof],
    required_account_name: EosAccountName,
) -> Result<ActionProofs> {
    let filtered: ActionProofs = action_proofs
        .iter()
        .filter(|proof| proof.action.account == required_account_name)
        .cloned()
        .collect();
    info!("✔ Filtering out proofs for other accounts...");
    debug!("Num proofs before: {}", action_proofs.len());
    debug!(" Num proofs after: {}", filtered.len());
    Ok(filtered)
}

pub fn filter_out_proofs_for_other_actions(
    action_proofs: &[ActionProof]
) -> Result<ActionProofs> {
    let required_action = EosActionName::from_str(REDEEM_ACTION_NAME)?;
    let filtered: ActionProofs = action_proofs
        .iter()
        .filter(|proof| proof.action.name == required_action)
        .cloned()
        .collect();
    info!("✔ Filtering out proofs for other actions...");
    debug!("Num proofs before: {}", action_proofs.len());
    debug!(" Num proofs after: {}", filtered.len());
    Ok(filtered)
}

pub fn filter_out_proofs_with_invalid_merkle_proofs(action_proofs: &[ActionProof]) -> Result<ActionProofs> {
    let filtered = action_proofs
        .iter()
        .map(|proof_data| proof_data.action_proof.as_slice())
        .map(verify_merkle_proof)
        .collect::<Result<Vec<bool>>>()?
        .into_iter()
        .zip(action_proofs.iter())
        .filter_map(|(proof_is_valid, proof)| {if proof_is_valid { Some(proof) } else { None }})
        .cloned()
        .collect::<ActionProofs>();
    debug!("Num proofs before: {}", action_proofs.len());
    debug!("Num proofs after : {}", filtered.len());
    Ok(filtered)
}

pub fn filter_out_invalid_action_receipt_digests(action_proofs: &[ActionProof]) -> Result<ActionProofs> {
    let filtered = action_proofs
        .iter()
        .map(|proof| proof.action_receipt.to_digest())
        .map(hex::encode)
        .zip(action_proofs.iter())
        .filter_map(|(digest, proof)| { if digest == proof.action_proof[0] { Some(proof) } else { None }})
        .cloned()
        .collect::<ActionProofs>();
    debug!("Num proofs before: {}", action_proofs.len());
    debug!("Num proofs after : {}", filtered.len());
    Ok(filtered)
}

pub fn filter_out_proofs_with_action_digests_not_in_action_receipts(
    action_proofs: &[ActionProof]
) -> Result<ActionProofs> {
    let filtered = action_proofs
        .iter()
        .map(|proof| proof.action.to_digest())
        .map(|digest_bytes| convert_bytes_to_checksum256(&digest_bytes))
        .collect::<Result<Vec<Checksum256>>>()?
        .into_iter()
        .zip(action_proofs.iter())
        .filter_map(|(digest, proof)| { if digest == proof.action_receipt.act_digest { Some(proof) } else { None }})
        .cloned()
        .collect::<ActionProofs>();
    debug!("Num proofs before: {}", action_proofs.len());
    debug!("Num proofs after : {}", filtered.len());
    Ok(filtered)
}

pub fn filter_out_already_processed_txs(
    redeem_infos: &RedeemInfos,
    processed_tx_ids: &ProcessedTxIds,
) -> Result<RedeemInfos> {
    Ok(
        RedeemInfos::new(
            &redeem_infos
                .0
                .iter()
                .filter(|params| !processed_tx_ids.contains(&params.global_sequence))
                .cloned()
                .collect::<Vec<RedeemInfo>>()
        )
    )
}

pub fn filter_duplicate_proofs(
    action_proofs: &[ActionProof]
) -> Result<ActionProofs> {
    let mut filtered: ActionProofs = Vec::new();
    action_proofs
        .iter()
        .map(|proof| {
            if !filtered.contains(&proof) {
                filtered.push(proof.clone())
            }
        })
        .for_each(drop);
    debug!("Num proofs before: {}", action_proofs.len());
    debug!("Num proofs after : {}", filtered.len());
    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eos::eos_utils::convert_hex_to_checksum256,
        btc_on_eos::eos::eos_test_utils::get_sample_action_proof_n,
    };


    #[test]
    fn should_not_filter_out_proofs_with_action_digests_in_action_receipts() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        let result = filter_out_proofs_with_action_digests_not_in_action_receipts(&action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_with_action_digests_not_in_action_receipts() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];

        let mut proof_with_invalid_action = get_sample_action_proof_n(3);
        proof_with_invalid_action.action.data[0] = 42;

        let mut dirty_action_proofs = vec![proof_with_invalid_action];
        dirty_action_proofs.extend_from_slice(&action_proofs);

        let result = filter_out_proofs_with_action_digests_not_in_action_receipts(&dirty_action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_not_filter_duplicate_action_proofs_if_there_are_no_duplicates() {
        let expected_num_proofs_after = 2;
        let expected_num_proofs_before = 2;

        let proofs_no_duplicates = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];

        let num_proofs_before = proofs_no_duplicates.len();
        assert_eq!(num_proofs_before, expected_num_proofs_before);

        let result = filter_duplicate_proofs(&proofs_no_duplicates)
            .unwrap();

        assert_eq!(result.len(), num_proofs_before);
        assert_eq!(result.len(), expected_num_proofs_after);

        assert_eq!(result[0], get_sample_action_proof_n(4));
        assert_eq!(result[1], get_sample_action_proof_n(1));
    }

    #[test]
    fn should_filter_duplicate_action_proofs() {
        let expected_num_proofs_after = 2;
        let expected_num_proofs_before = 3;

        let proofs_with_duplicate = vec![
            get_sample_action_proof_n(1),
            get_sample_action_proof_n(2),
            get_sample_action_proof_n(2),
        ];

        let num_proofs_before = proofs_with_duplicate.len();
        assert_eq!(num_proofs_before, expected_num_proofs_before);

        let result = filter_duplicate_proofs(&proofs_with_duplicate)
            .unwrap();

        assert!(result.len() < num_proofs_before);
        assert_eq!(result.len(), expected_num_proofs_after);

        assert_eq!(result[0], get_sample_action_proof_n(1));
        assert_eq!(result[1], get_sample_action_proof_n(2));
    }


    #[test]
    fn should_not_filter_out_valid_action_receipt_digests() {
        let action_proofs = vec![
            get_sample_action_proof_n(1),
            get_sample_action_proof_n(2),
        ];
        let result = filter_out_invalid_action_receipt_digests(&action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_out_invalid_action_receipt_digests() {
        let action_proofs = vec![
            get_sample_action_proof_n(1),
            get_sample_action_proof_n(2),
        ];

        let mut proof_with_invalid_receipt = get_sample_action_proof_n(3);
        proof_with_invalid_receipt.action_receipt.global_sequence = 42;

        let mut dirty_action_proofs = vec![proof_with_invalid_receipt];
        dirty_action_proofs.extend_from_slice(&action_proofs);

        let result = filter_out_invalid_action_receipt_digests(&dirty_action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_not_filter_out_proofs_with_valid_merkle_proofs() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        let result = filter_out_proofs_with_invalid_merkle_proofs(&action_proofs).unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_with_invalid_merkle_proofs() {
        let mut dirty_action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];

        dirty_action_proofs[0].action_proof.pop();

        let result = filter_out_proofs_with_invalid_merkle_proofs(&dirty_action_proofs)
            .unwrap();

        assert_eq!(result, [get_sample_action_proof_n(1)]);
    }

    #[test]
    fn should_not_filter_out_proofs_for_required_account() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        let account = EosAccountName::from_str("pbtctokenxxx").unwrap();

        assert_eq!(filter_out_proofs_for_other_accounts(&action_proofs, account).unwrap(), action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_for_other_accounts() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        let account = EosAccountName::from_str("provtestable").unwrap();

        assert_eq!(filter_out_proofs_for_other_accounts(&action_proofs, account).unwrap(), []);
    }

    #[test]
    fn should_not_filter_out_proofs_for_valid_actions() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        let result = filter_out_proofs_for_other_actions(&action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_for_other_actions() {
        let valid_action_name = EosActionName::from_str(REDEEM_ACTION_NAME).unwrap();
        let invalid_action_name = EosActionName::from_str("setproducers").unwrap();

        let mut dirty_action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        dirty_action_proofs[0].action.name = invalid_action_name;

        assert_ne!(dirty_action_proofs[0].action.name, valid_action_name);

        let result = filter_out_proofs_for_other_actions(&dirty_action_proofs)
            .unwrap();

        assert_eq!(result, [get_sample_action_proof_n(1)]);
    }

    #[test]
    fn should_not_filter_proofs_with_correct_action_mroot() {
        let action_proofs = vec![
            get_sample_action_proof_n(1),
            get_sample_action_proof_n(1),
            get_sample_action_proof_n(1),
        ];
        let action_mroot = convert_hex_to_checksum256("6ba2320b7d71d69770735f92b22f0d986d7e5d72f8842fa93b5604c63dd515c7").unwrap();
        let result = filter_proofs_with_wrong_action_mroot(&action_mroot, &action_proofs).unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_proofs_with_wrong_action_mroot() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(1),
        ];
        let action_mroot = convert_hex_to_checksum256("10c0518e15ae178bdd622e3f31249f0f12071c68045dd565a267a522df8ba96c").unwrap();
        let result = filter_proofs_with_wrong_action_mroot(&action_mroot, &action_proofs).unwrap();

        assert_eq!(result, [get_sample_action_proof_n(4)]);
    }
}
