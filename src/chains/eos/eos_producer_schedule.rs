use core::default::Default;

use eos_chain::{Checksum256, NumBytes, Read, Write};
use serde::{Deserialize, Serialize};

use crate::chains::eos::{
    eos_producer_key::{EosKey, EosKeysAndThreshold, EosProducerKeyV1, EosProducerKeyV2},
    parse_eos_schedule::{
        convert_v1_producer_key_jsons_to_v1_producer_keys,
        convert_v2_producer_key_jsons_to_v2_producer_keys,
        EosProducerScheduleJsonV1,
        EosProducerScheduleJsonV2,
    },
};

#[derive(Read, Write, NumBytes, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[eosio_core_root_path = "eos_chain"]
#[repr(C)]
pub struct EosProducerScheduleV2 {
    pub version: u32,
    pub producers: Vec<EosProducerKeyV2>,
}

impl EosProducerScheduleV2 {
    pub fn schedule_hash(&self) -> crate::Result<Checksum256> {
        Ok(Checksum256::hash(self.clone())?)
    }

    /// # Maybe deserialize `EosProducerScheduleV2` from JSON string
    ///
    /// This function deserializes `EosProducerScheduleV2` from JSON string.
    /// It also accepts JSON representation of `EosProducerScheduleV1`
    /// and implicitly converts it into `EosProducerScheduleV2`.
    pub fn from_json(json_string: &str) -> crate::Result<Self> {
        EosProducerScheduleJsonV2::from(json_string)
            .and_then(|json| Self::from_schedule_json(&json))
            .or_else(|_| EosProducerScheduleV1::from_json(json_string).map(EosProducerScheduleV2::from))
    }

    pub fn from_schedule_json(json: &EosProducerScheduleJsonV2) -> crate::Result<Self> {
        Ok(Self {
            version: json.version,
            producers: convert_v2_producer_key_jsons_to_v2_producer_keys(&json.producers)?,
        })
    }
}

impl Default for EosProducerScheduleV2 {
    fn default() -> Self {
        Self {
            version: 0,
            producers: vec![],
        }
    }
}

impl From<EosProducerScheduleV1> for EosProducerScheduleV2 {
    fn from(v1_schedule: EosProducerScheduleV1) -> Self {
        Self {
            version: v1_schedule.version,
            producers: v1_schedule
                .producers
                .iter()
                .map(|producer| EosProducerKeyV2 {
                    producer_name: producer.producer_name,
                    authority: (0, EosKeysAndThreshold {
                        threshold: 0,
                        keys: vec![EosKey {
                            weight: 0,
                            key: producer.block_signing_key.clone(),
                        }],
                    }),
                })
                .collect::<Vec<EosProducerKeyV2>>(),
        }
    }
}

#[derive(Deserialize, Serialize, Read, Write, NumBytes, Clone, Default, Debug, PartialEq)]
#[eosio_core_root_path = "eos_chain"]
pub struct EosProducerScheduleV1 {
    pub version: u32,
    pub producers: Vec<EosProducerKeyV1>,
}

impl EosProducerScheduleV1 {
    pub fn new(version: u32, producers: Vec<EosProducerKeyV1>) -> Self {
        Self { version, producers }
    }

    pub fn schedule_hash(&self) -> crate::Result<Checksum256> {
        Ok(Checksum256::hash(self.clone())?)
    }

    pub fn from_json(json_string: &str) -> crate::Result<Self> {
        EosProducerScheduleJsonV1::from(json_string).and_then(|json| Self::from_schedule_json(&json))
    }

    pub fn from_schedule_json(json: &EosProducerScheduleJsonV1) -> crate::Result<Self> {
        Ok(Self {
            version: json.version,
            producers: convert_v1_producer_key_jsons_to_v1_producer_keys(&json.producers)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eos::eos_test_utils::{
        get_sample_v1_schedule,
        get_sample_v1_schedule_json_string,
        get_sample_v2_schedule_json,
        get_sample_v2_schedule_json_string,
    };

    #[test]
    fn should_parse_v2_schedule_string_to_v2_schedule() {
        let schedule_string = get_sample_v2_schedule_json_string().unwrap();
        if let Err(e) = EosProducerScheduleV2::from_json(&schedule_string) {
            panic!("Error parseing schedule: {}", e);
        }
    }

    #[test]
    fn should_parse_v1_schedule_string_to_v2_schedule() {
        let schedule_string = get_sample_v1_schedule_json_string().unwrap();
        if let Err(e) = EosProducerScheduleV2::from_json(&schedule_string) {
            panic!("Error parseing schedule: {}", e);
        }
    }

    #[test]
    fn should_convert_v2_schedule_json_to_v2_schedule() {
        let schedule_json = get_sample_v2_schedule_json().unwrap();
        if let Err(e) = EosProducerScheduleV2::from_schedule_json(&schedule_json) {
            panic!("Error converting producer key json: {}", e);
        }
    }

    #[test]
    fn should_convert_v1_schedule_to_v2() {
        let v1_schedule = get_sample_v1_schedule().unwrap();
        EosProducerScheduleV2::from(v1_schedule);
    }
}
