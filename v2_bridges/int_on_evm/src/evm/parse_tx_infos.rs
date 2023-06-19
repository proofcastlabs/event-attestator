use common::{dictionaries::eth_evm::EthEvmTokenDictionary, traits::DatabaseInterface, types::Result};
use common_eth::{Erc777RedeemEvent, EthDbUtilsExt, EthLog, EthReceipt, EthState, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;

use crate::{
    constants::{PLTC_ADDRESS, PTLOS_ADDRESS},
    evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
};

impl IntOnEvmIntTxInfos {
    fn get_destination_chain_id(log: &EthLog, event_params: &Erc777RedeemEvent) -> Result<MetadataChainId> {
        if log.address == *PTLOS_ADDRESS {
            warn!("pTelos peg out detected, defaulting to TELOS mainnet as destination chain ID");
            Ok(MetadataChainId::TelosMainnet)
        } else if log.address == *PLTC_ADDRESS {
            warn!("pLTC peg out detected, defaulting to LTC mainnet as destination chain ID");
            Ok(MetadataChainId::LitecoinMainnet)
        } else {
            // NOTE This will error for legacy events that are not explicitly handled above, because
            // there will be no destination chain ID in the event log.
            event_params.get_destination_chain_id()
        }
    }

    fn get_origin_chain_id(log: &EthLog, event_params: &Erc777RedeemEvent) -> Result<MetadataChainId> {
        if log.address == *PTLOS_ADDRESS {
            warn!("pTelos peg out detected, defaulting to ETH mainnet as origin chain ID");
            Ok(MetadataChainId::EthereumMainnet)
        } else if log.address == *PLTC_ADDRESS {
            warn!("pLTC peg out detected, defaulting to ETH mainnet as origin chain ID");
            Ok(MetadataChainId::EthereumMainnet)
        } else {
            // NOTE This will error for legacy events that are not explicitly handled above, because
            // there will be no destination chain ID in the event log.
            event_params.get_origin_chain_id()
        }
    }

    fn from_eth_receipt(
        receipt: &EthReceipt,
        dictionary: &EthEvmTokenDictionary,
        router_address: &EthAddress,
        vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEvmIntTxInfos` from receipt...");
        Ok(Self::new(
            Self::get_relevant_logs_from_receipt(receipt, dictionary)
                .iter()
                .map(|log| {
                    // NOTE: The event parser can handle v1 events w/ & w/out user data, and also v2 events.
                    // This core will filter for some v1 events in order to facilitate migration from v1 to v2
                    // of some legacy bridges which do not have upgradeable smart contracts.
                    let event_params = Erc777RedeemEvent::from_eth_log(log)?;
                    let origin_chain_id = Self::get_origin_chain_id(log, &event_params)?;
                    let destination_chain_id = Self::get_destination_chain_id(log, &event_params)?;
                    let tx_info = IntOnEvmIntTxInfo {
                        origin_chain_id,
                        destination_chain_id,
                        vault_address: *vault_address,
                        evm_token_address: log.address,
                        router_address: *router_address,
                        token_sender: event_params.redeemer,
                        host_token_amount: event_params.value,
                        user_data: event_params.user_data.clone(),
                        originating_tx_hash: receipt.transaction_hash,
                        destination_address: event_params.underlying_asset_recipient.clone(),
                        eth_token_address: dictionary.get_eth_address_from_evm_address(&log.address)?,
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
        vault_address: &EthAddress,
    ) -> Result<Self> {
        info!("✔ Getting `IntOnEvmIntTxInfos` from submission material...");
        Ok(Self::new(
            submission_material
                .get_receipts()
                .iter()
                .map(|receipt| Self::from_eth_receipt(receipt, dictionary, router_address, vault_address))
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
        .and_then(|submission_material| {
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            } else {
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
                    .and_then(|tx_infos| tx_infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            }
        })
}

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_eth_address;
    use common_metadata::MetadataChainId;
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::test_utils::{
        get_sample_peg_out_submission_material,
        get_sample_router_address,
        get_sample_token_dictionary,
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
        assert_eq!(
            result.evm_token_address,
            convert_hex_to_eth_address("0xdd9f905a34a6c507c7d68384985905cf5eb032e9").unwrap()
        );
        assert_eq!(
            result.eth_token_address,
            convert_hex_to_eth_address("0xa83446f219baec0b6fd6b3031c5a49a54543045b").unwrap()
        );
        assert_eq!(result.destination_address, "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC");
        assert_eq!(
            result.originating_tx_hash,
            EthHash::from_slice(
                &hex::decode("61ac238ba14d7f8bc1fff8546f61127d9b44be6955819adb0f9861da6723bef1").unwrap(),
            ),
        );
    }
}
