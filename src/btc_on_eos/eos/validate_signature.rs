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
        eos_merkle_utils::IncreMerkle,
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
    interim_block_ids: &Checksum256s,
) -> Bytes {
    IncreMerkle::new(
        block_header.block_num().into(),
        interim_block_ids.clone(),
    ).get_root().to_bytes().to_vec()
}

fn get_schedule_hash(active_schedule: &EosProducerScheduleV2) -> Result<Bytes> {
    Ok(active_schedule.schedule_hash()? .to_bytes().to_vec())
}

fn get_signing_digest(
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerScheduleV2,
    interim_block_ids: &Checksum256s,
) -> Result<Bytes> {
    Ok(
        create_eos_signing_digest(
            get_block_mroot(block_header, interim_block_ids),
            get_schedule_hash(active_schedule)?,
            get_block_digest(block_header)?,
        )
    )
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
    producer_signature: &String,
    block_header: &EosBlockHeader,
    active_schedule: &EosProducerScheduleV2,
    interim_block_ids: &Checksum256s,
) -> Result<EosPublicKey> {
    EosPublicKey::recover_from_digest(
        &Message::from_slice(
            &get_signing_digest(
                &block_header,
                &active_schedule,
                &interim_block_ids,
            )?,
        )?,
        &EosSignature::from_str(producer_signature)?
    )
}

fn check_block_signature_is_valid(
    producer_signature: &String,
    block_header: &EosBlockHeader,
    interim_block_ids: &Checksum256s,
    active_schedule: &EosProducerScheduleV2,
) -> Result<()> {
    let signing_key = get_signing_key_from_active_schedule(
        block_header.producer,
        active_schedule,
    )?.to_string();
    let recovered_key = recover_block_signer_public_key(
        producer_signature,
        block_header,
        active_schedule,
        interim_block_ids,
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
        &state.interim_block_ids,
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
    use super::*;
    use eos_primitives::{
        Write,
        NumBytes,
    };
    use crate::btc_on_eos::{
        eos::{
            eos_types::EosSubmissionMaterial,
            eos_test_utils::{
                NUM_SAMPLES,
                get_sample_v2_schedule,
                get_sample_active_schedule,
                get_sample_eos_submission_material_n,
            },
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
            &submission_material.interim_block_ids,
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
        let active_schedule = get_sample_active_schedule(389)
            .unwrap();
        let result = get_schedule_hash(&active_schedule)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_signing_key_from_active_schedule() {
        let expected_result =
            "EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH";
        let submission_material = get_sample_eos_submission_material_n(1);
        let active_schedule = get_sample_active_schedule(
            submission_material.block_header.schedule_version
        ).unwrap();
        let result = get_signing_key_from_active_schedule(
            submission_material.block_header.producer,
            &active_schedule,
        )
            .unwrap()
            .to_string();
        assert_eq!(&result, expected_result);
    }

    #[test]
    fn should_get_correct_signing_digest_from_parts() {
        let submission_material = get_sample_eos_submission_material_n(5);
        let active_schedule = get_sample_active_schedule(
            submission_material.block_header.schedule_version
        ).unwrap();

        let expected_block_digest =
            "3883dd314f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda";
        let block_digest = get_block_digest(&submission_material.block_header)
            .unwrap();
        assert_eq!(hex::encode(&block_digest), expected_block_digest);

        let expected_schedule_hash =
            "4204d5ca327bae53aac3b5405e356172d2b2dd42c2f609f4f970e41d0d3dcae1";
        let schedule_hash = get_schedule_hash(
            &active_schedule
        ).unwrap();
        assert_eq!(hex::encode(&schedule_hash), expected_schedule_hash);

        let expected_block_mroot =
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let block_mroot = get_block_mroot(
            &submission_material.block_header,
            &submission_material.interim_block_ids,
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
        let active_schedule = get_sample_active_schedule(
            submission_material.block_header.schedule_version
        ).unwrap();
        let expected_signing_digest =
            "e991ea00a9c3564fc9c6de33dc19865abbe5ac4bf643036ecd89f95e31c49521";
        let result = get_signing_digest(
            &submission_material.block_header,
            &active_schedule,
            &submission_material.interim_block_ids,
        ).unwrap();
        assert_eq!(hex::encode(result), expected_signing_digest);
    }

    #[test]
    fn should_get_signing_key_for_all_submission_materials() {
        let expected_results = vec![
            "EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH",
            "EOS79e4HpvQ1y1HdzRSqE1gCKhdN9kFGjjUU2nKG7xjiVCakt5WSs",
            "EOS5hVMcN6UVWtrNCxdp5HJwsz4USULmdfNA22UDyjRNdprXEiAP6",
            "EOS6wkp1PpqQUgEA6UtgW21Zo3o1XcQeLXzcLLgKcPJhTz2aSF6fz",
            "EOS7A9BoRetjpKtE3sqA6HRykRJ955MjQ5XdRmCLionVte2uERL8h",
        ];
        let active_schedules = vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material|
                get_sample_active_schedule(
                    submission_material.block_header.schedule_version
                )
            )
            .collect::<Result<Vec<EosProducerScheduleV2>>>()
            .unwrap();
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .zip(active_schedules.iter())
            .map(|(submission_material, active_schedule)|
                 get_signing_key_from_active_schedule(
                     submission_material.block_header.producer,
                     &active_schedule,
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
        let submission_materials = vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .collect::<Vec<EosSubmissionMaterial>>();
        let active_schedules = submission_materials
            .iter()
            .map(|submission_material|
                get_sample_active_schedule(
                    submission_material.block_header.schedule_version
                )
            )
            .collect::<Result<Vec<EosProducerScheduleV2>>>()
            .unwrap();
        let expected_results = submission_materials
            .iter()
            .zip(active_schedules.iter())
            .map(|(submission_material, active_schedule)|
                 get_signing_key_from_active_schedule(
                     submission_material.block_header.producer,
                     &active_schedule,
                 )
             )
            .collect::<Result<Vec<PublicKey>>>()
            .unwrap();
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .zip(active_schedules.iter())
            .map(|(material, active_schedule)|
                 recover_block_signer_public_key(
                    &material.producer_signature,
                    &material.block_header,
                    &active_schedule,
                    &material.interim_block_ids,
                 )
             )
            .collect::<Result<Vec<EosPublicKey>>>()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, key)|
                assert_eq!(key.to_string(), expected_results[i].to_string())
            )
            .for_each(drop);
    }

    #[test]
    fn samples_blocks_should_be_valid() {
        let submission_materials = vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .collect::<Vec<EosSubmissionMaterial>>();
        let active_schedules = submission_materials
            .iter()
            .map(|submission_material|
                get_sample_active_schedule(
                    submission_material.block_header.schedule_version
                )
            )
            .collect::<Result<Vec<EosProducerScheduleV2>>>()
            .unwrap();
        submission_materials
            .iter()
            .zip(active_schedules.iter())
            .map(|(submission_material, active_schedule)|
                 check_block_signature_is_valid(
                    &submission_material.producer_signature,
                    &submission_material.block_header,
                    &submission_material.interim_block_ids,
                    &active_schedule,
                 )
             )
            .collect::<Result<Vec<()>>>()
            .unwrap();
    }

    #[test]
    fn should_x() {
        let submission_material = get_sample_eos_submission_material_n(6);
        let producer_signature = submission_material
            .producer_signature
            .clone();
        let block_mroot = get_block_mroot(
            &submission_material.block_header,
            &submission_material.interim_block_ids,
        );
        let block_header_digest = get_block_digest(
            &submission_material.block_header
        ).unwrap();
        let schedule_hash = get_sample_v2_schedule()
            .unwrap()
            .schedule_hash()
            .unwrap()
            .to_bytes()
            .to_vec();
        let sched_hash_2 = hex::decode("a722944989081591e0b9742e3065206251a0041e4480cd6a6642ce929f255194").unwrap();
        let signing_digest = create_eos_signing_digest(
            block_mroot.clone(),
            schedule_hash.clone(),
            //sched_hash_2.clone(),
            block_header_digest.clone(),
        );
        let recovered_pub_key = EosPublicKey::recover_from_digest(
            &Message::from_slice(&signing_digest).unwrap(),
            &EosSignature::from_str(&producer_signature).unwrap(),
        ).unwrap();
        let expected_pub_key =
            "EOS8X5NCx1Xqa1xgQgBa9s6EK7M1SjGaDreAcLion4kDVLsjhQr9n";

        println!("      schedule: {:?}", get_sample_v2_schedule().unwrap());
        println!("      producer: {}", submission_material.block_header.producer);
        println!("  block digest: {}", hex::encode(block_header_digest));
        println!("   block mroot: {}", hex::encode(block_mroot));
        println!(" schedule hash: {}", hex::encode(schedule_hash));
        println!("schedule hash2: {}", hex::encode(sched_hash_2));
        println!("signing digest: {}", hex::encode(signing_digest));
        println!(" recovered key: {}", recovered_pub_key);
        println!("  expected key: {}", expected_pub_key);
        println!("       matches: {}", expected_pub_key == &recovered_pub_key.to_string());

        let sched_serialized = get_sample_v2_schedule()
            .unwrap();
        let mut data = vec![0u8; sched_serialized.num_bytes()];
        sched_serialized.write(&mut data, &mut 0).unwrap();

        println!("sched num bytes: {}", get_sample_v2_schedule().unwrap().num_bytes());
        println!("sched serialized: {}", hex::encode(data));
    }
}
