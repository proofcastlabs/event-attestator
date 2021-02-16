use std::str::FromStr;

use eos_primitives::{AccountName as EosAccountName, Asset as EosAsset, NumBytes, Read, SerializeData, Write};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[eosio_core_root_path = "::eos_primitives"]
#[derive(Clone, Debug, Default, Read, Write, NumBytes)]
pub struct PTokenMintAction {
    pub to: EosAccountName,
    pub quantity: EosAsset,
    pub memo: String,
}

impl PTokenMintAction {
    pub fn new(to: EosAccountName, quantity: EosAsset, memo: &str) -> Self {
        PTokenMintAction {
            to,
            quantity,
            memo: memo.into(),
        }
    }

    pub fn from_str(to: &str, quantity: &str, memo: &str) -> crate::Result<Self> {
        Ok(Self::new(
            EosAccountName::from_str(to)?,
            EosAsset::from_str(quantity)?,
            memo,
        ))
    }
}

impl SerializeData for PTokenMintAction {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_ptoken_mint_action_from_str() {
        let result = PTokenMintAction::from_str("whateverxxx", "1.000 EOS", "a memo");
        assert!(result.is_ok());
    }
}
