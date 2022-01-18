use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_contracts::erc777::{Erc777RedeemEvent, ERC777_REDEEM_EVENT_TOPIC_V2},
        eth_database_utils::EthDbUtilsExt,
        eth_log::{EthLog, EthLogExt, EthLogs},
        eth_receipt::EthReceipt,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
        eth_utils::safely_convert_hex_to_eth_address,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::int::eth_tx_info::{EthOnIntEthTxInfo, EthOnIntEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl EthOnIntEthTxInfos {
    fn is_log_an_erc20_on_int_redeem(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        debug!(
            "✔ Checking log contains topic: {}",
            hex::encode(ERC777_REDEEM_EVENT_TOPIC_V2.as_bytes())
        );
        let token_is_supported = dictionary.is_evm_token_supported(&log.address);
        let log_contains_topic = log.contains_topic(&ERC777_REDEEM_EVENT_TOPIC_V2);
        debug!("✔ Log is supported: {}", token_is_supported);
        debug!("✔ Log has correct topic: {}", log_contains_topic);
        Ok(token_is_supported && log_contains_topic)
    }

    pub fn is_log_a_supported_redeem_event(log: &EthLog, dictionary: &EthEvmTokenDictionary) -> Result<bool> {
        match Self::is_log_an_erc20_on_int_redeem(log, dictionary)? {
            false => Ok(false),
            true => Ok(dictionary.is_evm_token_supported(&log.address)),
        }
    }

    pub fn get_logs_with_redeem_event_from_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
    ) -> EthLogs {
        EthLogs::new(
            receipt
                .logs
                .iter()
                .filter(|log| matches!(Self::is_log_a_supported_redeem_event(log, dictionary), Ok(true)))
                .cloned()
                .collect(),
        )
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Getting `EthOnIntEthTxInfos` from receipt...");
        Ok(Self::new(
            Self::get_logs_with_redeem_event_from_receipt(receipt, dictionary)
                .iter()
                .map(|log| {
                    let event_params = Erc777RedeemEvent::from_eth_log(log)?;
                    let tx_info = EthOnIntEthTxInfo {
                        evm_token_address: log.address,
                        token_sender: event_params.redeemer,
                        origin_chain_id: origin_chain_id.clone(),
                        user_data: event_params.user_data.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        eth_token_address: dictionary.get_eth_address_from_evm_address(&log.address)?,
                        destination_address: safely_convert_hex_to_eth_address(
                            &event_params.underlying_asset_recipient,
                        )?,
                        native_token_amount: dictionary
                            .convert_evm_amount_to_eth_amount(&log.address, event_params.value)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<EthOnIntEthTxInfo>>>()?,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Getting `EthOnIntEthTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, dictionary, origin_chain_id))
                .collect::<Result<Vec<EthOnIntEthTxInfos>>>()?
                .into_iter()
                .flatten()
                .collect(),
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `EthOnIntEthTxInfos`...");
    state
        .evm_db_utils
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
                EthEvmTokenDictionary::get_from_db(state.db)
                    .and_then(|account_names| {
                        EthOnIntEthTxInfos::from_submission_material(
                            &submission_material,
                            &account_names,
                            &state.evm_db_utils.get_eth_chain_id_from_db()?,
                        )
                    })
                    .and_then(|tx_infos| state.add_erc20_on_int_eth_tx_infos(tx_infos))
            },
        })
}

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::{
        chains::eth::eth_utils::convert_hex_to_eth_address,
        erc20_on_int::test_utils::{
            get_sample_peg_out_submission_material,
            get_sample_router_address,
            get_sample_token_dictionary,
        },
    };

    #[test]
    fn should_get_erc20_on_evm_eth_tx_info_from_submission_material() {
        let dictionary = get_sample_token_dictionary();
        let origin_chain_id = EthChainId::Ropsten;
        let material = get_sample_peg_out_submission_material();
        let results = EthOnIntEthTxInfos::from_submission_material(&material, &dictionary, &origin_chain_id).unwrap();
        let expected_num_results = 1;
        assert_eq!(results.len(), expected_num_results);
        let result = results[0].clone();
        assert_eq!(result.origin_chain_id, origin_chain_id);
        assert_eq!(result.user_data, hex::decode("decaff").unwrap());
        assert_eq!(result.native_token_amount, U256::from_dec_str("665").unwrap());
        assert_eq!(result.token_sender, get_sample_router_address());
        assert_eq!(
            result.evm_token_address,
            convert_hex_to_eth_address("0xa83446f219baec0b6fd6b3031c5a49a54543045b").unwrap(),
        );
        assert_eq!(
            result.eth_token_address,
            convert_hex_to_eth_address("0xc63ab9437f5589e2c67e04c00a98506b43127645").unwrap(),
        );
        assert_eq!(
            result.destination_address,
            convert_hex_to_eth_address("0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap(),
        );
        assert_eq!(
            result.originating_tx_hash,
            EthHash::from_slice(
                &hex::decode("149b9d2522fa706c17218ace8816e853b687ad740940ee0f45255fe285d93b32").unwrap(),
            )
        );
    }
}