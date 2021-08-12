use strum_macros::EnumIter;

use crate::types::Byte;

#[derive(Clone, Debug, EnumIter, Eq, PartialEq)]
pub enum EthReceiptType {
    Legacy,
    EIP2718,
}

impl EthReceiptType {
    pub fn from_byte(byte: &Byte) -> Self {
        match byte {
            0x00 => Self::Legacy,
            0x02 => Self::EIP2718,
            _ => Self::Legacy,
        }
    }

    pub fn to_byte(&self) -> Byte {
        match self {
            Self::Legacy => 0x00,
            Self::EIP2718 => 0x02,
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::types::Bytes;

    #[test]
    fn should_make_receipt_types_byte_roundtrip() {
        let expected_results = EthReceiptType::iter().collect::<Vec<EthReceiptType>>();
        let bytes = EthReceiptType::iter()
            .map(|receipt_type| receipt_type.to_byte())
            .collect::<Bytes>();
        let results = bytes
            .iter()
            .map(|ref byte| EthReceiptType::from_byte(byte))
            .collect::<Vec<EthReceiptType>>();
        assert_eq!(results, expected_results);
    }
}
