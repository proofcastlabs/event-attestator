use core::default::Default;

use eos_primitives::{NumBytes, ProducerKeyV2, Read, Write};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[derive(Read, Write, NumBytes, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[eosio_core_root_path = "eos_primitives"]
#[repr(C)]
pub struct ProducerScheduleV2 {
    pub version: u32,
    pub producers: Vec<ProducerKeyV2>,
}

impl Default for ProducerScheduleV2 {
    fn default() -> Self {
        ProducerScheduleV2 {
            version: 0,
            producers: vec![],
        }
    }
}
