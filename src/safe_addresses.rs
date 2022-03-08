use std::str::FromStr;

use bitcoin::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;

use crate::chains::eth::eth_utils::convert_hex_to_eth_address;

pub const SAFE_EOS_ADDRESS_STR: &str = "safu.ptokens";
pub const SAFE_BTC_ADDRESS_STR: &str = "136CTERaocm8dLbEtzCaFtJJX9jfFhnChK";
pub const SAFE_ETH_ADDRESS_HEX: &str = "0x71A440EE9Fa7F99FB9a697e96eC7839B8A1643B8";
pub const SAFE_EVM_ADDRESS_HEX: &str = SAFE_ETH_ADDRESS_HEX;

lazy_static! {
    pub static ref SAFE_EOS_ADDRESS: EosAddress = EosAddress::from_str(SAFE_EOS_ADDRESS_STR).unwrap();
    pub static ref SAFE_BTC_ADDRESS: BtcAddress = BtcAddress::from_str(SAFE_BTC_ADDRESS_STR).unwrap();
    pub static ref SAFE_ETH_ADDRESS: EthAddress = convert_hex_to_eth_address(SAFE_ETH_ADDRESS_HEX).unwrap();
    pub static ref SAFE_EVM_ADDRESS: EthAddress = convert_hex_to_eth_address(SAFE_EVM_ADDRESS_HEX).unwrap();
}

pub fn safely_convert_str_to_eth_address(s: &str) -> EthAddress {
    info!("✔ Safely converting str to ETH address...");
    match convert_hex_to_eth_address(s) {
        Ok(address) => address,
        Err(_) => {
            info!("✘ '{s}' is not a valid ETH address - defaulting to safe ETH address!");
            *SAFE_ETH_ADDRESS
        },
    }
}

pub fn safely_convert_str_to_btc_address(s: &str) -> BtcAddress {
    info!("✔ Safely converting str to BTC address...");
    match BtcAddress::from_str(s) {
        Ok(address) => address,
        Err(_) => {
            info!("✘ '{s}' is not a valid BTC address - defaulting to safe BTC address!");
            SAFE_BTC_ADDRESS.clone()
        },
    }
}

pub fn safely_convert_str_to_eos_address(s: &str) -> EosAddress {
    info!("✔ Safely converting str to EOS address...");
    match EosAddress::from_str(s) {
        Ok(address) => address,
        Err(_) => {
            info!("✘ '{s}' is not a valid EOS address - defaulting to safe EOS address!");
            *SAFE_EOS_ADDRESS
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_safely_convert_str_to_eth_address() {
        let s = "0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8";
        let expected_result = convert_hex_to_eth_address(s).unwrap();
        let result = safely_convert_str_to_eth_address(s);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_default_to_safe_address_if_eth_address_malformed() {
        let s = "not an good adddress";
        let expected_result = *SAFE_ETH_ADDRESS;
        let result = safely_convert_str_to_eth_address(s);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_safely_convert_str_to_btc_address() {
        let s = "bc1qwqdg6squsna38e46795at95yu9atm8azzmyvckulcc7kytlcckxswvvzej";
        let expected_result = BtcAddress::from_str(s).unwrap();
        let result = safely_convert_str_to_btc_address(s);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_default_to_safe_address_if_btc_address_malformed() {
        let s = "not an good adddress";
        let expected_result = SAFE_BTC_ADDRESS.clone();
        let result = safely_convert_str_to_btc_address(s);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_safely_convert_str_to_eos_address() {
        let s = "safu.ptokens";
        let expected_result = EosAddress::from_str(s).unwrap();
        let result = safely_convert_str_to_eos_address(s);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_default_to_safe_address_if_eos_address_malformed() {
        let s = "not an good adddress";
        let expected_result = SAFE_EOS_ADDRESS.clone();
        let result = safely_convert_str_to_eos_address(s);
        assert_eq!(result, expected_result);
    }
}
