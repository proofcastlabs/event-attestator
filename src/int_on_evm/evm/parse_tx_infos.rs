use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        eth_contracts::erc777::Erc777RedeemEvent,
        eth_database_utils::EthDbUtilsExt,
        eth_receipt::EthReceipt,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEvmIntTxInfos {
    fn from_eth_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
        router_address: &EthAddress,
        eth_vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEvmIntTxInfos` from receipt...");
        Ok(Self::new(
            Self::get_relevant_logs_from_receipt(receipt, dictionary)
                .iter()
                .map(|log| {
                    let event_params = Erc777RedeemEvent::from_eth_log(log)?;
                    let tx_info = IntOnEvmIntTxInfo {
                        router_address: *router_address,
                        token_sender: event_params.redeemer,
                        user_data: event_params.user_data.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        origin_chain_id: event_params.get_origin_chain_id()?,
                        evm_token_address: format!("0x{}", hex::encode(log.address)),
                        destination_chain_id: event_params.get_destination_chain_id()?,
                        eth_vault_address: format!("0x{}", hex::encode(eth_vault_address)),
                        destination_address: event_params.underlying_asset_recipient.clone(),
                        eth_token_address: format!(
                            "0x{}",
                            hex::encode(dictionary.get_eth_address_from_evm_address(&log.address)?)
                        ),
                        native_token_amount: dictionary
                            .convert_evm_amount_to_eth_amount(&log.address, event_params.value)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<IntOnEvmIntTxInfo>>>()?,
        ))
    }

    pub fn from_submission_material(
        submission_material: &EthSubmissionMaterial,
        dictionary: &EthEvmTokenDictionary,
        router_address: &EthAddress,
        eth_vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEvmIntTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, dictionary, router_address, eth_vault_address))
                .collect::<Result<Vec<IntOnEvmIntTxInfos>>>()?
                .into_iter()
                .flatten()
                .collect(),
        ))
    }
}

pub fn maybe_parse_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `IntOnEvmIntTxInfos`...");
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
                        IntOnEvmIntTxInfos::from_submission_material(
                            &submission_material,
                            &account_names,
                            &state.evm_db_utils.get_eth_router_smart_contract_address_from_db()?,
                            &state.evm_db_utils.get_int_on_evm_smart_contract_address_from_db()?,
                        )
                    })
                    .and_then(|tx_infos| state.add_int_on_evm_int_tx_infos(tx_infos))
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
            get_sample_peg_out_submission_material,
            get_sample_router_address,
            get_sample_token_dictionary,
        },
        metadata::metadata_chain_id::MetadataChainId,
    };

    #[test]
    fn should_get_erc20_on_evm_eth_tx_info_from_submission_material() {
        let dictionary = get_sample_token_dictionary();
        let material = get_sample_peg_out_submission_material();
        let router_address = get_sample_router_address();
        let vault_address = EthAddress::default();
        let results =
            IntOnEvmIntTxInfos::from_submission_material(&material, &dictionary, &router_address, &vault_address)
                .unwrap();
        let expected_num_results = 1;
        assert_eq!(results.len(), expected_num_results);
        let result = results[0].clone();
        assert_eq!(result.router_address, router_address);
        assert_eq!(result.user_data, hex::decode("decaff").unwrap());
        assert_eq!(result.origin_chain_id, MetadataChainId::EthereumRopsten);
        assert_eq!(result.destination_chain_id, MetadataChainId::EthereumRinkeby);
        assert_eq!(result.native_token_amount, U256::from_dec_str("666").unwrap());
        assert_eq!(
            result.token_sender,
            convert_hex_to_eth_address("0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap(),
        );
        assert_eq!(result.evm_token_address, "0xdd9f905a34a6c507c7d68384985905cf5eb032e9");
        assert_eq!(result.eth_token_address, "0xa83446f219baec0b6fd6b3031c5a49a54543045b");
        assert_eq!(result.destination_address, "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC");
        assert_eq!(
            result.originating_tx_hash,
            EthHash::from_slice(
                &hex::decode("61ac238ba14d7f8bc1fff8546f61127d9b44be6955819adb0f9861da6723bef1").unwrap(),
            ),
        );
    }
}
