use crate::{
    types::Result,
    errors::AppError,
};
use std::{
    str::FromStr,
    fs::read_to_string,
};
use eos_primitives::{
    PublicKey,
    AccountName as EosAccountName,
    ProducerKey as EosProducerKey,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EosScheduleJson {
    pub version: u32,
    pub producers: Vec<FullProducerKeyJson>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FullProducerKeyJson {
    pub producer_name: String,
    pub authority: (u32, AuthorityJson),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityJson
{

    pub threshold: u32,
    pub keys: Vec<ProducerKeyJson>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProducerKeyJson {
    weight: u16,
    key: String,
}

pub fn parse_schedule_string_to_json(
    schedule_string: &String
) -> Result<EosScheduleJson> {
    match serde_json::from_str(schedule_string) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_sample_v2_schedule_json_string() -> Result<String> {
        Ok(
            read_to_string(
                "src/btc_on_eos/eos/eos_test_utils/sample-schedule-v2.0.json"
            )?
        )
    }

    #[test]
    fn should_parse_v2_schedule_to_json() {
        let schedule_string = get_sample_v2_schedule_json_string()
            .unwrap();
        if let Err(e) = parse_schedule_string_to_json(&schedule_string) {
            panic!("Could not parse EOS schedule json: {}", e);
        }
    }
}
