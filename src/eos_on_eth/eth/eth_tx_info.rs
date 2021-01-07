use crate::{
    chains::{
        eos::{
            eos_database_utils::get_eos_account_name_string_from_db,
            eos_unit_conversions::convert_u64_to_eos_asset,
        },
        eth::{
            eth_constants::EOS_ON_ETH_ETH_TX_INFO_EVENT_TOPIC,
            eth_contracts::erc777::get_redeem_event_params_from_log,
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
    // TODO Test once we have sample material!
    pub fn from_eth_submission_material<D: DatabaseInterface>(
        db: &D,
        material: &EthSubmissionMaterial,
    ) -> Result<Self> {
        let address = get_eos_on_eth_smart_contract_address_from_db(db)?;
        let topic = &EOS_ON_ETH_ETH_TX_INFO_EVENT_TOPIC[0];
        Ok(Self(
            material
                .receipts
                .get_receipts_containing_logs_from_address_and_with_topic(&address, topic)
                .iter()
                .map(|receipt| {
                    receipt
                        .get_logs_from_address_with_topic(&address, topic)
                        .iter()
                        .map(|log| EosOnEthEthTxInfo::from_eth_log(db, &log, &receipt.transaction_hash))
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
    pub fn from_eth_log<D: DatabaseInterface>(db: &D, log: &EthLog, tx_hash: &EthHash) -> Result<Self> {
        get_redeem_event_params_from_log(log).and_then(|params| {
            Ok(Self {
                token_amount: params.value,
                eos_address: params.underlying_asset_recipient,
                eos_token_address: get_eos_account_name_string_from_db(db)?,
                eos_asset_amount: convert_u64_to_eos_asset(params.value.as_u64()),
                token_sender: params.redeemer,
                eth_token_address: get_eos_on_eth_smart_contract_address_from_db(db)?,
                originating_tx_hash: *tx_hash,
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
            EosOnEthEthTxInfos::from_eth_submission_material(&state.db, &material)
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
