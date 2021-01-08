use crate::{
    chains::{
        eos::{
            eos_action_proofs::EosActionProof,
            eos_state::EosState,
            eos_types::{GlobalSequence, GlobalSequences, ProcessedTxIds},
        },
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
    eos_on_eth::constants::MINIMUM_WEI_AMOUNT,
    traits::DatabaseInterface,
    types::Result,
};
use derive_more::{Constructor, Deref};
use eos_primitives::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct EosOnEthEosTxInfo {
    pub amount: U256,
    pub from: EosAccountName,
    pub recipient: EthAddress,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosOnEthEosTxInfos(pub Vec<EosOnEthEosTxInfo>);

impl EosOnEthEosTxInfos {
    pub fn from_action_proofs(action_proofs: &[EosActionProof]) -> Result<EosOnEthEosTxInfos> {
        Ok(EosOnEthEosTxInfos::new(
            action_proofs
                .iter()
                .map(|action_proof| action_proof.to_eos_on_eth_eos_tx_info())
                .collect::<Result<Vec<EosOnEthEosTxInfo>>>()?,
        ))
    }

    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedTxIds) -> Result<Self> {
        Ok(EosOnEthEosTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<EosOnEthEosTxInfo>>(),
        ))
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        GlobalSequences::new(
            self.iter()
                .map(|infos| infos.global_sequence)
                .collect::<Vec<GlobalSequence>>(),
        )
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnEthEosTxInfos::new(
            self.iter()
                .filter(|info| {
                    if info.amount >= min_amount {
                        true
                    } else {
                        info!("✘ Filtering out tx info ∵ value too low: {:?}", info);
                        false
                    }
                })
                .cloned()
                .collect::<Vec<EosOnEthEosTxInfo>>(),
        ))
    }

    // TODO/FIXME This needs finessing based on the token dictionary!
    pub fn to_eth_signed_txs(
        &self,
        smart_contract_address: &EthAddress,
        eth_account_nonce: u64,
        chain_id: u8,
        gas_price: u64,
        eth_private_key: EthPrivateKey,
    ) -> Result<EthTransactions> {
        info!("✔ Getting ETH signed transactions from `erc20-on-eos` redeem infos...");
        self.iter()
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
}

pub fn maybe_parse_eos_on_eth_eos_tx_infos_and_put_in_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Parsing redeem params from actions data...");
    EosOnEthEosTxInfos::from_action_proofs(&state.action_proofs).and_then(|tx_infos| {
        info!("✔ Parsed {} sets of redeem info!", tx_infos.len());
        state.add_eos_on_eth_eos_tx_info(tx_infos)
    })
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out already processed tx IDs...");
    state
        .eos_on_eth_eos_tx_infos
        .filter_out_already_processed_txs(&state.processed_tx_ids)
        .and_then(|filtered| state.add_eos_on_eth_eos_tx_info(filtered))
}

pub fn maybe_filter_out_value_too_low_txs_from_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Filtering out value too low txs from state...");
    state
        .eos_on_eth_eos_tx_infos
        .filter_out_those_with_value_too_low()
        .and_then(|filtered| state.replace_eos_on_eth_eos_tx_infos(filtered))
}

pub fn maybe_sign_normal_eth_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if state.erc20_on_eos_redeem_infos.len() == 0 {
        info!("✔ No EOS tx info in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        state
            .eos_on_eth_eos_tx_infos
            .to_eth_signed_txs(
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
