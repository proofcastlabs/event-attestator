use ethereum_types::Address as EthAddress;

use crate::{chains::eth::eth_utils::convert_hex_to_eth_address, constants::SAFE_ETH_ADDRESS, types::Result};

pub fn safely_convert_str_to_eth_address(s: &str) -> Result<EthAddress> {
    info!("✔ Safely converting str to  ETH address...");
    match convert_hex_to_eth_address(&s) {
        Ok(address) => Ok(address),
        Err(_) => {
            info!("✘ '{s}' is not a valid ETH address - defaulting to safe ETH address!");
            Ok(*SAFE_ETH_ADDRESS)
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
        let result = safely_convert_str_to_eth_address(s).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_default_to_safe_address_if_eth_address_malformed() {
        let s = "not an eth adddress";
        let expected_result = *SAFE_ETH_ADDRESS;
        let result = safely_convert_str_to_eth_address(s).unwrap();
        assert_eq!(result, expected_result);
    }
}
