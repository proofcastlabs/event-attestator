use common::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    metadata::ToMetadataChainId,
    safe_addresses::safely_convert_str_to_eos_address,
    traits::DatabaseInterface,
    types::Result,
    EthChainId,
};
use common_eth::{
    Erc777RedeemEvent,
    EthDbUtilsExt,
    EthLog,
    EthState,
    EthSubmissionMaterial,
    ERC777_REDEEM_EVENT_TOPIC_V2,
};
use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::int::eos_tx_info::{EosOnIntEosTxInfo, EosOnIntEosTxInfos};

impl EosOnIntEosTxInfo {
    pub fn from_int_log(
        log: &EthLog,
        tx_hash: &EthHash,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Parsing `EosOnIntEosTxInfo` from ETH log...");
        Erc777RedeemEvent::from_eth_log(log).and_then(|params| {
            Ok(Self {
                token_amount: params.value,
                originating_tx_hash: *tx_hash,
                token_sender: params.redeemer,
                int_token_address: log.address,
                user_data: params.user_data.clone(),
                origin_chain_id: origin_chain_id.to_metadata_chain_id(),
                destination_chain_id: params.get_destination_chain_id()?,
                router_address: format!("0x{}", hex::encode(router_address)),
                eos_token_address: token_dictionary.get_eos_account_name_from_eth_token_address(&log.address)?,
                eos_asset_amount: token_dictionary.convert_u256_to_eos_asset_string(&log.address, &params.value)?,
                destination_address: safely_convert_str_to_eos_address(&params.underlying_asset_recipient).to_string(),
            })
        })
    }
}

impl EosOnIntEosTxInfos {
    pub fn from_int_submission_material(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
        router_address: &EthAddress,
    ) -> Result<Self> {
        Self::from_int_submission_material_without_filtering(
            material,
            token_dictionary,
            origin_chain_id,
            router_address,
        )
        .map(|tx_infos| {
            debug!("Num tx infos before filtering: {}", tx_infos.len());
            let filtered = tx_infos.filter_out_those_with_zero_eos_asset_amount(token_dictionary);
            debug!("Num tx infos after filtering: {}", filtered.len());
            filtered
        })
    }

    fn from_int_submission_material_without_filtering(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
        router_address: &EthAddress,
    ) -> Result<Self> {
        let eth_contract_addresses = token_dictionary.to_eth_addresses();
        debug!("Addresses from dict: {:?}", eth_contract_addresses);
        Ok(Self(
            material
                .receipts
                .get_receipts_containing_log_from_addresses_and_with_topics(&eth_contract_addresses, &[
                    *ERC777_REDEEM_EVENT_TOPIC_V2,
                ])
                .iter()
                .map(|receipt| {
                    receipt
                        .get_logs_from_addresses_with_topics(&eth_contract_addresses, &[*ERC777_REDEEM_EVENT_TOPIC_V2])
                        .iter()
                        .map(|log| {
                            EosOnIntEosTxInfo::from_int_log(
                                log,
                                &receipt.transaction_hash,
                                token_dictionary,
                                origin_chain_id,
                                router_address,
                            )
                        })
                        .collect::<Result<Vec<EosOnIntEosTxInfo>>>()
                })
                .collect::<Result<Vec<Vec<EosOnIntEosTxInfo>>>>()?
                .concat(),
        ))
    }
}

pub fn maybe_parse_eth_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `EosOnIntEosTxInfos`...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|material| match material.receipts.is_empty() {
            true => {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            },
            false => {
                info!(
                    "✔ {} receipts in canon block ∴ parsing INT tx infos...",
                    material.receipts.len()
                );
                EosOnIntEosTxInfos::from_int_submission_material(
                    &material,
                    state.get_eos_eth_token_dictionary()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                )
                .and_then(|tx_infos| tx_infos.to_bytes())
                .map(|tx_infos| state.add_tx_infos(tx_infos))
            },
        })
}
