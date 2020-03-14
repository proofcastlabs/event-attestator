use eos_primitives::{
    SerializeData,
    ActionTransfer,
    PermissionLevel,
    Action as EosAction,
    Transaction as EosTransaction,
};
use crate::btc_on_eos::{
    types::Result,
    eos::{
        eos_types::EosSignedTransaction,
        eos_crypto::eos_private_key::EosPrivateKey,
        eos_constants::{
            EOS_TOKEN_NAME,
            EOS_TRANSFER_ACTION,
        },
    },
};

fn get_peos_permission_level(
    actor: &str,
    permission_level: &str,
) -> Result<PermissionLevel> {
    Ok(PermissionLevel::from_str(actor, permission_level)?)
}

fn get_peos_transfer_action(
    to: &str,
    from: &str,
    memo: &str,
    amount: &str,
) -> Result<ActionTransfer> {
    Ok(ActionTransfer::from_str(from, to, amount, memo)?)
}

fn get_eos_transfer_action(
    to: &str,
    from: &str,
    memo: &str,
    actor: &str,
    amount: &str,
    permission_level: &str,
) -> Result<EosAction> {
    Ok(
        EosAction::from_str(
            EOS_TOKEN_NAME,
            EOS_TRANSFER_ACTION,
            vec![get_peos_permission_level(actor, permission_level)?],
            get_peos_transfer_action(to, from, memo, amount)?,
        )?
    )
}

pub fn get_unsigned_peos_transaction(
    to: &str,
    from: &str,
    memo: &str,
    actor: &str,
    amount: &str,
    ref_block_num: u16,
    ref_block_prefix: u32,
    seconds_from_now: u32,
    permission_level: &str,
) -> Result<EosTransaction> {
    Ok(
        EosTransaction::new(
            seconds_from_now,
            ref_block_num,
            ref_block_prefix,
            vec![
                get_eos_transfer_action(
                    to,
                    from,
                    memo,
                    actor,
                    amount,
                    permission_level,
                )?
            ]
        )
    )
}

pub fn sign_peos_transaction(
    to: &str,
    amount: &str,
    chain_id: &str,
    eos_private_key: &EosPrivateKey,
    unsigned_transaction: &EosTransaction,
) -> Result<EosSignedTransaction> {
    Ok(
        EosSignedTransaction::new(
            format!(
                "{}",
                eos_private_key
                    .sign_message_bytes(
                        &unsigned_transaction.get_signing_data(chain_id)?
                    )?
            ),
            hex::encode(
                &unsigned_transaction.to_serialize_data()[..]
            ).to_string(),
            to.to_string(),
            amount.to_string(),
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::eos::{
        eos_test_utils::{
            EOS_JUNGLE_CHAIN_ID,
            get_sample_eos_private_key_2,
        },
        eos_constants::{
            MEMO,
            PEOS_ACCOUNT_NAME,
            PEOS_ACCOUNT_ACTOR,
            EOS_MAX_EXPIRATION_SECS,
            PEOS_ACCOUNT_PERMISSION_LEVEL,
        },
    };

    #[test]
    fn should_sign_a_tx_correctly_1() {
        let to = "provabletest";
        let amount = "1.0000 EOS";
        let ref_block_num = 1195;
        let ref_block_prefix = 4039442863;
        let unsigned_transaction = get_unsigned_peos_transaction(
            to,
            PEOS_ACCOUNT_NAME,
            MEMO,
            PEOS_ACCOUNT_ACTOR,
            amount,
            ref_block_num,
            ref_block_prefix,
            EOS_MAX_EXPIRATION_SECS,
            PEOS_ACCOUNT_PERMISSION_LEVEL,
        ).unwrap();
        let result = sign_peos_transaction(
            to,
            amount,
            EOS_JUNGLE_CHAIN_ID,
            &get_sample_eos_private_key_2(),
            &unsigned_transaction,
        )
            .unwrap()
            .transaction;
        let expected_result = "ab04af01c5f0000000000100a6823403ea3055000000572d3ccdcd013021cd2a1eb3e9ad00000000a8ed3232363021cd2a1eb3e9ad90b1ca2a1eb3e9ad102700000000000004454f53000000001570454f53202d3e20454f5320636f6d706c6574652100"
            .to_string();
        // NOTE: First 4 bytes are the timestamp (8 hex chars...)
        // NOTE: Signature not deterministic ∴ we don't test it.
        let result_without_timestamp = &result[8..];
        assert!(result_without_timestamp == expected_result);
    }

    #[test]
    fn should_sign_a_tx_correctly_2() {
        let to = "provabletest";
        let amount = "1.0000 EOS";
        let ref_block_num = 18188;
        let ref_block_prefix = 594982047;
        let unsigned_transaction = get_unsigned_peos_transaction(
            to,
            PEOS_ACCOUNT_NAME,
            MEMO,
            PEOS_ACCOUNT_ACTOR,
            amount,
            ref_block_num,
            ref_block_prefix,
            EOS_MAX_EXPIRATION_SECS,
            PEOS_ACCOUNT_PERMISSION_LEVEL,
        ).unwrap();
        let result = sign_peos_transaction(
            to,
            amount,
            EOS_JUNGLE_CHAIN_ID,
            &get_sample_eos_private_key_2(),
            &unsigned_transaction,
        )
            .unwrap()
            .transaction;
        // NOTE: Broadcast tx: https://jungle.bloks.io/transaction/621cdccee73d769cb201dbd6f52352e3df20e1b4797e993c9c65f28dd935648f
        let expected_result = "0c479fb47623000000000100a6823403ea3055000000572d3ccdcd013021cd2a1eb3e9ad00000000a8ed3232363021cd2a1eb3e9ad90b1ca2a1eb3e9ad102700000000000004454f53000000001570454f53202d3e20454f5320636f6d706c6574652100"
            .to_string();
        // NOTE: First 4 bytes are the timestamp (8 hex chars...)
        // NOTE: Signature not deterministic ∴ we don't test it.
        let result_without_timestamp = &result[8..];
        assert!(result_without_timestamp == expected_result); // FIXME!
    }
}
