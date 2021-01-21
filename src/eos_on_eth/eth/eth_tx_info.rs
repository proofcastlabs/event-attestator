use crate::{
    chains::{
        eos::{
            eos_crypto::eos_private_key::EosPrivateKey,
            eos_database_utils::get_eos_chain_id_from_db,
            eos_eth_token_dictionary::EosEthTokenDictionary,
            eos_types::EosSignedTransactions,
            sign_eos_transactions::get_signed_tx,
        },
        eth::{
            eth_constants::EOS_ON_ETH_ETH_TX_INFO_EVENT_TOPIC,
            eth_contracts::erc777::Erc777RedeemEvent,
            eth_database_utils::{get_eos_on_eth_smart_contract_address_from_db, get_eth_canon_block_from_db},
            eth_log::EthLog,
            eth_state::EthState,
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    eos_on_eth::constants::MINIMUM_WEI_AMOUNT,
    traits::DatabaseInterface,
    types::Result,
};
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EosOnEthEthTxInfos(pub Vec<EosOnEthEthTxInfo>);

impl EosOnEthEthTxInfos {
    pub fn from_eth_submission_material(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
        smart_contract_address: &EthAddress,
    ) -> Result<Self> {
        let topic = &EOS_ON_ETH_ETH_TX_INFO_EVENT_TOPIC[0];
        Ok(Self(
            material
                .receipts
                .get_receipts_containing_logs_from_address_and_with_topic(&smart_contract_address, topic)
                .iter()
                .map(|receipt| {
                    receipt
                        .get_logs_from_addresses_with_topic(&token_dictionary.to_eth_addresses(), topic)
                        .iter()
                        .map(|log| EosOnEthEthTxInfo::from_eth_log(&log, &receipt.transaction_hash, token_dictionary))
                        .collect::<Result<Vec<EosOnEthEthTxInfo>>>()
                })
                .collect::<Result<Vec<Vec<EosOnEthEthTxInfo>>>>()?
                .concat(),
        ))
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnEthEthTxInfos::new(
            self.iter()
                .filter(|info| {
                    if info.token_amount >= min_amount {
                        true
                    } else {
                        info!("✘ Filtering out tx info ∵ value too low: {:?}", info);
                        false
                    }
                })
                .cloned()
                .collect::<Vec<EosOnEthEthTxInfo>>(),
        ))
    }

    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &str,
        private_key: &EosPrivateKey,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `erc20-on-eos` peg in infos...", self.len());
        self.iter()
            .map(|tx_info| {
                info!("✔ Signing EOS tx from `erc20-on-eos` peg in info: {:?}", tx_info);
                get_signed_tx(
                    ref_block_num,
                    ref_block_prefix,
                    &tx_info.eos_address,
                    &tx_info.eos_asset_amount,
                    chain_id,
                    private_key,
                    &tx_info.eos_token_address,
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct EosOnEthEthTxInfo {
    pub token_amount: U256,
    pub eos_address: String,
    pub eos_token_address: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub eth_token_address: EthAddress,
    pub originating_tx_hash: EthHash,
}

impl EosOnEthEthTxInfo {
    pub fn from_eth_log(log: &EthLog, tx_hash: &EthHash, token_dictionary: &EosEthTokenDictionary) -> Result<Self> {
        Erc777RedeemEvent::from_eth_log(log).and_then(|params| {
            Ok(Self {
                token_amount: params.value,
                originating_tx_hash: *tx_hash,
                token_sender: params.redeemer,
                eth_token_address: log.address,
                eos_address: params.underlying_asset_recipient,
                eos_token_address: token_dictionary.get_eos_account_name_from_eth_token_address(&log.address)?,
                eos_asset_amount: token_dictionary.convert_u256_to_eos_asset_string(&log.address, &params.value)?,
            })
        })
    }
}

pub fn maybe_parse_eth_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `eos-on-eth` tx infos...");
    get_eth_canon_block_from_db(&state.db).and_then(|material| match material.receipts.is_empty() {
        true => {
            info!("✔ No receipts in canon block ∴ no info to parse!");
            Ok(state)
        },
        false => {
            info!(
                "✔ {} receipts in canon block ∴ parsing ETH tx info...",
                material.receipts.len()
            );
            EosOnEthEthTxInfos::from_eth_submission_material(
                &material,
                state.get_eos_eth_token_dictionary()?,
                &get_eos_on_eth_smart_contract_address_from_db(&state.db)?,
            )
            .and_then(|tx_infos| state.add_eos_on_eth_eth_tx_infos(tx_infos))
        },
    })
}

pub fn maybe_filter_out_eth_tx_info_with_value_too_low_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering `eos-on-eth` ETH tx infos...");
    debug!("✔ Num tx infos before: {}", state.erc20_on_eos_peg_in_infos.len());
    state
        .eos_on_eth_eth_tx_infos
        .filter_out_those_with_value_too_low()
        .and_then(|filtered_infos| {
            debug!("✔ Num tx infos after: {}", filtered_infos.len());
            state.replace_eos_on_eth_eth_tx_infos(filtered_infos)
        })
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe signing `erc20-on-eos` peg in txs...");
    let submission_material = state.get_eth_submission_material()?;
    state
        .eos_on_eth_eth_tx_infos
        .to_eos_signed_txs(
            submission_material.get_eos_ref_block_num()?,
            submission_material.get_eos_ref_block_prefix()?,
            &get_eos_chain_id_from_db(&state.db)?,
            &EosPrivateKey::get_from_db(&state.db)?,
        )
        .and_then(|signed_txs| state.add_eos_transactions(signed_txs))
}
