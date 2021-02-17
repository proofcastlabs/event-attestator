use eos_primitives::{bitutil, AccountName, BlockTimestamp, Checksum256, NumBytes, Read, Write};

use crate::chains::eos::{
    eos_extension::EosExtension,
    eos_producer_schedule::{EosProducerScheduleV1, EosProducerScheduleV2},
};
#[derive(Debug, Clone, Default, Read, Write, NumBytes, PartialEq, Deserialize, Serialize)]
#[eosio_core_root_path = "eos_primitives"]
pub struct EosBlockHeaderV1 {
    pub timestamp: BlockTimestamp,
    pub producer: AccountName,
    pub confirmed: u16,
    pub previous: Checksum256,
    pub transaction_mroot: Checksum256,
    pub action_mroot: Checksum256,
    pub schedule_version: u32,
    pub new_producers: Option<EosProducerScheduleV1>,
    pub header_extensions: Vec<EosExtension>,
}

impl core::fmt::Display for EosBlockHeaderV1 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "block_num: {}\n\
            id: {}\n\
            timestamp: {}\n\
            producer: {}\n\
            confirmed: {}\n\
            previous: {}\n\
            transaction_mroot: {}\n\
            action_mroot: {}\n\
            schedule_version: {}\n\
            new_producers: {:?}\n\
            header_extensions: {:?}",
            self.block_num(),
            self.id().unwrap(),
            self.timestamp,
            self.producer,
            self.confirmed,
            self.previous,
            self.transaction_mroot,
            self.action_mroot,
            self.schedule_version,
            self.new_producers,
            self.header_extensions,
        )
    }
}

impl EosBlockHeaderV1 {
    pub fn new(
        timestamp: BlockTimestamp,
        producer: AccountName,
        confirmed: u16,
        previous: Checksum256,
        transaction_mroot: Checksum256,
        action_mroot: Checksum256,
        schedule_version: u32,
        new_producers: Option<EosProducerScheduleV1>,
        header_extensions: Vec<EosExtension>,
    ) -> Self {
        Self {
            timestamp,
            producer,
            confirmed,
            previous,
            transaction_mroot,
            action_mroot,
            schedule_version,
            new_producers,
            header_extensions,
        }
    }

    pub fn digest(&self) -> crate::Result<Checksum256> {
        Ok(Checksum256::hash(self.clone())?)
    }

    pub fn id(&self) -> crate::Result<Checksum256> {
        let mut result = self.digest()?;
        let mut hash0 = result.hash0();
        hash0 &= 0xffffffff00000000;
        hash0 += bitutil::endian_reverse_u32(self.block_num()) as u64;
        result.set_hash0(hash0);
        Ok(result)
    }

    pub fn block_num(&self) -> u32 {
        Self::num_from_id(self.previous) + 1
    }

    pub fn num_from_id(id: Checksum256) -> u32 {
        bitutil::endian_reverse_u32(id.hash0() as u32)
    }
}

#[derive(Debug, Clone, Default, Read, Write, NumBytes, PartialEq, Deserialize, Serialize)]
#[eosio_core_root_path = "eos_primitives"]
pub struct EosBlockHeaderV2 {
    pub timestamp: BlockTimestamp,
    pub producer: AccountName,
    pub confirmed: u16,
    pub previous: Checksum256,
    pub transaction_mroot: Checksum256,
    pub action_mroot: Checksum256,
    pub schedule_version: u32,
    pub new_producer_schedule: Option<EosProducerScheduleV2>,
    pub header_extensions: Vec<EosExtension>,
}

impl core::fmt::Display for EosBlockHeaderV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "block_num: {}\n\
            id: {}\n\
            timestamp: {}\n\
            producer: {}\n\
            confirmed: {}\n\
            previous: {}\n\
            transaction_mroot: {}\n\
            action_mroot: {}\n\
            schedule_version: {}\n\
            new_producers: {:?}\n\
            header_extensions: {:?}",
            self.block_num(),
            self.id().unwrap(),
            self.timestamp,
            self.producer,
            self.confirmed,
            self.previous,
            self.transaction_mroot,
            self.action_mroot,
            self.schedule_version,
            self.new_producer_schedule,
            self.header_extensions,
        )
    }
}

impl EosBlockHeaderV2 {
    pub fn new(
        timestamp: BlockTimestamp,
        producer: AccountName,
        confirmed: u16,
        previous: Checksum256,
        transaction_mroot: Checksum256,
        action_mroot: Checksum256,
        schedule_version: u32,
        new_producer_schedule: Option<EosProducerScheduleV2>,
        header_extensions: Vec<EosExtension>,
    ) -> Self {
        Self {
            timestamp,
            producer,
            confirmed,
            previous,
            transaction_mroot,
            action_mroot,
            schedule_version,
            new_producer_schedule,
            header_extensions,
        }
    }

    pub fn digest(&self) -> crate::Result<Checksum256> {
        Ok(Checksum256::hash(self.clone())?)
    }

    pub fn id(&self) -> crate::Result<Checksum256> {
        let mut result = self.digest()?;
        let mut hash0 = result.hash0();
        hash0 &= 0xffffffff00000000;
        hash0 += bitutil::endian_reverse_u32(self.block_num()) as u64;
        result.set_hash0(hash0);
        Ok(result)
    }

    pub fn block_num(&self) -> u32 {
        Self::num_from_id(self.previous) + 1
    }

    pub fn num_from_id(id: Checksum256) -> u32 {
        bitutil::endian_reverse_u32(id.hash0() as u32)
    }
}
