use std::str::FromStr;
use chrono::prelude::*;
use eos_primitives::{
    Extension,
    TimePoint,
    AccountName,
    BlockTimestamp,
    BlockHeader as EosBlockHeader,
};
use crate::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    btc_on_eos::{
        utils::convert_hex_to_checksum256,
        eos::{
            eos_state::EosState,
            parse_eos_schedule::convert_schedule_json_to_schedule_v2,
            eos_types::{
                ActionProof,
                ActionProofs,
                Checksum256s,
                ActionProofJsons,
                EosBlockHeaderJson,
                EosSubmissionMaterial,
                EosSubmissionMaterialJson,
            },
        },
    },
};

fn parse_eos_action_proof_jsons_to_action_proofs(
    action_proof_jsons: &ActionProofJsons,
) -> Result<ActionProofs> {
    action_proof_jsons
        .iter()
        .map(|json| ActionProof::from_json(json))
        .collect()
}

pub fn parse_eos_submission_material_string_to_json(
    submission_material_string: &String
) -> Result<EosSubmissionMaterialJson> {
    match serde_json::from_str(submission_material_string) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

fn convert_timestamp_string_to_block_timestamp(
    timestamp: &str,
) -> Result<BlockTimestamp> {
    let timestamp_format = "%Y-%m-%dT%H:%M:%S%.3f";
    Ok(
        BlockTimestamp::from(
            TimePoint::from_unix_nano_seconds(
                Utc
                    .datetime_from_str(timestamp, timestamp_format)?
                    .timestamp_millis() * 1_000_000
            )
        )
    )
}

fn convert_hex_string_to_extension(hex_string: &str) -> Result<Extension> {
    Ok(Extension::new(hex::decode(hex_string)?))
}

fn convert_hex_strings_to_extensions(
    extension_strings: &Vec<String>,
) -> Result<Vec<Extension>> {
    extension_strings
        .iter()
        .map(|hex| convert_hex_string_to_extension(&hex))
        .collect::<Result<Vec<Extension>>>()
}

pub fn parse_eos_block_header_from_json(
    eos_block_header_json: &EosBlockHeaderJson
) -> Result<EosBlockHeader> {
    Ok(
        EosBlockHeader::new(
            convert_timestamp_string_to_block_timestamp(
                &eos_block_header_json.timestamp
            )?,
            AccountName::from_str(
                &eos_block_header_json.producer
            )?,
            eos_block_header_json.confirmed,
            convert_hex_to_checksum256(
                &eos_block_header_json.previous
            )?,
            convert_hex_to_checksum256(
                &eos_block_header_json.transaction_mroot
            )?,
            convert_hex_to_checksum256(
                &eos_block_header_json.action_mroot
            )?,
            eos_block_header_json.schedule_version,
            match &eos_block_header_json.new_producers {
                None => None,
                Some(producer_schedule_json) =>
                    Some(
                        convert_schedule_json_to_schedule_v2(
                            &producer_schedule_json
                        )?
                    )
            },
            match &eos_block_header_json.header_extension {
                None => vec![],
                Some(hex_strings) => convert_hex_strings_to_extensions(
                    &hex_strings
                )?
            },
        )
    )
}

fn parse_blockroot_merkle_from_json(
    blockroot_merkle_json: &Vec<String>,
) -> Result<Checksum256s> {
    blockroot_merkle_json
        .iter()
        .map(convert_hex_to_checksum256)
        .collect()
}

fn parse_eos_submission_material_json_to_struct(
    submission_material_json: EosSubmissionMaterialJson
) -> Result<EosSubmissionMaterial> {
    Ok(
        EosSubmissionMaterial {
            block_header: parse_eos_block_header_from_json(
                &submission_material_json.block_header,
            )?,
            blockroot_merkle: parse_blockroot_merkle_from_json(
                &submission_material_json.blockroot_merkle,
            )?,
            action_proofs: parse_eos_action_proof_jsons_to_action_proofs(
               &submission_material_json.action_proofs,
            )?,
            producer_signature: submission_material_json
                .block_header
                .producer_signature
                .clone(),
        }
    )
}

pub fn parse_eos_submission_material_string_to_struct(
    submission_material: &String,
) -> Result<EosSubmissionMaterial> {
    parse_eos_submission_material_string_to_json(submission_material)
        .and_then(parse_eos_submission_material_json_to_struct)
}

pub fn parse_submission_material_and_add_to_state<D>(
    submission_material: String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    parse_eos_submission_material_string_to_struct(&submission_material)
        .and_then(|material| state.add_submission_material(material))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::eos::eos_test_utils::{
        get_sample_eos_submission_material_string_n,
    };

    #[test]
    fn should_parse_eos_submission_material_string_to_json() {
        let string = get_sample_eos_submission_material_string_n(2)
            .unwrap();
        if let Err(e) = parse_eos_submission_material_string_to_json(&string) {
            panic!("Error parsing eos_block_and_json: {}", e);
        }
    }

    #[test]
    fn should_convert_timestamp_string_to_block_timestamp() {
        let expected_result = BlockTimestamp(1192621811);
        let eos_time_stamp_string = "2018-11-23T17:55:05.500";
        let result = convert_timestamp_string_to_block_timestamp(
            &eos_time_stamp_string
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_hex_string_to_extension() {
        let hex = "01030307";
        let expected_u16 = 769;
        let expected_bytes = [3u8, 7u8];
        let result = convert_hex_string_to_extension(&hex)
            .unwrap();
        assert_eq!(result.0, expected_u16);
        assert_eq!(result.1, expected_bytes);
    }

    #[test]
    fn should_parse_eos_block_header() {
        let expected_id = convert_hex_to_checksum256(
            &"04cb6d0413d124ea2df08183d579967e3e47c9853c40126f06110bb20e9330d4"
                .to_string()
        ).unwrap();
        let string = get_sample_eos_submission_material_string_n(2)
            .unwrap();
        let json = parse_eos_submission_material_string_to_json(&string)
            .unwrap();
        let result = parse_eos_block_header_from_json(&json.block_header)
            .unwrap();
        let id = result.id().unwrap();
        assert_eq!(id, expected_id);
    }

    #[test]
    fn should_parse_eos_submission_material_string_to_struct() {
        let string = get_sample_eos_submission_material_string_n(2)
            .unwrap();
        let json = parse_eos_submission_material_string_to_json(&string)
            .unwrap();
        if let Err(e) = parse_eos_submission_material_json_to_struct(json) {
            panic!("Error parsing submission json: {}", e);
        }
    }

    #[test]
    fn should_parse_block_header_from_json_2() {
        // NOTE: This block === https://jungle.bloks.io/block/10800
        // NOTE: Blocks herein chosen because of repo here:
        // https://github.com/KyberNetwork/bridge_eth_smart_contracts/tree/master/test
        // Which has producer keys etc as test vectors.
        let block_id =
            "00002a304f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda"
            .to_string();
        let expected_block_id = convert_hex_to_checksum256(&block_id)
            .unwrap();
        let json = EosBlockHeaderJson {
            block_id,
            confirmed: 0,
            producer: "funnyhamster".to_string(),
            previous: "00002a2fb72da881babc192b80bab59c289e2db1b4318160a4c0ab5e50618f57".to_string(),
            block_num: 1337,
            timestamp: "2018-11-23T17:55:05.500".to_string(),
            action_mroot: "33cfa41c93d0d37dd162d1341114122d76446ec6ce5ff6686205fa15f2fe6d46".to_string(),
            schedule_version: 2,
            transaction_mroot: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            producer_signature: "SIG_K1_KX9Y5xYQrBYtpdKm4njsMerfzoPU6qbiW3G3RmbmbSyZ5sjE2J1U4PHC1vQ8arZQrBKqwW1adLPwYDzqt3v137GLp1ZWse".to_string(), // Ignored
            header_extension: None,
            new_producers: None,
        };
        let result = parse_eos_block_header_from_json(&json)
            .unwrap();
        let expected_serialized = "f3f615477055c6d2343fa75e000000002a2fb72da881babc192b80bab59c289e2db1b4318160a4c0ab5e50618f57000000000000000000000000000000000000000000000000000000000000000033cfa41c93d0d37dd162d1341114122d76446ec6ce5ff6686205fa15f2fe6d46020000000000";
        let result_serialized = hex::encode(result.serialize().unwrap());
        assert_eq!(result.id().unwrap(), expected_block_id);
        assert_eq!(result_serialized, expected_serialized);
    }

    #[test]
    fn should_parse_block_header_from_json_3() {
        // NOTE: This block === https://jungle.bloks.io/block/10801
        let block_id =
            "00002a31c3261813a1e737a5b821a1f318f731ff12c5dd9cc14dc2a1c661fce6"
            .to_string();
        let expected_block_id = convert_hex_to_checksum256(&block_id)
            .unwrap();
        let json = EosBlockHeaderJson {
            block_id,
            confirmed: 240,
            producer: "gorillapower".to_string(),
            previous: "00002a304f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda".to_string(),
            block_num: 1337,
            timestamp: "2018-11-23T17:55:06.000".to_string(),
            action_mroot: "ff146c3b50187542da35111cc9057031d1d5a6961110725cc4409e0895de572b".to_string(),
            schedule_version: 2,
            transaction_mroot: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            producer_signature: "SIG_K1_KAYaAyqWGxo38cxuNexehkqQEghJY5iekGj56A1v7c8Qs61v4rLgH3cFdqpQ6rLzeNcAb1xZVXsNfayiHuQKzbyC2Kr36Y".to_string(),
            header_extension: None,
            new_producers: None,
        };
        let result = parse_eos_block_header_from_json(&json)
            .unwrap();
        let expected_serialized = "f4f615477015a7d5c4e82e65f00000002a304f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda0000000000000000000000000000000000000000000000000000000000000000ff146c3b50187542da35111cc9057031d1d5a6961110725cc4409e0895de572b020000000000";
        let result_serialized = hex::encode(result.serialize().unwrap());
        assert_eq!(result.id().unwrap(), expected_block_id);
        assert_eq!(result_serialized, expected_serialized);
    }

    #[test]
    fn should_parse_block_header_from_json_4() {
        // NOTE: This block === https://jungle.bloks.io/block/75230993
        let block_id =
            "047bef11966be96d0898f76a951637367e83eb13de5f8a9e3770c5c8a32e736f"
            .to_string();
        let expected_block_id = convert_hex_to_checksum256(&block_id)
            .unwrap();
        let json = EosBlockHeaderJson {
            block_id,
            confirmed: 0,
            producer: "jungleswedeo".to_string(),
            previous: "047bef1059cd1da401e09bda1617bc2b58c6dfdb11d7f05db14c55f442d036ad".to_string(),
            block_num: 1337,
            timestamp: "2020-02-11T09:17:41.500".to_string(),
            action_mroot: "74ef05af4a312a8f010e3e442f3097dc33ec4b22738504ab38d8e30724f24d4b".to_string(),
            schedule_version: 379,
            transaction_mroot: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            producer_signature: "SIG_K1_K8S9NPR8Xv8hyi7EWT6fjy4iBYtt3F6PPxv5S5H2a9rucP8YxtZUmxeyxxsxg6HHNeNQ4JJTRKCzdqdN3drRFWDi9KJduL".to_string(),
            header_extension: None,
            new_producers: None,
        };
        let result = parse_eos_block_header_from_json(&json)
            .unwrap();
        let expected_serialized = "6b5baa4b4055521cabc8a67e0000047bef1059cd1da401e09bda1617bc2b58c6dfdb11d7f05db14c55f442d036ad000000000000000000000000000000000000000000000000000000000000000074ef05af4a312a8f010e3e442f3097dc33ec4b22738504ab38d8e30724f24d4b7b0100000000";
        let result_serialized = hex::encode(result.serialize().unwrap());
        assert_eq!(result.id().unwrap(), expected_block_id);
        assert_eq!(result_serialized, expected_serialized);
    }

    // TODO Need a block with something in the new_producers field.

    #[test]
    fn should_parse_submisson_material_with_action_proofs() {
        let material = get_sample_eos_submission_material_string_n(2)
            .unwrap();
        if let Err(e) =  parse_eos_submission_material_string_to_struct(
            &material
        ) {
            panic!("Error parsing submission material: {}", e);
        }
    }
}
