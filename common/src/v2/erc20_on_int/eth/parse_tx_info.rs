use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::{Erc20VaultPegInEventParams, ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2},
        eth_database_utils::EthDbUtilsExt,
        eth_log::{EthLog, EthLogExt, EthLogs},
        eth_receipt::EthReceipt,
        eth_submission_material::EthSubmissionMaterial,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

impl Erc20OnIntIntTxInfos {
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
        info!("✔ Getting `erc20-on-int` peg in infos from receipt...");
        Ok(Self::new(
            Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, vault_address)
                .iter()
                .map(|log| {
                    let event_params = Erc20VaultPegInEventParams::from_eth_log(log)?;
                    let tx_info = Erc20OnIntIntTxInfo {
                        vault_address: *vault_address,
                        router_address: *router_address,
                        token_sender: event_params.token_sender,
                        user_data: event_params.user_data.clone(),
                        eth_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        native_token_amount: event_params.token_amount,
                        origin_chain_id: event_params.get_origin_chain_id()?,
                        destination_address: event_params.destination_address.clone(),
                        destination_chain_id: event_params.get_destination_chain_id()?,
                        evm_token_address: dictionary.get_evm_address_from_eth_address(&event_params.token_address)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<Erc20OnIntIntTxInfo>>>()?,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
        dictionary: &EthEvmTokenDictionary,
        router_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `Erc20OnIntIntTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, vault_address, dictionary, router_address))
                .collect::<Result<Vec<Erc20OnIntIntTxInfos>>>()?
                .iter()
                .map(|infos| infos.iter().cloned().collect())
                .collect::<Vec<Vec<Erc20OnIntIntTxInfo>>>()
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
        .and_then(|submission_material| {
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                Erc20OnIntIntTxInfos::from_submission_material(
                    &submission_material,
                    &state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                    &EthEvmTokenDictionary::get_from_db(state.db)?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                )
                .and_then(|tx_infos| tx_infos.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::{
        chains::eth::eth_utils::convert_hex_to_eth_address,
        erc20_on_int::test_utils::{
            get_sample_peg_in_1_submission_material,
            get_sample_router_address,
            get_sample_token_dictionary,
            get_sample_vault_address,
        },
        metadata::metadata_chain_id::MetadataChainId,
    };

    #[test]
    fn should_get_erc20_on_evm_evm_tx_info_from_submission_material() {
        let material = get_sample_peg_in_1_submission_material();
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_token_dictionary();
        let router_address = get_sample_router_address();
        let results =
            Erc20OnIntIntTxInfos::from_submission_material(&material, &vault_address, &dictionary, &router_address)
                .unwrap();
        let expected_num_results = 1;
        assert_eq!(results.len(), expected_num_results);
        let result = results[0].clone();
        assert_eq!(result.router_address, router_address);
        assert_eq!(result.user_data, hex::decode("c0ffee").unwrap());
        assert_eq!(result.origin_chain_id, MetadataChainId::EthereumRinkeby);
        assert_eq!(result.destination_chain_id, MetadataChainId::EthereumRopsten);
        assert_eq!(result.native_token_amount, U256::from_dec_str("1337").unwrap());
        assert_eq!(
            result.token_sender,
            convert_hex_to_eth_address("0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap(),
        );
        assert_eq!(
            result.evm_token_address,
            convert_hex_to_eth_address("0xa83446f219baec0b6fd6b3031c5a49a54543045b").unwrap()
        );
        assert_eq!(
            result.eth_token_address,
            convert_hex_to_eth_address("0xc63ab9437f5589e2c67e04c00a98506b43127645").unwrap(),
        );
        assert_eq!(result.destination_address, "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac");
        assert_eq!(
            result.originating_tx_hash,
            EthHash::from_slice(
                &hex::decode("f691d432fe940b2ecef70b6c9069ae124af9d160d761252d7ca570f5cd443dd4").unwrap(),
            ),
        );
    }
}
