use derive_more::{Constructor, Deref, DerefMut};

#[derive(Clone, Debug, Deref, DerefMut, Constructor)]
pub struct EthSignature(pub [u8; 65]);

impl EthSignature {
    pub fn set_recovery_param(self) -> Self {
        let mut mutable_self = self.clone();
        mutable_self[64] = if mutable_self[64] == 1 { 0x1c } else { 0x1b };
        mutable_self
    }
}
