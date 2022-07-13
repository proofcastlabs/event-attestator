use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::{Erc20VaultPegInEventParams, ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2},
        eth_database_utils::EthDbUtilsExt,
        eth_log::{EthLog, EthLogExt, EthLogs},
        eth_receipt::EthReceipt,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    int_on_evm::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos},
    safe_addresses::safely_convert_str_to_eth_address,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEvmEvmTxInfos {
    fn is_log_erc20_on_evm_peg_in(log: &EthLog, vault_address: &EthAddress) -> Result<bool> {
        let log_contains_topic = log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2);
        let log_is_from_vault_address = log.address == *vault_address;
        Ok(log_contains_topic && log_is_from_vault_address)
    }

    pub fn get_supported_erc20_on_evm_logs_from_receipt(receipt: &EthReceipt, vault_address: &EthAddress) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_erc20_on_evm_peg_in(log, vault_address), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `int-on-evm` peg in infos from receipt...");
        Ok(Self::new(
            Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, vault_address)
                .iter()
                .map(|log| {
                    let event_params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let tx_info = IntOnEvmEvmTxInfo {
                        vault_address: *vault_address,
                        router_address: *router_address,
                        token_sender: event_params.token_sender,
                        user_data: event_params.user_data.clone(),
                        eth_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        native_token_amount: event_params.token_amount,
                        origin_chain_id: event_params.get_origin_chain_id()?,
                        destination_chain_id: event_params.get_destination_chain_id()?,
                        destination_address: safely_convert_str_to_eth_address(&event_params.destination_address),
                        evm_token_address: dictionary.get_evm_address_from_eth_address(&event_params.token_address)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<IntOnEvmEvmTxInfo>>>()?,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEvmEvmTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, vault_address, dictionary, router_address))
                .collect::<Result<Vec<IntOnEvmEvmTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<IntOnEvmEvmTxInfo>>>()
                .concat(),
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `erc20-on-int` peg-in infos...");
    state
        .eth_db_utils
        .get_eth_canon_block_from_db()
        .and_then(|submission_material| match submission_material.receipts.is_empty() {
            true => {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            },
            false => {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                IntOnEvmEvmTxInfos::from_submission_material(
                    &submission_material,
                    &state.eth_db_utils.get_int_on_evm_smart_contract_address_from_db()?,
                    &EthEvmTokenDictionary::get_from_db(state.db)?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                )
                .and_then(|tx_infos| state.add_int_on_evm_evm_tx_infos(tx_infos))
            },
        })
}

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::{
        chains::eth::eth_utils::convert_hex_to_eth_address,
        int_on_evm::test_utils::{
            get_sample_peg_in_submission_material,
            get_sample_router_address,
            get_sample_token_dictionary,
            get_sample_vault_address,
        },
        metadata::metadata_chain_id::MetadataChainId,
    };

    #[test]
    fn should_get_erc20_on_evm_evm_tx_info_from_submission_material() {
        let material = get_sample_peg_in_submission_material();
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_token_dictionary();
        let router_address = get_sample_router_address();
        let results =
            IntOnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary, &router_address)
                .unwrap();
        let expected_num_results = 1;
        assert_eq!(results.len(), expected_num_results);
        let result = results[0].clone();
        assert_eq!(result.token_sender, router_address);
        assert_eq!(result.router_address, router_address);
        assert_eq!(result.user_data, hex::decode("c0ffee").unwrap());
        assert_eq!(result.origin_chain_id, MetadataChainId::InterimChain);
        assert_eq!(result.destination_chain_id, MetadataChainId::EthereumRopsten);
        assert_eq!(result.native_token_amount, U256::from_dec_str("1337").unwrap());
        assert_eq!(
            result.evm_token_address,
            convert_hex_to_eth_address("dd9f905a34a6c507c7d68384985905cf5eb032e9").unwrap(),
        );
        assert_eq!(
            result.eth_token_address,
            convert_hex_to_eth_address("a83446f219baec0b6fd6b3031c5a49a54543045b").unwrap(),
        );
        assert_eq!(
            result.destination_address,
            convert_hex_to_eth_address("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap(),
        );
        assert_eq!(
            result.originating_tx_hash,
            EthHash::from_slice(
                &hex::decode("41bda64700fcd700e2c5ec7015da9b224f6c55e4859cb18ea164f2f826bede31").unwrap(),
            ),
        );
    }
}
