use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, H256};
use web3::signing::recover;

use crate::types::Result;

#[derive(Clone, Debug, Deref, DerefMut, Constructor)]
pub struct EthSignature(pub [u8; 65]);

impl EthSignature {
    pub fn set_recovery_param(self) -> Self {
        // NOTE: Eth recovery params are different. See here for more info:
        // https://bitcoin.stackexchange.com/questions/38351/ecdsa-v-r-s-what-is-v
        let mut mutable_self = self.clone();
        mutable_self[64] = if mutable_self[64] == 1 { 0x1c } else { 0x1b };
        mutable_self
    }

    fn get_ecdsa_recovery_param(&self) -> u8 {
        match self[64] {
            0x1c => 1,
            _ => 0,
        }
    }

    pub fn recover_signer_address(&self, hash: &H256) -> Result<EthAddress> {
        Ok(recover(
            hash.as_bytes(),
            &self[..64],
            self.get_ecdsa_recovery_param().into(),
        )?)
    }
}
