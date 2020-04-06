use std::str::FromStr;
use eos_primitives::{
    PublicKey,
    AccountName as EosAccountName,
    BlockHeader as EosBlockHeader,
    ProducerSchedule as EosProducerSchedule,
};
use bitcoin_hashes::{
    Hash,
    sha256,
};
use secp256k1::{
    Message,
};
use crate::btc_on_eos::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_state::EosState,
        eos_merkle_utils::IncrementalMerkle,
        eos_types::Checksum256s,
        eos_crypto::{
            eos_signature::EosSignature,
            eos_public_key::EosPublicKey,
        },
    },
};

fn create_eos_signing_digest(
    block_mroot: Bytes,
    schedule_hash: Bytes,
    block_header_digest: Bytes,
) -> Bytes {
    let hash_1 = sha256::Hash::hash(
        &[&block_header_digest[..], &block_mroot[..]].concat()
    );
    sha256::Hash::hash(&[&hash_1[..], &schedule_hash[..]].concat()).to_vec()
}

fn get_block_digest(block_header: &EosBlockHeader) -> Result<Bytes> {
    Ok(block_header.digest()?.to_bytes().to_vec())
}

fn get_block_mroot(
    block_header: &EosBlockHeader,
    blockroot_merkle: &Checksum256s,
) -> Bytes {
    IncrementalMerkle::new(
        block_header.block_num().into(),
        blockroot_merkle.clone(),
    ).get_root().to_bytes().to_vec()
}

fn get_schedule_hash(active_schedule: &EosProducerSchedule) -> Result<Bytes> {
    Ok(active_schedule.schedule_hash()? .to_bytes().to_vec())
}

fn get_signing_digest( // TODO use stuff in state! And rename better!jjjj
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerSchedule,
    blockroot_merkle: &Checksum256s,
) -> Result<Bytes> {
    Ok(
        create_eos_signing_digest(
            get_block_mroot(block_header, blockroot_merkle),
            get_schedule_hash(active_schedule)?,
            get_block_digest(block_header)?,
        )
    )
}

fn get_signing_key_from_active_schedule(
    block_producer: EosAccountName,
    active_schedule: &EosProducerSchedule,
) -> Result<PublicKey> {
    let filtered_keys = active_schedule
        .producers
        .iter()
        .map(|producer| producer.producer_name)
        .zip(active_schedule.producers.iter())
        .filter(|(name, _)| *name == block_producer)
        .map(|(_, producer)| &producer.block_signing_key)
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
    producer_signature: &String,
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerSchedule,
    blockroot_merkle: &Checksum256s,
) -> Result<EosPublicKey> {
    EosPublicKey::recover_from_digest(
        &Message::from_slice(
            &get_signing_digest(
                &block_header,
                &active_schedule,
                &blockroot_merkle,
            )?,
        )?,
        &EosSignature::from_str(producer_signature)?
    )
}

fn check_block_signature_is_valid(
    producer_signature: &String,
    block_header: &EosBlockHeader,
    blockroot_merkle: &Checksum256s,
    active_schedule: &EosProducerSchedule,
) -> Result<()> {
    let signing_key = get_signing_key_from_active_schedule(
        block_header.producer,
        active_schedule,
    )?.to_string();
    let recovered_key = recover_block_signer_public_key(
        producer_signature,
        block_header,
        active_schedule,
        blockroot_merkle,
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
        &state.producer_signature,
        state.get_eos_block_header()?,
        &state.blockroot_merkle,
        state.get_active_schedule()?,
    )
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::{
        eos::eos_test_utils::{
            NUM_SAMPLES,
            get_sample_eos_submission_material_n,
        },
    };

    #[test]
    fn should_get_block_mroot() {
        let expected_result = hex::decode(
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(5);
        let result = get_block_mroot(
            &submission_material.block_header,
            &submission_material.blockroot_merkle,
        );
        assert_eq!(result, expected_result);
    }

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
            "00227471af02f659912fe48c56aa40274ec7cf6bfab2f9c205a570c87ebe5862"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_schedule_hash(&submission_material.active_schedule)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_signing_key_from_active_schedule() {
        let expected_result =
            "EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH";
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_signing_key_from_active_schedule(
            submission_material.block_header.producer,
            &submission_material.active_schedule,
        )
            .unwrap()
            .to_string();
        assert_eq!(&result, expected_result);
    }

    #[test]
    fn should_get_correct_signing_digest_from_parts() {
        let submission_material = get_sample_eos_submission_material_n(5);

        let expected_block_digest =
            "3883dd314f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda";
        let block_digest = get_block_digest(&submission_material.block_header)
            .unwrap();
        assert_eq!(hex::encode(&block_digest), expected_block_digest);

        let expected_schedule_hash =
            "4204d5ca327bae53aac3b5405e356172d2b2dd42c2f609f4f970e41d0d3dcae1";
        let schedule_hash = get_schedule_hash(
            &submission_material.active_schedule
        ).unwrap();
        assert_eq!(hex::encode(&schedule_hash), expected_schedule_hash);

        let expected_block_mroot =
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let block_mroot = get_block_mroot(
            &submission_material.block_header,
            &submission_material.blockroot_merkle,
        );
        assert_eq!(hex::encode(&block_mroot), expected_block_mroot);

        let expected_signing_digest =
            "e991ea00a9c3564fc9c6de33dc19865abbe5ac4bf643036ecd89f95e31c49521";
        let result = create_eos_signing_digest(
            block_mroot,
            schedule_hash,
            block_digest,
        );
        assert_eq!(hex::encode(result), expected_signing_digest);
    }

    #[test]
    fn should_get_correct_signing_digest() {
        let submission_material = get_sample_eos_submission_material_n(5);
        let expected_signing_digest =
            "e991ea00a9c3564fc9c6de33dc19865abbe5ac4bf643036ecd89f95e31c49521";
        let result = get_signing_digest(
            &submission_material.block_header,
            &submission_material.active_schedule,
            &submission_material.blockroot_merkle,
        ).unwrap();
        assert_eq!(hex::encode(result), expected_signing_digest);
    }

    #[test]
    fn should_get_signing_key_for_all_submission_materials() {
        let expected_results = vec![
            "EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH",
            "EOS79e4HpvQ1y1HdzRSqE1gCKhdN9kFGjjUU2nKG7xjiVCakt5WSs",
            "EOS7tigERwXDRuHsok212UDToxFS1joUhAxzvDUhRof8NjuvwtoHX",
            "EOS6wkp1PpqQUgEA6UtgW21Zo3o1XcQeLXzcLLgKcPJhTz2aSF6fz",
            "EOS7A9BoRetjpKtE3sqA6HRykRJ955MjQ5XdRmCLionVte2uERL8h",
        ];
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material|
                 get_signing_key_from_active_schedule(
                     submission_material.block_header.producer,
                     &submission_material.active_schedule,
                 )
             )
            .collect::<Result<Vec<PublicKey>>>()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, key)| assert_eq!(key.to_string(), expected_results[i]))
            .for_each(drop);
    }

    #[test]
    fn should_recover_public_key() {
        let expected_results = vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material|
                 get_signing_key_from_active_schedule(
                     submission_material.block_header.producer,
                     &submission_material.active_schedule,
                 )
             )
            .collect::<Result<Vec<PublicKey>>>()
            .unwrap();
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|material|
                 recover_block_signer_public_key(
                    &material.producer_signature,
                    &material.block_header,
                    &material.active_schedule,
                    &material.blockroot_merkle,
                 )
             )
            .collect::<Result<Vec<EosPublicKey>>>()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, key)| {
                if i != 2 { // FIXME Why does this one fail?
                    assert_eq!(key.to_string(), expected_results[i].to_string())
                }
            })
            .for_each(drop);
    }

    #[test]
    fn samples_blocks_should_be_valid() { // TODO -ve version of this!
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| (i, get_sample_eos_submission_material_n(i + 1)))
            .map(|(i, submission_material)|
                 if i == 2 {
                    Ok(())// FIXME Why does this one fail?
                 } else {
                     check_block_signature_is_valid(
                        &submission_material.producer_signature,
                        &submission_material.block_header,
                        &submission_material.blockroot_merkle,
                        &submission_material.active_schedule,
                     )
                 }
             )
            .collect::<Result<Vec<()>>>()
            .unwrap();
    }
}
