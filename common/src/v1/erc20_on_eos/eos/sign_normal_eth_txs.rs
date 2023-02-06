use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_constants::ZERO_ETH_VALUE,
        eth_contracts::erc20_vault::encode_erc20_vault_peg_out_fxn_data_without_user_data,
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::{EthTransaction, EthTransactions},
        },
        eth_database_utils::EthDbUtilsExt,
    },
    erc20_on_eos::eos::eth_tx_info::Erc20OnEosEthTxInfos,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_eth_signed_txs(
    eth_tx_infos: &Erc20OnEosEthTxInfos,
    erc20_on_eos_smart_contract_address: &EthAddress,
    eth_account_nonce: u64,
    chain_id: &EthChainId,
    gas_price: u64,
    eth_private_key: &EthPrivateKey,
) -> Result<EthTransactions> {
    info!("✔ Getting ETH signed transactions from `erc20-on-eos` redeem infos...");
    Ok(EthTransactions::new(
        eth_tx_infos
            .iter()
            .enumerate()
            .map(|(i, eth_tx_info)| {
                info!(
                    "✔ Signing ETH tx for amount: {}, to address: {}",
                    eth_tx_info.amount, eth_tx_info.destination_address
                );
                EthTransaction::new_unsigned(
                    encode_erc20_vault_peg_out_fxn_data_without_user_data(
                        eth_tx_info.destination_address,
                        eth_tx_info.eth_token_address,
                        eth_tx_info.amount,
                    )?,
                    eth_account_nonce + i as u64,
                    ZERO_ETH_VALUE,
                    *erc20_on_eos_smart_contract_address,
                    chain_id,
                    chain_id.get_erc20_vault_pegout_without_user_data_gas_limit(),
                    gas_price,
                )
                .sign(eth_private_key)
            })
            .collect::<Result<Vec<EthTransaction>>>()?,
    ))
}

pub fn maybe_sign_normal_eth_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ No redeem infos in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        Erc20OnEosEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                get_eth_signed_txs(
                    &infos,
                    &state.eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
                    state.eth_db_utils.get_eth_account_nonce_from_db()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    state.eth_db_utils.get_eth_gas_price_from_db()?,
                    &state.eth_db_utils.get_eth_private_key_from_db()?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_eth_signed_txs(signed_txs)
            })
    }
}
