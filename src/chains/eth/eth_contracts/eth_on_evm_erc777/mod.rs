use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::{
        eth_constants::{ETH_ADDRESS_SIZE_IN_BYTES, ETH_WORD_SIZE_IN_BYTES},
        eth_contracts::encode_fxn_call,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::{
            get_erc777_contract_address_from_db,
            get_eth_account_nonce_from_db,
            get_eth_chain_id_from_db,
            get_eth_gas_price_from_db,
            get_eth_private_key_from_db,
            increment_eth_account_nonce_in_db,
        },
        eth_log::EthLog,
        eth_traits::EthLogCompatible,
    },
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};

pub const EMPTY_DATA: Bytes = vec![];
pub const ETH_ON_EVM_ERC777_CHANGE_PNETWORK_GAS_LIMIT: usize = 30_000;
pub const ETH_ON_EVM_ERC777_MINT_WITH_DATA_GAS_LIMIT: usize = 300_000;
pub const ETH_ON_EVM_ERC777_MINT_WITH_NO_DATA_GAS_LIMIT: usize = 180_000;

pub const ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"newPNetwork\",\"type\":\"address\"}],\"name\":\"changePNetwork\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"signature\":\"0xfd4add66\"},{\"constant\":false,\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"constant\":false,\"inputs\":[{\"name\":\"recipient\",\"type\":\"address\"},{\"name\":\"value\",\"type\":\"uint256\"},{\"name\":\"userData\",\"type\":\"bytes\"},{\"name\":\"operatorData\",\"type\":\"bytes\"}],\"name\":\"mint\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]";

lazy_static! {
    pub static ref ERC_777_REDEEM_EVENT_TOPIC: EthHash = {
        EthHash::from_slice(
            &hex::decode("78e6c3f67f57c26578f2487b930b70d844bcc8dd8f4d629fb4af81252ab5aa65")
                .expect("✘ Invalid hex in `BTC_ON_ETH_REDEEM_EVENT_TOPIC`"),
        )
    };
}

pub fn encode_erc777_change_pnetwork_fxn_data(new_ptoken_address: EthAddress) -> Result<Bytes> {
    encode_fxn_call(ABI, "changePNetwork", &[EthAbiToken::Address(new_ptoken_address)])
}

pub fn encode_erc777_mint_with_no_data_fxn(recipient: &EthAddress, value: &U256) -> Result<Bytes> {
    encode_fxn_call(ABI, "mint", &[
        EthAbiToken::Address(*recipient),
        EthAbiToken::Uint(*value),
    ])
}

fn encode_erc777_mint_with_data_fxn(
    recipient: &EthAddress,
    value: &U256,
    user_data: &[Byte],
    operator_data: &[Byte],
) -> Result<Bytes> {
    encode_fxn_call(ABI, "mint", &[
        EthAbiToken::Address(*recipient),
        EthAbiToken::Uint(*value),
        EthAbiToken::Bytes(operator_data.to_vec()),
        EthAbiToken::Bytes(user_data.to_vec()),
    ])
}

fn get_eth_calldata_from_maybe_data(maybe_data: Option<&[Byte]>) -> Bytes {
    maybe_data.unwrap_or(&EMPTY_DATA).to_vec()
}

pub fn encode_erc777_mint_fxn_maybe_with_data(
    recipient: &EthAddress,
    value: &U256,
    user_data: Option<&[Byte]>,
    operator_data: Option<&[Byte]>,
) -> Result<Bytes> {
    match user_data.is_some() | operator_data.is_some() {
        false => encode_erc777_mint_with_no_data_fxn(recipient, value),
        true => encode_erc777_mint_with_data_fxn(
            recipient,
            value,
            &get_eth_calldata_from_maybe_data(user_data),
            &get_eth_calldata_from_maybe_data(operator_data),
        ),
    }
}

pub fn get_signed_erc777_change_pnetwork_tx<D: DatabaseInterface>(db: &D, new_address: EthAddress) -> Result<String> {
    const ZERO_ETH_VALUE: usize = 0;
    let nonce_before_incrementing = get_eth_account_nonce_from_db(db)?;
    increment_eth_account_nonce_in_db(db, 1).and(Ok(EthTransaction::new_unsigned(
        encode_erc777_change_pnetwork_fxn_data(new_address)?,
        nonce_before_incrementing,
        ZERO_ETH_VALUE,
        get_erc777_contract_address_from_db(db)?,
        get_eth_chain_id_from_db(db)?,
        ETH_ON_EVM_ERC777_CHANGE_PNETWORK_GAS_LIMIT,
        get_eth_gas_price_from_db(db)?,
    )
    .sign(&get_eth_private_key_from_db(db)?)?
    .serialize_hex()))
}

#[derive(Debug, Clone, Constructor, Eq, PartialEq)]
pub struct EthOnEvmErc777RedeemEvent {
    pub value: U256,
    pub redeemer: EthAddress,
    pub underlying_asset_recipient: EthAddress,
    pub user_data: Bytes,
}

impl EthOnEvmErc777RedeemEvent {
    fn get_err_msg(field: &str) -> String {
        format!("Error getting `{}` from `EthOnEvmErc777RedeemEvent`!", field)
    }

    pub fn from_log<T: EthLogCompatible>(log: &T) -> Result<Self> {
        let tokens = eth_abi_decode(
            &vec![
                EthAbiParamType::Uint(256),
                EthAbiParamType::Address,
                EthAbiParamType::Bytes,
            ],
            &log.get_data(),
        )?;
        Ok(Self {
            redeemer: EthAddress::from_slice(
                &log.get_topics()[1][ETH_WORD_SIZE_IN_BYTES - ETH_ADDRESS_SIZE_IN_BYTES..],
            ),
            value: match tokens[0] {
                EthAbiToken::Uint(value) => Ok(value),
                _ => Err(Self::get_err_msg("value").to_string()),
            }?,
            underlying_asset_recipient: match tokens[1] {
                EthAbiToken::Address(value) => Ok(value),
                _ => Err(Self::get_err_msg("underlying_asset_recipient").to_string()),
            }?,
            user_data: match tokens[2] {
                EthAbiToken::Bytes(ref value) => Ok(value.clone()),
                _ => Err(Self::get_err_msg("user_data").to_string()),
            }?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_test_utils::get_sample_log_with_eth_on_evm_erc777_peg_out;

    #[test]
    fn should_decode_eth_on_evm_erc777_event_correctly() {
        let log = get_sample_log_with_eth_on_evm_erc777_peg_out();
        let result = EthOnEvmErc777RedeemEvent::from_log(&log).unwrap();
        let expected_result = EthOnEvmErc777RedeemEvent {
            value: U256::from_dec_str("666").unwrap(),
            redeemer: EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap()),
            underlying_asset_recipient: EthAddress::from_slice(
                &hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap(),
            ),
            user_data: vec![0xde, 0xca, 0xff],
        };
        assert_eq!(result, expected_result);
    }
}
