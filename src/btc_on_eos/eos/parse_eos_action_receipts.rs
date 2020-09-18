use std::str::FromStr;
use eos_primitives::{
    AccountName,
    AuthSequence,
    AuthSequences,
    ActionReceipt as EosActionReceipt,
};
use crate::{
    types::Result,
    chains::eos::eos_utils::convert_hex_to_checksum256,
    btc_on_eos::eos::eos_types::{
        AuthSequenceJson,
        EosActionReceiptJson,
    },
};

fn parse_auth_sequence_json(
    auth_sequence_json: &AuthSequenceJson
) -> Result<AuthSequence> {
    Ok(
        AuthSequence::new(
            &auth_sequence_json.0,
            auth_sequence_json.1
        )?
    )
}

fn parse_auth_sequence_jsons(
    auth_sequence_jsons: &[AuthSequenceJson]
) -> Result<AuthSequences> {
    auth_sequence_jsons
        .iter()
        .map(parse_auth_sequence_json)
        .collect::<Result<AuthSequences>>()
}

pub fn parse_eos_action_receipt_json(
    eos_action_receipt_json: &EosActionReceiptJson
) -> Result<EosActionReceipt> {
    Ok(
        EosActionReceipt {
            abi_sequence: eos_action_receipt_json.abi_sequence,
            code_sequence: eos_action_receipt_json.code_sequence,
            recipient: AccountName::from_str(
                &eos_action_receipt_json
                    .receiver
            )?,
            act_digest: convert_hex_to_checksum256(
                &eos_action_receipt_json.act_digest
            )?,
            global_sequence: eos_action_receipt_json
                .global_sequence,
            recv_sequence: eos_action_receipt_json
                .recv_sequence,
            auth_sequence: parse_auth_sequence_jsons(
                &eos_action_receipt_json.auth_sequence
            )?,
        }
    )
}
