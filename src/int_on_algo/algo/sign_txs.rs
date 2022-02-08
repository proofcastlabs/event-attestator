use ethereum_types::Address as EthAddress;

use crate::{
    chains::{
        algo::algo_state::AlgoState,
        eth::{
            eth_chain_id::EthChainId,
            eth_constants::ZERO_ETH_VALUE,
            eth_contracts::erc20_vault::encode_erc20_vault_peg_out_fxn_data_with_user_data,
            eth_crypto::{
                eth_private_key::EthPrivateKey,
                eth_transaction::{EthTransaction as IntTransaction, EthTransactions as IntTransactions},
            },
            eth_database_utils::EthDbUtilsExt,
        },
    },
    int_on_algo::algo::int_tx_info::{IntOnAlgoIntTxInfo, IntOnAlgoIntTxInfos},
    metadata::metadata_traits::ToMetadata,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnAlgoIntTxInfo {
    pub fn to_evm_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        int_private_key: &EthPrivateKey,
    ) -> Result<IntTransaction> {
        info!("✔ Signing ETH transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas price: {}", gas_price);
        debug!(
            "✔ Signing tx for eventual token recipient: {}",
            self.destination_address,
        );
        debug!(
            "✔ Signing tx for token address : {}",
            self.int_token_address.to_string()
        );
        debug!(
            "✔ Signing tx for token amount: {}",
            self.native_token_amount.to_string()
        );
        debug!("✔ Signing tx for vault address: {}", self.int_vault_address.to_string());
        IntTransaction::new_unsigned(
            encode_erc20_vault_peg_out_fxn_data_with_user_data(
                self.router_address,
                self.int_token_address,
                self.native_token_amount,
                vec![], //self.to_metadata_bytes()?, // FIXME impl!
            )?,
            nonce,
            ZERO_ETH_VALUE,
            self.int_vault_address,
            chain_id,
            chain_id.get_erc20_vault_pegout_with_user_data_gas_limit(),
            gas_price,
        )
        .sign(int_private_key)
    }
}

impl IntOnAlgoIntTxInfos {
    pub fn to_eth_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        int_private_key: &EthPrivateKey,
    ) -> Result<IntTransactions> {
        info!("✔ Signing `IntOnAlgoIntTxInfos` INT transactions...");
        Ok(IntTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    IntOnAlgoIntTxInfo::to_evm_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_price,
                        int_private_key,
                    )
                })
                .collect::<Result<Vec<IntTransaction>>>()?,
        ))
    }
}

pub fn maybe_sign_int_txs_and_add_to_algo_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    let tx_infos = state.get_int_on_algo_int_tx_infos();
    if tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no INT transactions to sign!");
        Ok(state)
    } else {
        info!("✔ Signing transactions for `IntOnAlgoIntTxInfos`...");
        tx_infos
            .to_eth_signed_txs(
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                &state.eth_db_utils.get_eth_chain_id_from_db()?,
                state.eth_db_utils.get_eth_gas_price_from_db()?,
                &state.eth_db_utils.get_eth_private_key_from_db()?,
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

// TODO test!
