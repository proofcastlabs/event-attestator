use derive_more::{Constructor, Deref, DerefMut};

#[derive(Clone, Debug, Deref, DerefMut, Constructor)]
pub struct EthSignature(pub [u8; 65]);
