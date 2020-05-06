#![cfg(test)]
#![allow(unused_imports)]
use bitcoin_hashes::{
    sha256,
    Hash as HashTrait
};
use secp256k1::{
    key::SecretKey,
    Message as Secp256k1Message,
};
use std::{
    path::Path,
    str::FromStr,
    fs::read_to_string,
};
use eos_primitives::{
    ActionName,
    Checksum256,
    AccountName,
    AuthSequence,
    ActionTransfer,
    PermissionLevel,
    Action as EosAction,
    BlockHeader as EosBlockHeader,
    ActionReceipt as EosActionReceipt,
    ProducerScheduleV2 as EosProducerScheduleV2,
};
use crate::{
    errors::AppError,
    types::{
        Bytes,
        Result,
    },
    btc_on_eos::{
        test_utils::get_sample_message_to_sign,
        utils::{
            convert_hex_to_checksum256,
            convert_bytes_to_checksum256,
        },
        eos::{
            eos_merkle_utils::Incremerkle,
            eos_state::EosState,
            initialize_eos::eos_init_utils::EosInitJson,
            parse_eos_schedule::{
                EosProducerScheduleJson,
                parse_schedule_string_to_json,
                parse_schedule_string_to_schedule,
                convert_schedule_json_to_schedule_v2,
            },
            parse_submission_material::{
                parse_eos_block_header_from_json,
                parse_eos_submission_material_string_to_json,
                parse_eos_submission_material_string_to_struct,
            },
            eos_types::{
                ActionProof,
                ActionProofs,
                Checksum256s,
                EosBlockHeaderJson,
                EosSignedTransaction,
                EosSignedTransactions,
                EosSubmissionMaterial,
                EosSubmissionMaterialJson,
            },
            eos_crypto::{
                eos_signature::EosSignature,
                eos_public_key::EosPublicKey,
                eos_private_key::EosPrivateKey,
            },
        },
    },
};

pub const NUM_INIT_SAMPLES: usize = 3;

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_1: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-81784220.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_2: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-80440580.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_3: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-84187467.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_4: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-81772484.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_5: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-10800.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_6: &str =
    "src/btc_on_eos/eos/eos_test_utils/jungle-3-block-8242000.json";

pub const SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_7: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-block-10700626.json";

pub const SAMPLE_INIT_BLOCK_JSON_PATH_1: &str =
    "src/btc_on_eos/eos/eos_test_utils/jungle-3-init-block-10857380.json";

pub const SAMPLE_INIT_BLOCK_JSON_PATH_2: &str =
    "src/btc_on_eos/eos/eos_test_utils/jungle-3-init-block-11879805.json";

pub const SAMPLE_INIT_BLOCK_JSON_PATH_3: &str =
    "src/btc_on_eos/eos/eos_test_utils/jungle-3-init-block-11379805.json";

pub const SAMPLE_EOS_ACTIVE_SCHEDULE_PATH_PREFIX: &str =
    "src/btc_on_eos/eos/eos_test_utils/sample-active-schedule-";

pub const SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_JUNGLE_3_JSON_1: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-init-and-subsequent-blocks-jungle-3-1.json";

pub const SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_MAINNET_JSON_1: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-init-and-subsequent-blocks-mainnet-1.json";

pub const SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_MAINNET_JSON_2: &str =
    "src/btc_on_eos/eos/eos_test_utils/eos-init-and-subsequent-blocks-mainnet-2.json";

pub const EOS_JUNGLE_CHAIN_ID: &str =
    "e70aaab8997e1dfce58fbfac80cbbb8fecec7b99cf982a9444273cbc64c41473";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosInitAndSubsequentJson {
    pub init_block: EosInitJson,
    pub subsequent_blocks: Vec<EosSubmissionMaterialJson>,
}

impl EosInitAndSubsequentJson {
    pub fn from_json_string(json_string: &String) -> Result<Self> {
        match serde_json::from_str(&json_string) {
            Ok(result) => Ok(result),
            Err(e) => Err(AppError::Custom(e.to_string()))
        }
    }

    pub fn total_num_blocks(&self) -> usize {
        self.subsequent_blocks.len() + 1
    }

    fn check_n(&self, n: usize) -> Result<()> {
        match n >= 1 && n <= self.total_num_blocks() {
            true => Ok(()),
            false => Err(AppError::Custom(
                format!("✘ Not enough blocks to get block num {}!", n)
            )),
        }
    }

    fn get_incremerkle_for_initial_block(&self) -> Result<Incremerkle> {
        Ok(
            Incremerkle::new(
                self.init_block.block.block_num - 1,
                self.init_block
                    .blockroot_merkle
                    .iter()
                    .map(convert_hex_to_checksum256)
                    .collect::<Result<Checksum256s>>()?,
            )
        )
    }

    pub fn get_active_schedule(&self) -> Result<EosProducerScheduleV2> {
        convert_schedule_json_to_schedule_v2(&self.init_block.active_schedule)
    }

    pub fn get_block_json_n(&self, n: usize) -> Result<EosBlockHeaderJson> {
        self.check_n(n)
            .and_then(|_|
                match n == 1 {
                    true => Ok(self.init_block.block.clone()),
                    false => Ok(
                        self.subsequent_blocks[n - 2].block_header.clone()
                    ),
                }
            )
    }

    pub fn get_block_n(&self, n: usize) -> Result<EosBlockHeader> {
        parse_eos_block_header_from_json(&self.get_block_json_n(n)?)
    }

    pub fn get_producer_signature_for_block_n(
        &self,
        n: usize
    ) -> Result<String> {
        self.check_n(n)
            .and_then(|_| self.get_block_json_n(n))
            .map(|block_json| block_json.producer_signature)
    }

    pub fn get_interim_ids_for_block_n(
        &self,
        n: usize
    ) -> Result<Checksum256s> {
        match n < 1 && n <= self.total_num_blocks() {
            false => Err(AppError::Custom(
                format!("✘ Error getting interim IDs for block {}", n)
            )),
            true => self
                .subsequent_blocks[n]
                .interim_block_ids
                .iter()
                .map(convert_hex_to_checksum256)
                .collect::<Result<Checksum256s>>()
        }
    }

    pub fn get_incremerkle_for_block_n(&self, n: usize) -> Result<Incremerkle> {
        self.check_n(n)
            .and_then(|_| {
                let mut incremerkle = self.get_incremerkle_for_initial_block()?;
                match n == 1 {
                    true => Ok(incremerkle),
                    _ => {
                        vec![0; n - 1]
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                let mut block_ids = vec![];
                                self
                                    .subsequent_blocks[i]
                                    .interim_block_ids
                                    .iter()
                                    .map(|id| block_ids.push(id.clone()))
                                    .for_each(drop);
                                block_ids
                            })
                            .flatten()
                            .map(convert_hex_to_checksum256)
                            .collect::<Result<Checksum256s>>()?
                            .iter()
                            .map(|checksum| incremerkle.append(*checksum))
                            .for_each(drop);
                        Ok(incremerkle)
                    }
                }
            })
    }

    pub fn get_block_mroot_for_block_n(&self, n: usize) -> Result<Bytes> {
        self.get_incremerkle_for_block_n(n)
            .map(|incremerkle| incremerkle.get_root().to_bytes().to_vec())
    }
}

pub fn get_init_and_subsequent_blocks_json_n(
    num: usize
) -> Result<EosInitAndSubsequentJson> {
    let path = match num {
        1 => Ok(SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_JUNGLE_3_JSON_1),
        2 => Ok(SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_MAINNET_JSON_1),
        3 => Ok(SAMPLE_INIT_AND_SUBSEQUENT_BLOCKS_MAINNET_JSON_2),
        _ => Err(AppError::Custom(
            format!("Cannot find sample block num: {}", num)
        ))
    }?;
    let string = match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err(AppError::Custom(
            format!("✘ Can't find sample init block json file @ path: {}", path)
        ))
    }?;
    EosInitAndSubsequentJson::from_json_string(&string)
}

pub fn get_init_json_n(num: usize) -> Result<EosInitJson> {
    let path = match num {
        1 => Ok(SAMPLE_INIT_BLOCK_JSON_PATH_1),
        2 => Ok(SAMPLE_INIT_BLOCK_JSON_PATH_2),
        3 => Ok(SAMPLE_INIT_BLOCK_JSON_PATH_3),
        _ => Err(AppError::Custom(
            format!("Cannot find sample block num: {}", num)
        ))
    }?;
    let string = match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err(AppError::Custom(
            format!("✘ Can't find sample init block json file @ path: {}", path)
        ))
    }?;
    EosInitJson::from_json_string(&string)
}

pub fn sha256_hash_message_bytes(
    message_bytes: &Bytes
) -> Result<Secp256k1Message> {
    Ok(Secp256k1Message::from_slice(&sha256::Hash::hash(message_bytes))?)
}

pub fn get_sample_v2_schedule_json_string() -> Result<String> {
    Ok(
        read_to_string(
            "src/btc_on_eos/eos/eos_test_utils/sample-schedule-v2.0.json"
        )?
    )
}

pub fn get_sample_v2_schedule_json() -> Result<EosProducerScheduleJson> {
    get_sample_v2_schedule_json_string()
        .and_then(|json_string| parse_schedule_string_to_json(&json_string))
}

pub fn get_sample_v2_schedule() -> Result<EosProducerScheduleV2> {
    get_sample_v2_schedule_json()
        .and_then(|json| convert_schedule_json_to_schedule_v2(&json))
}

pub fn get_sample_eos_submission_material_n(
    n: usize
) -> EosSubmissionMaterial {
    parse_eos_submission_material_string_to_struct(
        &get_sample_eos_submission_material_string_n(n).unwrap()
    ).unwrap()
}

pub fn get_sample_eos_submission_material_json_n(
    n: usize
) -> EosSubmissionMaterialJson {
    parse_eos_submission_material_string_to_json(
        &get_sample_eos_submission_material_string_n(n).unwrap()
    ).unwrap()
}

pub fn get_sample_eos_submission_material_string_n(
    num: usize,
) -> Result<String> {
    let path = match num {
        1 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_1),
        2 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_2),
        3 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_3),
        4 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_4),
        5 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_5),
        6 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_6),
        7 => Ok(SAMPLE_EOS_BLOCK_AND_ACTION_JSON_PATH_7),
        _ => Err(AppError::Custom(
            format!("Cannot find sample block num: {}", num)
        ))
    }?;
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err(AppError::Custom(
            format!("✘ Cannot find sample-eos-block-and-action-json file!")
        ))
    }
}

pub fn get_sample_eos_private_key_str() -> &'static str {
    "5K8ufCfDxaFXqkRdeGmLywEh32F3MZf67E8hFFvQoH3imDwQ9Ea"
}

pub fn get_sample_eos_public_key_str() -> &'static str {
    "EOS5vMQQqeG6ixyaLSvQacyZe9bH1kmMeYY296fFdc3d3317MdV2f"
}

pub fn get_sample_eos_private_key() -> EosPrivateKey {
    EosPrivateKey::from_wallet_import_format(
        get_sample_eos_private_key_str()
    ).unwrap()
}

pub fn get_sample_eos_public_key() -> EosPublicKey {
    get_sample_eos_private_key()
        .to_public_key()
}

pub fn get_sample_eos_public_key_bytes() -> Bytes {
    get_sample_eos_public_key()
        .to_bytes()
}

pub fn get_sample_eos_signature() -> EosSignature {
    get_sample_eos_private_key()
        .sign_message_bytes(&get_sample_message_to_sign().as_bytes())
        .unwrap()
}

fn get_sample_action_receipts() -> Vec<EosActionReceipt> {
    vec![
        EosActionReceipt::new(
            "eosio",
            "3b434aa9331f5e2a0e7a0060d576fa6688406667100bdf3458104dede44ec4e9",
            62826453,
            12,
            503081363,
            10,
            vec![AuthSequence::new(
                "eosio",
                61285932
            ).unwrap()],
        ).unwrap(),
        EosActionReceipt::new(
            "pokerpokerts",
            "3d380413463e8716ef9c1f8c853dfab0c70f209cce75cae9a5b74e4e678a68a0",
            241512,
            4,
            503081364,
            30,
            vec![AuthSequence::new(
                "pokerpokerts",
                241552
            ).unwrap()],
        ).unwrap(),
        EosActionReceipt::new(
            "oracleoracle",
            "065527f0429dfa9bb79575ec5270b20f714fb9e61a9ce6ba9c86b2e69a773f82",
            531231,
            2,
            503081365,
            2,
            vec![AuthSequence::new(
                "feeder111112",
                152730
            ).unwrap()],
        ).unwrap(),
        EosActionReceipt::new(
            "dvmh1tbb1him",
            "18e42aa86473509cf620764ca606136b037e1a8ee6fb8efaa8fa657c7fa2fffc",
            805647,
            2,
            503081366,
            1,
            vec![AuthSequence::new(
                "dvmh1tbb1him",
                805667
            ).unwrap()],
        ).unwrap(),
    ]
}

pub fn get_sample_action_digests() -> Vec<Bytes> {
    get_sample_action_receipts()
        .into_iter()
        .map(|receipt| receipt.to_digest())
        .collect()
}

fn get_sample_action_proofs_n(
    n: usize
) -> ActionProofs {
    get_sample_eos_submission_material_n(n).action_proofs
}

pub fn get_sample_action_proof_n(
    n: usize
) -> ActionProof {
    get_sample_action_proofs_n(n)[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_sample_eos_blocks_n() {
        let max = 1;
        for i in 1..max {
            get_sample_eos_submission_material_json_n(i);
        }
    }
}
