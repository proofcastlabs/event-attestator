use derive_more::Constructor;
use ethabi::Token as EthAbiToken;
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    chains::eth::eth_contracts::encode_fxn_call,
    types::{Bytes, Result},
};

pub const ERC20_VAULT_PEGOUT_GAS_LIMIT: usize = 180_000;
pub const ERC20_VAULT_MIGRATE_GAS_LIMIT: usize = 6_000_000;
pub const ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT: usize = 100_000;

pub const ERC20_VAULT_ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenRecipient\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"_tokenAmount\",\"type\":\"uint256\"}],\"name\":\"pegOut\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"addresspayable\",\"name\":\"_to\",\"type\":\"address\"}],\"name\":\"migrate\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"}],\"name\":\"addSupportedToken\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"SUCCESS\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_tokenAddress\",\"type\":\"address\"}],\"name\":\"removeSupportedToken\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"SUCCESS\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

pub fn encode_erc20_vault_peg_out_fxn_data(
    recipient: EthAddress,
    token_contract_address: EthAddress,
    amount: U256,
) -> Result<Bytes> {
    encode_fxn_call(ERC20_VAULT_ABI, "pegOut", &[
        EthAbiToken::Address(recipient),
        EthAbiToken::Address(token_contract_address),
        EthAbiToken::Uint(amount),
    ])
}

pub fn encode_erc20_vault_migrate_fxn_data(migrate_to: EthAddress) -> Result<Bytes> {
    encode_fxn_call(ERC20_VAULT_ABI, "migrate", &[EthAbiToken::Address(migrate_to)])
}

pub fn encode_erc20_vault_add_supported_token_fx_data(token_to_support: EthAddress) -> Result<Bytes> {
    encode_fxn_call(ERC20_VAULT_ABI, "addSupportedToken", &[EthAbiToken::Address(
        token_to_support,
    )])
}

pub fn encode_erc20_vault_remove_supported_token_fx_data(token_to_remove: EthAddress) -> Result<Bytes> {
    encode_fxn_call(ERC20_VAULT_ABI, "removeSupportedToken", &[EthAbiToken::Address(
        token_to_remove,
    )])
}

#[derive(Debug, PartialEq, Constructor)]
pub struct Erc20VaultPegInEventParams {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub token_address: EthAddress,
    pub destination_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_test_utils::get_sample_eth_address;

    #[test]
    fn should_encode_peg_out_fxn_data() {
        let amount = U256::from(1337);
        let recipient_address =
            EthAddress::from_slice(&hex::decode("edB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap());
        let token_address = EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap());
        let expected_result = "83c09d42000000000000000000000000edb86cd455ef3ca43f0e227e00469c3bdfa40628000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac0000000000000000000000000000000000000000000000000000000000000539";
        let result = hex::encode(encode_erc20_vault_peg_out_fxn_data(recipient_address, token_address, amount).unwrap());
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_encode_migrate_fxn_data() {
        let address = EthAddress::from_slice(&hex::decode("edB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap());
        let expected_result = "ce5494bb000000000000000000000000edb86cd455ef3ca43f0e227e00469c3bdfa40628";
        let result = hex::encode(encode_erc20_vault_migrate_fxn_data(address).unwrap());
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_encode_erc20_vault_add_supported_token_fx_data() {
        let expected_result = "6d69fcaf0000000000000000000000001739624f5cd969885a224da84418d12b8570d61a";
        let address = get_sample_eth_address();
        let result = encode_erc20_vault_add_supported_token_fx_data(address).unwrap();
        assert_eq!(hex::encode(&result), expected_result);
    }

    #[test]
    fn should_encode_erc20_vault_remove_supported_token_fx_data() {
        let expected_result = "763191900000000000000000000000001739624f5cd969885a224da84418d12b8570d61a";
        let address = get_sample_eth_address();
        let result = encode_erc20_vault_remove_supported_token_fx_data(address).unwrap();
        assert_eq!(hex::encode(&result), expected_result);
    }
}
