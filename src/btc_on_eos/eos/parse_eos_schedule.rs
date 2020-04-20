use std::str::FromStr;
use crate::{
    types::Result,
    errors::AppError,
};
use eos_primitives::{
    Key as EosKey,
    PublicKey as EosPublicKey,
    AccountName as EosAccountName,
    ProducerKeyV2 as EosProducerKeyV2,
    KeysAndThreshold as EosKeysAndThreshold,
    ProducerScheduleV2 as EosProducerScheduleV2,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EosProducerScheduleJson {
    pub version: u32,
    pub producers: Vec<FullProducerKeyJson>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FullProducerKeyJson {
    pub producer_name: String,
    pub authority: (u32, AuthorityJson),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityJson {
    pub threshold: u32,
    pub keys: Vec<ProducerKeyJson>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProducerKeyJson {
    weight: u16,
    key: String,
}

fn convert_full_producer_key_jsons_to_producer_keys_v2(
    json: &Vec<FullProducerKeyJson>,
) -> Result<Vec<EosProducerKeyV2>> {
    json
        .iter()
        .map(convert_full_producer_key_json_to_producer_key_v2)
        .collect()
}

fn convert_full_producer_key_json_to_producer_key_v2(
    json: &FullProducerKeyJson,
) -> Result<EosProducerKeyV2> {
    Ok(
        EosProducerKeyV2 {
            producer_name: EosAccountName::from_str(&json.producer_name)?,
            authority: (
                json.authority.0,
                convert_authority_json_to_eos_keys_and_threshold(
                    &json.authority.1
                )?
            )
        }
    )
}

fn convert_authority_json_to_eos_keys_and_threshold(
    json: &AuthorityJson,
) -> Result<EosKeysAndThreshold> {
    Ok(
        EosKeysAndThreshold {
            threshold: json.threshold,
            keys: convert_keys_json_to_vec_of_eos_keys(&json.keys)?,
        }
    )
}

pub fn convert_keys_json_to_vec_of_eos_keys(
    keys_json: &Vec<ProducerKeyJson>,
) -> Result<Vec<EosKey>> {
    keys_json
        .iter()
        .map(convert_key_json_to_eos_key)
        .collect()
}

pub fn convert_key_json_to_eos_key(
    key_json: &ProducerKeyJson,
) -> Result<EosKey> {
    Ok(
        EosKey {
            weight: key_json.weight,
            key: EosPublicKey::from_str(&key_json.key)?,
        }
    )
}

pub fn parse_schedule_string_to_json(
    schedule_string: &String
) -> Result<EosProducerScheduleJson> {
    match serde_json::from_str(schedule_string) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

pub fn convert_schedule_json_to_schedule_v2(
    json: &EosProducerScheduleJson,
) -> Result<EosProducerScheduleV2> {
    Ok(
        EosProducerScheduleV2 {
            version: json.version,
            producers: convert_full_producer_key_jsons_to_producer_keys_v2(
                &json.producers
            )?,
        }
    )
}

pub fn parse_schedule_string_to_schedule(
    schedule_string: &String
) -> Result<EosProducerScheduleV2> {
    parse_schedule_string_to_json(schedule_string)
        .and_then(|json| convert_schedule_json_to_schedule_v2(&json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::eos::eos_test_utils::{
        get_sample_v2_schedule_json,
        get_sample_v2_schedule_json_string,
    };

    #[test]
    fn should_parse_v2_schedule_to_json() {
        let schedule_string = get_sample_v2_schedule_json_string()
            .unwrap();
        if let Err(e) = parse_schedule_string_to_json(&schedule_string) {
            panic!("Could not parse EOS schedule json: {}", e);
        }
    }

    #[test]
    fn should_convert_full_producer_key_json_to_producer_key_v2() {
        let producer_key_json = get_sample_v2_schedule_json()
            .unwrap()
            .producers[0]
            .clone();
        if let Err(e) = convert_full_producer_key_json_to_producer_key_v2(
            &producer_key_json
        ) {
            panic!("Error converting producer key json: {}", e);
        }
    }

    #[test]
    fn should_convert_schedule_json_to_schedule_v2() {
        let schedule_json = get_sample_v2_schedule_json()
            .unwrap();
        if let Err(e) = convert_schedule_json_to_schedule_v2(&schedule_json) {
            panic!("Error converting producer key json: {}", e);
        }
    }

    #[test]
    fn should_parse_schedule_string_to_schedule() {
        let schedule_string = get_sample_v2_schedule_json_string()
            .unwrap();
        if let Err(e) = parse_schedule_string_to_schedule(&schedule_string) {
            panic!("Error parseing schedule: {}", e);
        }
    }
}
