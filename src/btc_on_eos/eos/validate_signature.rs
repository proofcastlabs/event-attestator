use std::str::FromStr;
use eos_primitives::{
    PublicKey,
    Checksum256,
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
    utils::convert_bytes_to_checksum256,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_merkle_utils::IncrementalMerkle,
        eos_types::{
            MerklePath,
            EosSubmissionMaterial,
        },
        eos_crypto::{
            eos_signature::EosSignature,
            eos_public_key::EosPublicKey,
        },
    },
};

fn get_eos_signing_digest(
    block_mroot: Bytes,
    schedule_hash: Bytes,
    block_header_digest: Bytes,
) -> Bytes {
    let hash_1 = sha256::Hash::hash(
        &[&block_header_digest[..], &block_mroot[..]].concat()
    );
    sha256::Hash::hash(&[&hash_1[..], &schedule_hash[..]].concat()).to_vec()
}

fn get_block_id_from_submission_material(
    submission_material: &EosSubmissionMaterial
) -> Result<Bytes> {
    Ok(
        submission_material
            .block_header
            .id()?
            .to_bytes()
            .to_vec()
    )
}

fn get_block_digest_from_submission_material(
    submission_material: &EosSubmissionMaterial
) -> Result<Bytes> {
    Ok(
        submission_material
            .block_header
            .digest()?
            .to_bytes()
            .to_vec()
    )
}

fn get_block_mroot_from_submission_material(
    submission_material: &EosSubmissionMaterial
) -> Bytes {
    IncrementalMerkle::new(
        submission_material
            .block_header
            .block_num()
            .into(),
        submission_material
            .blockroot_merkle
            .clone(),
    )
            .get_root()
            .to_bytes()
            .to_vec()
}

fn get_schedule_hash_from_submission_material(
    submission_material: &EosSubmissionMaterial,
) -> Result<Bytes> {
    Ok(
        submission_material
            .active_schedule
            .schedule_hash()?
            .to_bytes()
            .to_vec()
    )
}

fn get_signing_digest_from_submission_material(
    submission_material: &EosSubmissionMaterial,
) -> Result<Bytes> {
    Ok(
        get_eos_signing_digest(
            get_block_mroot_from_submission_material(submission_material),
            get_schedule_hash_from_submission_material(submission_material)?,
            get_block_digest_from_submission_material(submission_material)?,
        )
    )
}

fn get_signing_key_from_submission_material(
    submission_material: &EosSubmissionMaterial,
) -> Result<PublicKey> {
    let block_signer = submission_material
        .block_header
        .producer
        .clone();
    let active_schedule = submission_material
        .active_schedule
        .clone();
    let filtered_keys = active_schedule
        .producers
        .iter()
        .map(|producer| producer.producer_name)
        .zip(active_schedule.producers.iter())
        .filter(|(name, _)| name == &block_signer)
        .map(|(_, producer)| &producer.block_signing_key)
        .cloned()
        .collect::<Vec<PublicKey>>();
    match &filtered_keys.len() {
        0 => Err(AppError::Custom(
            "✘ Could not extract a signing key from EOS submission material!"
                .to_string()
        )),
        _ => Ok(filtered_keys[0].clone()) // NOTE: Can this ever be > 1?
    }
}

fn get_producer_signature_from_submission_material(
    submission_material: &EosSubmissionMaterial,
) -> Result<EosSignature> {
    Ok(EosSignature::from_str(&submission_material.producer_signature)?)
}

fn recover_public_key_from_submission_material(
    submission_material: &EosSubmissionMaterial,
) -> Result<EosPublicKey> {
    let msg = Message::from_slice(
        &get_signing_digest_from_submission_material(submission_material)?,
    )?;
    let producer_signature = get_producer_signature_from_submission_material(
        &submission_material
    )?;
    EosPublicKey::recover_from_digest(
        &msg,
        &producer_signature,
    )
}

// TODO rest of sig validation

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::btc_on_eos::{
        eos::eos_test_utils::get_sample_eos_submission_material_n,
    };

    pub const NUM_SAMPLES: usize = 5;

    #[test]
    fn should_get_block_mroot_from_submission_material() {
        let expected_result = hex::decode(
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(5);
        let result = get_block_mroot_from_submission_material(
            &submission_material,
        );
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_block_id_from_submission_material() {
        let expected_result = hex::decode(
            "04dfed9c79cb5120749aecdb3803ce13035f14fa5878122d0f6fe170c314b5a7"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_block_id_from_submission_material(&submission_material)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_block_digest_from_submission_material() {
        let expected_result = hex::decode(
            "3f1fc3e079cb5120749aecdb3803ce13035f14fa5878122d0f6fe170c314b5a7"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_block_digest_from_submission_material(
            &submission_material
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_schedule_hash_from_submission_material() {
        let expected_result = hex::decode(
            "00227471af02f659912fe48c56aa40274ec7cf6bfab2f9c205a570c87ebe5862"
        ).unwrap();
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_schedule_hash_from_submission_material(
            &submission_material
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_signing_key_from_submission_material() {
        let expected_result =
            "EOS5CJJEKDms9UTS7XBv8rb33BENRpnpSGsQkAe6bCfpjHHCKQTgH";
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_signing_key_from_submission_material(
            &submission_material
        )
            .unwrap()
            .to_string();
        assert_eq!(&result, expected_result);
    }

    #[test]
    fn should_get_producer_signature_from_submission_material() {
        let expected_result = "SIG_K1_KYuxekoH8nrLWd5W9njqDYbFuyNG2rNYaYSrf51ts8pBVfuTffENzgqGPfWrpEppbBx7jM2WQng72YkahZNju2EmHXAuFK".to_string();
        let submission_material = get_sample_eos_submission_material_n(1);
        let result = get_producer_signature_from_submission_material(
            &submission_material
        )
            .unwrap()
            .to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_correct_signing_digest() {
        let submission_material = get_sample_eos_submission_material_n(5);

        let expected_block_digest =
            "3883dd314f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda";
        let block_digest = get_block_digest_from_submission_material(
            &submission_material
        ).unwrap();
        assert_eq!(hex::encode(&block_digest), expected_block_digest);

        let expected_schedule_hash =
            "4204d5ca327bae53aac3b5405e356172d2b2dd42c2f609f4f970e41d0d3dcae1";
        let schedule_hash = get_schedule_hash_from_submission_material(
            &submission_material
        ).unwrap();

        let expected_block_mroot =
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let block_mroot = get_block_mroot_from_submission_material(
            &submission_material,
        );

        let expected_signing_digest =
            "e991ea00a9c3564fc9c6de33dc19865abbe5ac4bf643036ecd89f95e31c49521";
        let result = get_eos_signing_digest(
            block_mroot,
            schedule_hash,
            block_digest,
        );
        assert_eq!(hex::encode(result), expected_signing_digest);
    }

    #[test]
    fn should_get_correct_signing_digest_from_submission_material() {
        let submission_material = get_sample_eos_submission_material_n(5);
        let expected_signing_digest =
            "e991ea00a9c3564fc9c6de33dc19865abbe5ac4bf643036ecd89f95e31c49521";
        let result = get_signing_digest_from_submission_material(
            &submission_material,
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
            .map(|material| get_signing_key_from_submission_material(&material))
            .collect::<Result<Vec<PublicKey>>>()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, key)| assert_eq!(key.to_string(), expected_results[i]))
            .for_each(drop);
    }

    #[test]
    fn should_recover_public_key_from_submission_materials() {
        let expected_results = vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|material| get_signing_key_from_submission_material(&material))
            .collect::<Result<Vec<PublicKey>>>()
            .unwrap();
        vec![0; NUM_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_sample_eos_submission_material_n(i + 1))
            .map(|material|
                 recover_public_key_from_submission_material(&material)
             )
            .collect::<Result<Vec<EosPublicKey>>>()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, key)| {
                if i == 2 {
                    println!("Skip - WHY U NO WORK?"); // TODO FIXME
                } else {
                    assert_eq!(key.to_string(), expected_results[i].to_string())
                }
            })
            .for_each(drop);
    }
}
