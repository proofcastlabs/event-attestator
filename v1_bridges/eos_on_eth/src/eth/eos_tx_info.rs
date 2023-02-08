use std::str::FromStr;

use common::{
    chains::{
        eos::{
            eos_actions::PTokenPegOutAction,
            eos_chain_id::EosChainId,
            eos_constants::{EOS_ACCOUNT_PERMISSION_LEVEL, PEGOUT_ACTION_NAME},
            eos_crypto::{
                eos_private_key::EosPrivateKey,
                eos_transaction::{EosSignedTransaction, EosSignedTransactions},
            },
            eos_utils::get_eos_tx_expiration_timestamp_with_offset,
        },
        eth::{
            eth_contracts::erc777_token::{
                Erc777RedeemEvent,
                ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
            },
            eth_database_utils::EthDbUtilsExt,
            eth_log::EthLog,
            eth_submission_material::EthSubmissionMaterial,
            EthState,
        },
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::{ToMetadata, ToMetadataChainId},
        Metadata,
    },
    safe_addresses::safely_convert_str_to_eos_address,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    EthChainId,
};
use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Action as EosAction, PermissionLevel, Transaction as EosTransaction};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use crate::{
    constants::MINIMUM_WEI_AMOUNT,
    fees_calculator::{FeeCalculator, FeesCalculator},
};

const ZERO_ETH_ASSET_STR: &str = "0.0000 EOS";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EosOnEthEosTxInfo {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub token_sender: EthAddress,
    pub eos_asset_amount: String,
    pub eos_token_address: String,
    pub origin_chain_id: EthChainId,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub eth_token_address: EthAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor, Deref, Serialize, Deserialize)]
pub struct EosOnEthEosTxInfos(pub Vec<EosOnEthEosTxInfo>);

impl FeesCalculator for EosOnEthEosTxInfos {
    fn get_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `EosOnEthEosTxInfos`...");
        self.iter()
            .map(|info| info.calculate_peg_out_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if fee.is_zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee, dictionary)
                        }
                    })
                    .collect::<Result<_>>()?,
            ))
        })
    }
}

impl EosOnEthEosTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }

    pub fn from_eth_submission_material(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        Self::from_eth_submission_material_without_filtering(material, token_dictionary, origin_chain_id).map(
            |tx_infos| {
                debug!("Num tx infos before filtering: {}", tx_infos.len());
                let filtered = tx_infos.filter_out_those_with_zero_eos_asset_amount(token_dictionary);
                debug!("Num tx infos after filtering: {}", filtered.len());
                filtered
            },
        )
    }

    fn from_eth_submission_material_without_filtering(
        material: &EthSubmissionMaterial,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        let eth_contract_addresses = token_dictionary.to_eth_addresses();
        debug!("Addresses from dict: {:?}", eth_contract_addresses);
        Ok(Self(
            material
                .receipts
                .get_receipts_containing_log_from_addresses_and_with_topics(&eth_contract_addresses, &[
                    *ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                    *ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
                ])
                .iter()
                .map(|receipt| {
                    receipt
                        .get_logs_from_addresses_with_topics(&eth_contract_addresses, &[
                            *ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
                            *ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
                        ])
                        .iter()
                        .map(|log| {
                            EosOnEthEosTxInfo::from_eth_log(
                                log,
                                &receipt.transaction_hash,
                                token_dictionary,
                                origin_chain_id,
                            )
                        })
                        .collect::<Result<Vec<EosOnEthEosTxInfo>>>()
                })
                .collect::<Result<Vec<Vec<EosOnEthEosTxInfo>>>>()?
                .concat(),
        ))
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnEthEosTxInfos::new(
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
                .collect::<Vec<EosOnEthEosTxInfo>>(),
        ))
    }

    pub fn to_eos_signed_txs(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        chain_id: &EosChainId,
        pk: &EosPrivateKey,
        eos_smart_contract: &EosAccountName,
    ) -> Result<EosSignedTransactions> {
        info!("✔ Signing {} EOS txs from `EosOnEthEosTxInfos`...", self.len());
        Ok(EosSignedTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    info!("✔ Signing EOS tx from `EosOnEthEosTxInfo`: {:?}", tx_info);
                    tx_info.to_eos_signed_tx(
                        ref_block_num,
                        ref_block_prefix,
                        eos_smart_contract,
                        ZERO_ETH_ASSET_STR,
                        chain_id,
                        pk,
                        get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                    )
                })
                .collect::<Result<Vec<EosSignedTransaction>>>()?,
        ))
    }

    fn filter_out_those_with_zero_eos_asset_amount(&self, dictionary: &EosEthTokenDictionary) -> Self {
        info!("✔ Filtering out `EosOnEthEosTxInfos` if they have a zero EOS asset amount...");
        Self::new(
            self.iter()
                .filter(|tx_info| {
                    match dictionary.get_zero_eos_asset_amount_via_eth_token_address(&tx_info.eth_token_address) {
                        Err(_) => {
                            info!(
                                "✘ Filtering out tx ∵ cannot determine zero EOS asset amount! {:?}",
                                tx_info
                            );
                            false
                        },
                        Ok(zero_asset_amount) => tx_info.eos_asset_amount != zero_asset_amount,
                    }
                })
                .cloned()
                .collect::<Vec<EosOnEthEosTxInfo>>(),
        )
    }
}

impl ToMetadata for EosOnEthEosTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        Ok(Metadata::new(
            &self.user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id.to_metadata_chain_id())?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Eos)
    }
}

impl FeeCalculator for EosOnEthEosTxInfo {
    fn get_amount(&self) -> U256 {
        info!("✔ Getting token amount in `EosOnEthEosTxInfo` of {}", self.token_amount);
        self.token_amount
    }

    fn get_eth_token_address(&self) -> EthAddress {
        debug!(
            "Getting EOS token address in `EosOnEthEosTxInfo` of {}",
            self.eth_token_address
        );
        self.eth_token_address
    }

    fn get_eos_token_address(&self) -> Result<EosAccountName> {
        debug!(
            "Getting EOS token address in `EosOnEthEosTxInfo` of {}",
            self.eos_token_address
        );
        Ok(EosAccountName::from_str(&self.eos_token_address)?)
    }

    fn update_amount(&self, new_amount: U256, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        let mut new_self = self.clone();
        new_self.token_amount = new_amount;
        new_self.eos_asset_amount =
            dictionary.convert_u256_to_eos_asset_string(&self.eth_token_address, &new_amount)?;
        Ok(new_self)
    }
}

impl EosOnEthEosTxInfo {
    pub fn from_eth_log(
        log: &EthLog,
        tx_hash: &EthHash,
        token_dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EthChainId,
    ) -> Result<Self> {
        info!("✔ Parsing `EosOnEthEosTxInfo` from ETH log...");
        Erc777RedeemEvent::from_eth_log(log).and_then(|params| {
            Ok(Self {
                token_amount: params.value,
                user_data: params.user_data,
                originating_tx_hash: *tx_hash,
                token_sender: params.redeemer,
                eth_token_address: log.address,
                origin_chain_id: origin_chain_id.clone(),
                eos_token_address: token_dictionary.get_eos_account_name_from_eth_token_address(&log.address)?,
                eos_asset_amount: token_dictionary.convert_u256_to_eos_asset_string(&log.address, &params.value)?,
                destination_address: safely_convert_str_to_eos_address(&params.underlying_asset_recipient).to_string(),
            })
        })
    }

    fn get_eos_ptoken_peg_out_action(
        from: &str,
        actor: &str,
        permission_level: &str,
        token_contract: &str,
        quantity: &str,
        recipient: &str,
        metadata: &[Byte],
    ) -> Result<EosAction> {
        debug!(
            "from: {}\nactor: {}\npermission_level: {}\ntoken_contract: {}\nquantity: {}\nrecipient: {}\nmetadata: '0x{}'",
            from, actor, permission_level, token_contract, quantity, recipient, hex::encode(metadata),
        );
        Ok(EosAction::from_str(
            from,
            PEGOUT_ACTION_NAME,
            vec![PermissionLevel::from_str(actor, permission_level)?],
            PTokenPegOutAction::from_str(token_contract, quantity, recipient, metadata)?,
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn to_eos_signed_tx(
        &self,
        ref_block_num: u16,
        ref_block_prefix: u32,
        eos_smart_contract: &EosAccountName,
        amount: &str,
        chain_id: &EosChainId,
        pk: &EosPrivateKey,
        timestamp: u32,
    ) -> Result<EosSignedTransaction> {
        info!("✔ Signing eos tx...");
        let metadata = if self.user_data.is_empty() {
            Ok(vec![])
        } else {
            info!("✔ Wrapping `user_data` in metadata for `EosOnEthEosTxInfo`...");
            self.to_metadata_bytes()
        }?;
        debug!(
            "smart-contract: {}\namount: {}\nchain ID: {}\nmetadata: {}",
            &eos_smart_contract,
            &amount,
            &chain_id.to_hex(),
            hex::encode(&metadata),
        );
        Self::get_eos_ptoken_peg_out_action(
            &eos_smart_contract.to_string(),
            &eos_smart_contract.to_string(),
            EOS_ACCOUNT_PERMISSION_LEVEL,
            &self.eos_token_address,
            &self.eos_asset_amount,
            &self.destination_address,
            &metadata,
        )
        .map(|action| EosTransaction::new(timestamp, ref_block_num, ref_block_prefix, vec![action]))
        .and_then(|ref unsigned_tx| {
            EosSignedTransaction::from_unsigned_tx(&eos_smart_contract.to_string(), amount, chain_id, pk, unsigned_tx)
        })
    }
}

pub fn maybe_parse_eth_tx_info_from_canon_block_and_add_to_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe parsing `eos-on-eth` tx infos...");
    state.eth_db_utils.get_eth_canon_block_from_db().and_then(|material| {
        if material.receipts.is_empty() {
            info!("✔ No receipts in canon block ∴ no info to parse!");
            Ok(state)
        } else {
            info!(
                "✔ {} receipts in canon block ∴ parsing ETH tx info...",
                material.receipts.len()
            );
            EosOnEthEosTxInfos::from_eth_submission_material(
                &material,
                state.get_eos_eth_token_dictionary()?,
                &state.eth_db_utils.get_eth_chain_id_from_db()?,
            )
            .and_then(|tx_infos| tx_infos.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
        }
    })
}

pub fn maybe_filter_out_eth_tx_info_with_value_too_low_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ Not filtering `EosOnEthEosTxInfos` because there are none to filter!");
        Ok(state)
    } else {
        info!("✔ Maybe filtering `EosOnEthEosTxInfos`...");
        EosOnEthEosTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                debug!("✔ Num tx infos before: {}", infos.len());
                infos.filter_out_those_with_value_too_low()
            })
            .and_then(|filtered_infos| {
                debug!("✔ Num tx infos after: {}", filtered_infos.len());
                filtered_infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

pub fn maybe_sign_eos_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ Not signing `EosOnEthEosTxInfos` because there are none to sign!");
        Ok(state)
    } else {
        info!("✔ Signing `EosOnEthEosTxInfos`...");
        let submission_material = state.get_eth_submission_material()?;
        EosOnEthEosTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                infos.to_eos_signed_txs(
                    submission_material.get_eos_ref_block_num()?,
                    submission_material.get_eos_ref_block_prefix()?,
                    &state.eos_db_utils.get_eos_chain_id_from_db()?,
                    &EosPrivateKey::get_from_db(state.db)?,
                    &state.eos_db_utils.get_eos_account_name_from_db()?,
                )
            })
            .and_then(|signed_txs| state.add_eos_transactions(signed_txs))
    }
}

pub fn maybe_filter_out_zero_eos_asset_amounts_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero eos asset amounts in state...");
    let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
    EosOnEthEosTxInfos::from_bytes(&state.tx_infos)
        .map(|infos| infos.filter_out_those_with_zero_eos_asset_amount(&dictionary))
        .and_then(|filtered| filtered.to_bytes())
        .map(|bytes| state.add_tx_infos(bytes))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::dictionaries::eos_eth::EosEthTokenDictionaryEntry;

    use super::*;
    use crate::test_utils::{
        get_dictionary_for_fee_calculations,
        get_eth_submission_material_n,
        get_eth_submission_material_with_bad_eos_account_name,
        get_eth_submission_material_with_two_peg_ins,
        get_sample_eos_eth_token_dictionary,
    };

    fn get_sample_eos_private_key() -> EosPrivateKey {
        EosPrivateKey::from_slice(
            &hex::decode("17b116e5e55af3b9985ff6c6e0320578176b83ca55570a66683d3b36d9deca64").unwrap(),
        )
        .unwrap()
    }

    fn get_sample_eos_on_eth_eth_tx_infos() -> EosOnEthEosTxInfos {
        EosOnEthEosTxInfos::from_eth_submission_material(
            &get_eth_submission_material_n(1).unwrap(),
            &get_sample_eos_eth_token_dictionary(),
            &EthChainId::Rinkeby,
        )
        .unwrap()
    }

    fn get_sample_eos_on_eth_eth_tx_info() -> EosOnEthEosTxInfo {
        get_sample_eos_on_eth_eth_tx_infos()[0].clone()
    }

    #[test]
    fn should_get_tx_info_from_eth_submission_material() {
        let tx_infos = get_sample_eos_on_eth_eth_tx_infos();
        let result = tx_infos[0].clone();
        let expected_token_amount = U256::from_dec_str("100000000000000").unwrap();
        let expected_eos_address = "whateverxxxx";
        let expected_eos_token_address = "eosio.token".to_string();
        let expected_eos_asset_amount = "0.0001 EOS".to_string();
        let expected_token_sender =
            EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap());
        let expected_eth_token_address =
            EthAddress::from_slice(&hex::decode("711c50b31ee0b9e8ed4d434819ac20b4fbbb5532").unwrap());
        let expected_originating_tx_hash = EthHash::from_slice(
            &hex::decode("9b9b2b88bdd495c132704154003d2deb65bd34ce6f8836ed6efdf0ba9def2b3e").unwrap(),
        );
        assert_eq!(result.token_amount, expected_token_amount);
        assert_eq!(result.destination_address, expected_eos_address);
        assert_eq!(result.eos_token_address, expected_eos_token_address);
        assert_eq!(result.eos_asset_amount, expected_eos_asset_amount);
        assert_eq!(result.token_sender, expected_token_sender);
        assert_eq!(result.eth_token_address, expected_eth_token_address);
        assert_eq!(result.originating_tx_hash, expected_originating_tx_hash);
    }

    #[test]
    fn should_get_eos_signed_txs_from_tx_info() {
        let tx_infos = get_sample_eos_on_eth_eth_tx_infos();
        let ref_block_num = 1;
        let ref_block_prefix = 1;
        let chain_id = EosChainId::EosMainnet;
        let pk = get_sample_eos_private_key();
        let eos_smart_contract = EosAccountName::from_str("11ppntoneos").unwrap();
        let result = tx_infos
            .to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &eos_smart_contract)
            .unwrap()[0]
            .transaction
            .clone();
        let expected_result = "010001000000000000000100305593e6596b0800000000644d99aa0100305593e6596b0800000000a8ed32322100a6823403ea3055010000000000000004454f5300000000d07bef576d954de30000";
        let result_with_no_timestamp = &result[8..];
        assert_eq!(result_with_no_timestamp, expected_result);
    }

    #[test]
    fn should_filter_out_zero_eth_amounts() {
        let dictionary = EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(
            "{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TOK\",\"eos_symbol\":\"EOS\",\"eth_address\":\"9a74c1e17b31745199b229b5c05b61081465b329\",\"eos_address\":\"eosio.token\"}"
        ).unwrap()]);
        let submission_material = get_eth_submission_material_n(2).unwrap();
        let expected_result_before = 1;
        let expected_result_after = 0;
        let origin_chain_id = EthChainId::Rinkeby;
        let result_before = EosOnEthEosTxInfos::from_eth_submission_material_without_filtering(
            &submission_material,
            &dictionary,
            &origin_chain_id,
        )
        .unwrap();
        assert_eq!(result_before.len(), expected_result_before);
        assert_eq!(result_before[0].eos_asset_amount, "0.0000 EOS");
        let result_after = result_before.filter_out_those_with_zero_eos_asset_amount(&dictionary);
        assert_eq!(result_after.len(), expected_result_after);
    }

    #[test]
    fn should_default_to_safe_address_when_signing_tx_with_bad_eos_account_name_in_submission_material() {
        let token_dictionary_entry_str = "{\"eth_token_decimals\":18,\"eos_token_decimals\":4,\"eth_symbol\":\"TLOS\",\"eos_symbol\":\"TLOS\",\"eth_address\":\"b6c53431608e626ac81a9776ac3e999c5556717c\",\"eos_address\":\"eosio.token\"}";
        let token_dictionary =
            EosEthTokenDictionary::new(vec![
                EosEthTokenDictionaryEntry::from_str(token_dictionary_entry_str).unwrap()
            ]);
        let submission_material = get_eth_submission_material_with_bad_eos_account_name();
        let origin_chain_id = EthChainId::Rinkeby;
        let tx_infos =
            EosOnEthEosTxInfos::from_eth_submission_material(&submission_material, &token_dictionary, &origin_chain_id)
                .unwrap();
        let ref_block_num = 1;
        let ref_block_prefix = 2;
        let chain_id = EosChainId::EosMainnet;
        let eos_smart_contract = EosAccountName::from_str("11ppntoneos").unwrap();
        let pk = get_sample_eos_private_key();
        let result = tx_infos.to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &eos_smart_contract);
        assert!(result.is_ok());
    }

    #[test]
    fn same_param_tx_infos_should_not_create_same_signatures() {
        let submission_material = get_eth_submission_material_with_two_peg_ins();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let origin_chain_id = EthChainId::Rinkeby;
        let tx_infos =
            EosOnEthEosTxInfos::from_eth_submission_material(&submission_material, &dictionary, &origin_chain_id)
                .unwrap();
        let ref_block_num = 1;
        let ref_block_prefix = 2;
        let chain_id = EosChainId::EosMainnet;
        let eos_smart_contract = EosAccountName::from_str("11ppntoneos").unwrap();
        let pk = get_sample_eos_private_key();
        let result = tx_infos
            .to_eos_signed_txs(ref_block_num, ref_block_prefix, &chain_id, &pk, &eos_smart_contract)
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_ne!(result[0], result[1]);
    }

    #[test]
    fn should_subtract_amount_from_eos_on_eth_eth_tx_info() {
        let info = get_sample_eos_on_eth_eth_tx_info();
        let subtrahend = U256::from(1337);
        let dictionary = get_sample_eos_eth_token_dictionary();
        let expected_eos_asset_amount = "0.0000 EOS".to_string();
        let expected_token_amount = U256::from_dec_str("99999999998663").unwrap();
        assert!(info.token_amount > expected_token_amount);
        assert_ne!(info.eos_asset_amount, expected_eos_asset_amount);
        let mut expected_result = info.clone();
        expected_result.token_amount = expected_token_amount;
        expected_result.eos_asset_amount = expected_eos_asset_amount;
        let result = info.subtract_amount(subtrahend, &dictionary).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_fees_from_eos_on_eth_eth_tx_infos() {
        let dictionary = get_dictionary_for_fee_calculations();
        let infos = get_sample_eos_on_eth_eth_tx_infos();
        let result = infos.get_fees(&dictionary).unwrap();
        let expected_result = vec![(
            EthAddress::from_slice(&hex::decode("711c50b31ee0b9e8ed4d434819ac20b4fbbb5532").unwrap()),
            U256::from_dec_str("120000000000").unwrap(),
        )];
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_fees_from_eos_on_eth_eth_tx_infos() {
        let dictionary = get_dictionary_for_fee_calculations();
        let infos = get_sample_eos_on_eth_eth_tx_infos();
        let result = infos.subtract_fees(&dictionary).unwrap();
        let expected_amount = U256::from_dec_str("99880000000000").unwrap();
        let expected_result = EosOnEthEosTxInfos::new(vec![get_sample_eos_on_eth_eth_tx_info()
            .update_amount(expected_amount, &dictionary)
            .unwrap()]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_metadata_bytes_from_eos_on_eth_eth_tx_info() {
        let info = get_sample_eos_on_eth_eth_tx_info();
        let result = hex::encode(info.to_metadata_bytes().unwrap());
        let expected_result =
            "01000400f343682a307866656466653236313665623336363163623866656432373832663566306363393164353964636163";
        assert_eq!(result, expected_result);
    }
}
