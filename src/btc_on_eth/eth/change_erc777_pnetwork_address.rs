use ethabi::Token;
use ethereum_types::Address as EthAddress;
use crate::{
    errors::AppError,
    traits::DatabaseInterface,
    chains::eth::eth_contracts::get_contract::instantiate_contract_from_abi,
    btc_on_eth::eth::{
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::EthTransaction,
        },
        eth_database_utils::{
            get_eth_chain_id_from_db,
            get_eth_gas_price_from_db,
            get_eth_private_key_from_db,
            get_eth_account_nonce_from_db,
            increment_eth_account_nonce_in_db,
            get_eth_smart_contract_address_from_db, // TODO rename to get_erc777...etc
        },
    },
    types::{
        Bytes,
        Result,
    },
};

const ERC777_PNETWORK_CHANGE_GAS_LIMIT: usize = 30_000; // TODO finesse!
const CHANGE_PNETWORK_FXN_NAME: &str = "changePNetwork";
pub const CHANGE_PNETWORK_ABI: &str = "[{\"constant\":false,\"inputs\":[{\"name\":\"newPNetwork\",\"type\":\"address\"}],\"name\":\"changePNetwork\",\"outputs\":[],\"payable\":false,\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"signature\":\"0xfd4add66\"}]";


fn encode_erc777_change_pnetwork_fxn_data(new_ptoken_address: EthAddress) -> Result<Bytes> {
    instantiate_contract_from_abi(CHANGE_PNETWORK_ABI)
        .and_then(|contract|
            match contract.function(CHANGE_PNETWORK_FXN_NAME) {
                Err(e) => Err(AppError::Custom(
                    format!("✘ Error getting `{}` from contract: {}", CHANGE_PNETWORK_FXN_NAME, e)
                )),
                Ok(function) => match function.encode_input(&[Token::Address(new_ptoken_address)]) {
                    Err(e) => Err(AppError::Custom(format!("✘ Error encoding input: {}", e))),
                    Ok(bytes) => Ok(bytes),
                }
            }
        )
}

pub fn get_signed_change_erc777_pnetwork_tx<D>(
    db: &D,
    new_ptoken_address: EthAddress
) -> Result<String>
    where D: DatabaseInterface
{
    let nonce = get_eth_account_nonce_from_db(db)?;
    increment_eth_account_nonce_in_db(db, 1)
        .and_then(|_|
            Ok(
                EthTransaction::new(
                    encode_erc777_change_pnetwork_fxn_data(new_ptoken_address)?,
                    nonce,
                    0,
                    get_eth_smart_contract_address_from_db(db)?,
                    get_eth_chain_id_from_db(db)?,
                    ERC777_PNETWORK_CHANGE_GAS_LIMIT,
                    get_eth_gas_price_from_db(db)?,
                )
                    .sign(get_eth_private_key_from_db(db)?)?
                    .serialize_hex()
            )
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_erc777_change_pnetwork_fxn_data() {
        let expected_result = "fd4add66000000000000000000000000736661736533bcfc9cc35649e6324acefb7d32c1";
        let address = EthAddress::from_slice(&hex::decode("736661736533BcfC9cc35649e6324aceFb7D32c1").unwrap());
        let result = encode_erc777_change_pnetwork_fxn_data(address).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }
}
