use crate::{
    chains::{
        eos::eos_state::EosState,
        eth::{
            eth_constants::ZERO_ETH_VALUE,
            eth_contracts::erc777::{encode_erc777_mint_with_no_data_fxn, ERC777_MINT_WITH_NO_DATA_GAS_LIMIT},
            eth_crypto::{eth_private_key::EthPrivateKey, eth_transaction::EthTransaction},
            eth_database_utils::{
                get_eos_on_eth_smart_contract_address_from_db,
                get_eth_account_nonce_from_db,
                get_eth_chain_id_from_db,
                get_eth_gas_price_from_db,
                get_eth_private_key_from_db,
            },
            eth_types::EthTransactions,
        },
    },
    eos_on_eth::eos::eos_tx_info::EosOnEthEosTxInfos,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::Address as EthAddress;

pub fn get_eth_signed_txs(
    tx_info: &EosOnEthEosTxInfos,
    smart_contract_address: &EthAddress,
    eth_account_nonce: u64,
    chain_id: u8,
    gas_price: u64,
    eth_private_key: EthPrivateKey,
) -> Result<EthTransactions> {
    info!("✔ Getting ETH signed transactions from `erc20-on-eos` redeem infos...");
    tx_info
        .iter()
        .enumerate()
        .map(|(i, redeem_info)| {
            info!(
                "✔ Signing ETH tx for amount: {}, to address: {}",
                redeem_info.amount, redeem_info.recipient
            );
            EthTransaction::new_unsigned(
                encode_erc777_mint_with_no_data_fxn(&redeem_info.recipient, &redeem_info.amount)?,
                eth_account_nonce + i as u64,
                ZERO_ETH_VALUE,
                *smart_contract_address,
                chain_id,
                ERC777_MINT_WITH_NO_DATA_GAS_LIMIT,
                gas_price,
            )
            .sign(eth_private_key.clone())
        })
        .collect::<Result<EthTransactions>>()
}

pub fn maybe_sign_normal_eth_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if state.erc20_on_eos_redeem_infos.len() == 0 {
        info!("✔ No EOS tx info in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        get_eth_signed_txs(
            &state.eos_on_eth_eos_tx_infos,
            &get_eos_on_eth_smart_contract_address_from_db(&state.db)?,
            get_eth_account_nonce_from_db(&state.db)?,
            get_eth_chain_id_from_db(&state.db)?,
            get_eth_gas_price_from_db(&state.db)?,
            get_eth_private_key_from_db(&state.db)?,
        )
        .and_then(|signed_txs| {
            #[cfg(feature = "debug")]
            {
                debug!("✔ Signed transactions: {:?}", signed_txs);
            }
            state.add_eth_signed_txs(signed_txs)
        })
    }
}
