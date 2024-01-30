use std::{fmt, str::FromStr};

use common::{
    errors::AppError,
    types::{Byte, Bytes, Result},
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Clone, Debug, EnumIter, Eq, PartialEq, Serialize, Deserialize)]
pub enum EthReceiptType {
    Legacy,
    EIP2930, // NOTE: Optional access lists: https://eips.ethereum.org/EIPS/eip-2930
    EIP1559, // NOTE: New tx pricing mechanism: https://eips.ethereum.org/EIPS/eip-1559
    EIP4844, // NOTE: Dencun-fork blob-carrying txs: https://eips.ethereum.org/EIPS/eip-4844
    EIP2718, // NOTE: The original EIP introducing new tx envelopes: https://eips.ethereum.org/EIPS/eip-2718
    ArbitrumRetryTxType,
    ArbitrumLegacyTxType,
    ArbitrumDepositTxType,
    ArbitrumUnsignedTxType,
    ArbitrumContractTxType,
    ArbitrumInternalTxType,
    ArbitrumSubmitRetryableTxType,
}

impl EthReceiptType {
    pub fn from_byte(byte: &Byte) -> Self {
        match byte {
            0x00 => Self::Legacy,
            0x01 => Self::EIP2930,
            0x02 => Self::EIP1559,
            0x03 => Self::EIP4844,
            0x68 => Self::ArbitrumRetryTxType,
            0x64 => Self::ArbitrumDepositTxType,
            0x65 => Self::ArbitrumUnsignedTxType,
            0x66 => Self::ArbitrumContractTxType,
            0x78 => Self::ArbitrumLegacyTxType,
            0x6a => Self::ArbitrumInternalTxType,
            0x69 => Self::ArbitrumSubmitRetryableTxType,
            _ => Self::Legacy,
        }
    }

    pub fn to_byte(&self) -> Byte {
        match self {
            Self::Legacy => 0x00,
            Self::EIP2930 => 0x01,
            Self::EIP4844 => 0x03,
            Self::ArbitrumRetryTxType => 0x68,
            Self::ArbitrumLegacyTxType => 0x78,
            Self::ArbitrumDepositTxType => 0x64,
            Self::ArbitrumUnsignedTxType => 0x65,
            Self::ArbitrumContractTxType => 0x66,
            Self::ArbitrumInternalTxType => 0x6a,
            Self::ArbitrumSubmitRetryableTxType => 0x69,
            // NOTE: This is to remain backwards compatible where an earlier bug caused
            // some submission materail to have a type "EIP2718" when it should in fact
            // have been "0x2" or "EIP1559".
            Self::EIP1559 | Self::EIP2718 => 0x02,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        vec![self.to_byte()]
    }
}

impl fmt::Display for EthReceiptType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Legacy => "0x0",
            Self::EIP2930 => "0x1",
            Self::EIP4844 => "0x3",
            Self::ArbitrumRetryTxType => "0x68",
            Self::ArbitrumLegacyTxType => "0x78",
            Self::ArbitrumDepositTxType => "0x64",
            Self::EIP1559 | Self::EIP2718 => "0x2", // NOTE: Ibid
            Self::ArbitrumUnsignedTxType => "0x65",
            Self::ArbitrumContractTxType => "0x66",
            Self::ArbitrumInternalTxType => "0x6a",
            Self::ArbitrumSubmitRetryableTxType => "0x69",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for EthReceiptType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "0x0" | "0" => Ok(Self::Legacy),
            "EIP2930" | "0x1" | "1" => Ok(Self::EIP2930),
            "EIP4844" | "0x3" | "3" => Ok(Self::EIP4844),
            "EIP1559" | "EIP2718" | "0x2" | "2" => Ok(Self::EIP1559), // NOTE: Ibid
            "ArbitrumRetryTxType" | "0x68" | "68" => Ok(Self::ArbitrumRetryTxType),
            "ArbitrumLegacyTxType" | "0x78" | "78" => Ok(Self::ArbitrumLegacyTxType),
            "ArbitrumDepositTxType" | "0x64" | "64" => Ok(Self::ArbitrumDepositTxType),
            "ArbitrumUnsignedTxType" | "0x65" | "65" => Ok(Self::ArbitrumUnsignedTxType),
            "ArbitrumContractTxType" | "0x66" | "66" => Ok(Self::ArbitrumContractTxType),
            "ArbitrumInternalTxType" | "0x6a" | "6a" => Ok(Self::ArbitrumInternalTxType),
            "ArbitrumSubmitRetryableTxType" | "0x69" | "69" => Ok(Self::ArbitrumSubmitRetryableTxType),
            _ => Err(format!("Unrecognized ETH receipt type: {s}").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use common::types::Bytes;
    use strum::IntoEnumIterator;

    use super::*;
    use crate::EthSubmissionMaterial;

    #[test]
    fn should_make_receipt_types_byte_roundtrip() {
        let expected_results = EthReceiptType::iter()
            // NOTE: We remove the following due to bug explained in notes above
            .filter(|x| x != &EthReceiptType::EIP2718)
            .collect::<Vec<EthReceiptType>>();
        let bytes = EthReceiptType::iter()
            // NOTE: Ibid
            .filter(|x| x != &EthReceiptType::EIP2718)
            .map(|receipt_type| receipt_type.to_byte())
            .collect::<Bytes>();
        let results = bytes
            .iter()
            .map(EthReceiptType::from_byte)
            .collect::<Vec<EthReceiptType>>();
        assert_eq!(results, expected_results);
    }

    #[test]
    fn should_parse_submission_material_with_eip4844_receipt_type() {
        let p = "src/test_utils/goerli-sub-mat-with-eip-4844-receipt-type.json";
        let r = EthSubmissionMaterial::from_str(&read_to_string(p).unwrap());
        assert!(r.is_ok());
    }
}
