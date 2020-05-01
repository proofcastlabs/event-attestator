use std::str::FromStr;
use eos_primitives::{
    PublicKey,
    AccountName as EosAccountName,
    BlockHeader as EosBlockHeader,
    ProducerScheduleV2 as EosProducerScheduleV2,
};
use bitcoin_hashes::{
    Hash,
    sha256,
};
use secp256k1::{
    Message,
};
use crate::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
    },
    btc_on_eos::eos::{
        eos_state::EosState,
        eos_crypto::{
            eos_signature::EosSignature,
            eos_public_key::EosPublicKey,
        },
    },
};

fn create_eos_signing_digest(
    block_mroot: &Bytes,
    schedule_hash: &Bytes,
    block_header_digest: &Bytes,
) -> Bytes {
    let hash_1 = sha256::Hash::hash(
        &[&block_header_digest[..], &block_mroot[..]].concat()
    );
    sha256::Hash::hash(&[&hash_1[..], &schedule_hash[..]].concat()).to_vec()
}

fn get_block_digest(block_header: &EosBlockHeader) -> Result<Bytes> {
    Ok(block_header.digest()?.to_bytes().to_vec())
}

fn get_schedule_hash(active_schedule: &EosProducerScheduleV2) -> Result<Bytes> {
    Ok(active_schedule.schedule_hash()? .to_bytes().to_vec())
}

fn get_signing_digest(
    block_mroot: &Bytes,
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerScheduleV2,
) -> Result<Bytes> {
    let block_digest = get_block_digest(block_header)?;
    let schedule_hash = get_schedule_hash(active_schedule)?;
    let signing_digest = create_eos_signing_digest(
        block_mroot,
        &schedule_hash,
        &block_digest
    );
    debug!("   block mroot: {}", hex::encode(&block_mroot));
    debug!("  block digest: {}", hex::encode(&block_digest));
    debug!(" schedule hash: {}", hex::encode(&schedule_hash));
    debug!("      schedule: {:?}", active_schedule);
    debug!("signing digest: {}", hex::encode(&signing_digest));
    Ok(signing_digest)
}

fn get_signing_key_from_active_schedule(
    block_producer: EosAccountName,
    active_schedule: &EosProducerScheduleV2,
) -> Result<PublicKey> {
    let filtered_keys = active_schedule
        .producers
        .iter()
        .map(|producer| producer.producer_name)
        .zip(active_schedule.producers.iter())
        .filter(|(name_from_schedule, _)| *name_from_schedule == block_producer)
        // NOTE/FIXME We're only getting the first key so far.
        .map(|(_, producer)| &producer.authority.1.keys[0].key)
        .cloned()
        .collect::<Vec<PublicKey>>();
    match &filtered_keys.len() {
        0 => Err(AppError::Custom(
            "✘ Could not extract a signing key from active schedule!"
                .to_string()
        )),
        _ => Ok(filtered_keys[0].clone()) // NOTE: Can this ever be > 1?
    }
}

fn recover_block_signer_public_key(
    block_mroot: &Bytes,
    producer_signature: &String,
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerScheduleV2,
) -> Result<EosPublicKey> {
    EosPublicKey::recover_from_digest(
        &Message::from_slice(
            &get_signing_digest(
                &block_mroot,
                &block_header,
                &active_schedule,
            )?,
        )?,
        &EosSignature::from_str(producer_signature)?
    )
}

pub fn check_block_signature_is_valid(
    block_mroot: &Bytes,
    producer_signature: &String,
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerScheduleV2,
) -> Result<()> {
    let signing_key = get_signing_key_from_active_schedule(
        block_header.producer,
        active_schedule,
    )?.to_string();
    let recovered_key = recover_block_signer_public_key(
        block_mroot,
        producer_signature,
        block_header,
        active_schedule,
    )?.to_string();
    match signing_key == recovered_key {
        true => Ok(()),
        _ => Err(AppError::Custom("✘ Block signature not valid!".to_string()))
    }
}

pub fn validate_block_header_signature<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Validating EOS block header signature...");
    check_block_signature_is_valid(
        &state.incremerkle
            .get_root()
            .to_bytes()
            .to_vec(),
        &state.producer_signature,
        state.get_eos_block_header()?,
        state.get_active_schedule()?,
    )
        .and(Ok(state))
}

// TODO fixme! These all are now failing since we don't use the blockroot merkle
// that we pass in anymore. Instead we're going to keep the incremerkle in the
// db and append to it any blocks in the interim between two submissions.
// Update this module to reflect that!
#[cfg(test)]
mod tests {
    #![allow(unused_imports)] // FIXME
    use super::*;
    use eos_primitives::{
        Write,
        NumBytes,
        Checksum256,
    };
    use crate::btc_on_eos::{
        utils::convert_hex_to_checksum256,
        eos::{
            eos_merkle_utils::Incremerkle,
            parse_eos_schedule::convert_schedule_json_to_schedule_v2,
            parse_submission_material::parse_eos_block_header_from_json,
            eos_types::EosSubmissionMaterial,
            eos_test_utils::{
                get_init_json_n,
                get_sample_v2_schedule,
                get_sample_eos_submission_material_n,
                get_init_and_subsequent_blocks_json_n,
            },
        },
    };

    #[test]
    fn should_get_block_digest() {
        let expected_result = hex::decode(
            "3f1fc3e079cb5120749aecdb3803ce13035f14fa5878122d0f6fe170c314b5a7"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_block_digest(&submission_material.block_header)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_schedule_hash() {
        let expected_result = hex::decode(
            "a722944989081591e0b9742e3065206251a0041e4480cd6a6642ce929f255194"
        ).unwrap();
        let active_schedule = get_sample_v2_schedule()
            .unwrap();
        let result = get_schedule_hash(&active_schedule)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_validate_initial_and_subequent_jungle_3_blocks() {
        let blocks_json = get_init_and_subsequent_blocks_json_n(1)
            .unwrap();
        blocks_json.init_block.validate();
        let block_num = 2;
        let producer_signature = blocks_json
            .get_producer_signature_for_block_n(block_num)
            .unwrap();
        let block_header = blocks_json.get_block_n(block_num)
            .unwrap();
        let active_schedule = blocks_json.get_active_schedule()
            .unwrap();
        let block_mroot = blocks_json
            .get_block_mroot_for_block_n(block_num)
            .unwrap();
        if let Err(e) = check_block_signature_is_valid(
            &block_mroot,
            &producer_signature,
            &block_header,
            &active_schedule,
        ) {
            panic!("{}", e);
        }
    }

    #[test]
    fn should_validate_initial_and_subequent_mainnet_blocks() {
        /*
        use simplelog::{
            TermLogger,
            LevelFilter,
            Config,
            TerminalMode,
        };
        TermLogger::init(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed
        ).unwrap();
        */
        let blocks_json = get_init_and_subsequent_blocks_json_n(3)
            .unwrap();
        blocks_json.init_block.validate();
        let block_num = 2;
        let producer_signature = blocks_json
            .get_producer_signature_for_block_n(block_num)
            .unwrap();
        let block_header = blocks_json.get_block_n(block_num)
            .unwrap();
        let active_schedule = blocks_json.get_active_schedule()
            .unwrap();
        let block_mroot = blocks_json
            .get_block_mroot_for_block_n(block_num)
            .unwrap();
        if let Err(e) = check_block_signature_is_valid(
            &block_mroot,
            &producer_signature,
            &block_header,
            &active_schedule,
        ) {
            panic!("{}", e);
        }
    }
}
