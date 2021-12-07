use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_constants::{MAX_BYTES_FOR_ETH_USER_DATA, ZERO_ETH_VALUE},
        eth_contracts::{
            erc20_vault::{Erc20VaultPegInEventParams, ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2},
            erc777::{encode_erc777_mint_fxn_maybe_with_data, ERC777_MINT_WITH_DATA_GAS_LIMIT},
        },
        eth_crypto::{
            eth_private_key::EthPrivateKey as EvmPrivateKey,
            eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
        },
        eth_database_utils::EthDbUtilsExt,
        eth_log::{EthLog, EthLogExt, EthLogs},
        eth_receipt::{EthReceipt, EthReceipts},
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
        eth_utils::safely_convert_hex_to_eth_address,
    },
    constants::SAFE_EVM_ADDRESS,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    int_on_evm::fees_calculator::{FeeCalculator, FeesCalculator},
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::ToMetadata,
        Metadata,
    },
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

#[derive(Debug, Clone, PartialEq, Eq, Constructor)]
pub struct IntOnEvmEvmTxInfo {
    pub native_token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
    pub origin_chain_id: MetadataChainId,
    pub router_address: EthAddress,
    pub destination_chain_id: MetadataChainId,
}

impl ToMetadata for IntOnEvmEvmTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            // TODO Test for this case!
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new_v2(
            &user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id)?,
            &self.destination_chain_id,
            &MetadataAddress::new_from_eth_address(&self.destination_address, &self.destination_chain_id)?,
            None,
            None,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}

impl FeeCalculator for IntOnEvmEvmTxInfo {
    fn get_amount(&self) -> U256 {
        debug!(
            "Getting token amount in `IntOnEvmEvmTxInfo` of {}",
            self.native_token_amount
        );
        self.native_token_amount
    }

    fn get_token_address(&self) -> EthAddress {
        debug!(
            "Getting token address in `IntOnEvmEvmTxInfo` of {}",
            self.eth_token_address
        );
        self.eth_token_address
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.native_token_amount {
            Err("Cannot subtract amount from `IntOnEvmEvmTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.native_token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `EthOnEvmEthTxInfo`!",
                subtrahend, self.native_token_amount, new_amount
            );
            Ok(self.update_amount(new_amount))
        }
    }
}

impl IntOnEvmEvmTxInfo {
    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.native_token_amount = new_amount;
        new_self
    }

    fn update_destination_address(&self, new_address: EthAddress) -> Self {
        let mut new_self = self.clone();
        new_self.destination_address = new_address;
        new_self
    }

    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        info!("✔ Checking if the destination address is the same as the INT token contract address...");
        if self.destination_address == self.evm_token_address {
            info!("✔ Recipient address is same as INT token address! Diverting to safe address...");
            self.update_destination_address(*SAFE_EVM_ADDRESS)
        } else {
            self.clone()
        }
    }

    pub fn to_evm_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransaction> {
        let operator_data = None;
        let metadata_bytes = if self.user_data.is_empty() {
            vec![]
        } else {
            self.to_metadata_bytes()?
        };
        info!("✔ Signing INT transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        if !metadata_bytes.is_empty() {
            debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata_bytes))
        } else {
            debug!("✔ No user data ∴ not wrapping in metadata!");
        };
        encode_erc777_mint_fxn_maybe_with_data(
            &self.destination_address,
            &self.get_host_token_amount(dictionary)?,
            if metadata_bytes.is_empty() {
                None
            } else {
                Some(metadata_bytes)
            },
            operator_data,
        )
        .map(|data| {
            EvmTransaction::new_unsigned(
                data,
                nonce,
                ZERO_ETH_VALUE,
                self.evm_token_address,
                chain_id,
                gas_limit,
                gas_price,
            )
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(evm_private_key))
    }

    pub fn get_host_token_amount(&self, dictionary: &EthEvmTokenDictionary) -> Result<U256> {
        dictionary.convert_eth_amount_to_evm_amount(&self.eth_token_address, self.native_token_amount)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct IntOnEvmEvmTxInfos(pub Vec<IntOnEvmEvmTxInfo>);

impl FeesCalculator for IntOnEvmEvmTxInfos {
    fn get_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `IntOnEvmEvmTxInfo`...");
        self.iter()
            .map(|info| info.calculate_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if *fee == U256::zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<IntOnEvmEvmTxInfo>>>()?,
            ))
        })
    }
}

impl IntOnEvmEvmTxInfos {
    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        Self::new(
            self.iter()
                .map(|info| info.divert_to_safe_address_if_destination_is_token_contract_address())
                .collect::<Vec<IntOnEvmEvmTxInfo>>(),
        )
    }

    fn get_host_token_amounts(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<U256>> {
        self.iter()
            .map(|tx_info| tx_info.get_host_token_amount(dictionary))
            .collect::<Result<Vec<U256>>>()
    }

    pub fn filter_out_zero_values(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        let host_token_amounts = self.get_host_token_amounts(dictionary)?;
        Ok(Self::new(
            self.iter()
                .zip(host_token_amounts.iter())
                .filter(|(tx_info, evm_amount)| match *evm_amount != &U256::zero() {
                    true => true,
                    false => {
                        info!(
                            "✘ Filtering out peg in info due to zero INT asset amount: {:?}",
                            tx_info
                        );
                        false
                    },
                })
                .map(|(info, _)| info)
                .cloned()
                .collect::<Vec<IntOnEvmEvmTxInfo>>(),
        ))
    }

    fn is_log_erc20_on_evm_peg_in(log: &EthLog, vault_address: &EthAddress) -> Result<bool> {
        let log_contains_topic = log.contains_topic(&ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2);
        let log_is_from_vault_address = log.address == *vault_address;
        Ok(log_contains_topic && log_is_from_vault_address)
    }

    fn receipt_contains_supported_erc20_on_evm_peg_in(receipt: &EthReceipt, vault_address: &EthAddress) -> bool {
        Self::get_supported_erc20_on_evm_logs_from_receipt(receipt, vault_address).len() > 0
    }

    fn get_supported_erc20_on_evm_logs_from_receipt(receipt: &EthReceipt, vault_address: &EthAddress) -> EthLogs {
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
                    let tx_info = IntOnEvmEvmTxInfo {
                        router_address: *router_address,
                        token_sender: event_params.token_sender,
                        user_data: event_params.user_data.clone(),
                        eth_token_address: event_params.token_address,
                        originating_tx_hash: receipt.transaction_hash,
                        native_token_amount: event_params.token_amount,
                        origin_chain_id: event_params.get_origin_chain_id()?,
                        destination_chain_id: event_params.get_destination_chain_id()?,
                        destination_address: safely_convert_hex_to_eth_address(&event_params.destination_address)?,
                        evm_token_address: dictionary.get_evm_address_from_eth_address(&event_params.token_address)?,
                    };
                    info!("✔ Parsed tx info: {:?}", tx_info);
                    Ok(tx_info)
                })
                .collect::<Result<Vec<IntOnEvmEvmTxInfo>>>()?,
        ))
    }

    fn filter_eth_submission_material_for_supported_peg_ins(
        submission_material: &EthSubmissionMaterial,
        vault_address: &EthAddress,
    ) -> Result<EthSubmissionMaterial> {
        info!("✔ Filtering submission material receipts for those pertaining to `erc20-on-int` peg-ins...");
        info!(
            "✔ Num receipts before filtering: {}",
            submission_material.receipts.len()
        );
        let filtered_receipts = EthReceipts::new(
            submission_material
                .receipts
                .iter()
                .filter(|receipt| {
                    IntOnEvmEvmTxInfos::receipt_contains_supported_erc20_on_evm_peg_in(receipt, vault_address)
                })
                .cloned()
                .collect(),
        );
        info!("✔ Num receipts after filtering: {}", filtered_receipts.len());
        Ok(EthSubmissionMaterial::new(
            submission_material.get_block()?,
            filtered_receipts,
            None,
            None,
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

    pub fn to_evm_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `erc20-on-int` INT transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    IntOnEvmEvmTxInfo::to_evm_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_limit,
                        gas_price,
                        evm_private_key,
                        dictionary,
                    )
                })
                .collect::<Result<Vec<EvmTransaction>>>()?,
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

pub fn filter_out_zero_value_evm_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero value `IntOnEvmEvmTxInfos`...");
    debug!(
        "✔ Num `IntOnEvmEvmTxInfos` before: {}",
        state.int_on_evm_int_tx_infos.len()
    );
    state
        .int_on_evm_evm_tx_infos
        .filter_out_zero_values(&EthEvmTokenDictionary::get_from_db(state.db)?)
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `IntOnEvmEvmTxInfos` after: {}", filtered_tx_infos.len());
            state.replace_int_on_evm_evm_tx_infos(filtered_tx_infos)
        })
}

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `erc20-on-int` peg in events...");
    let vault_address = state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?;
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(&vault_address, &[*ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2])
        .and_then(|filtered_submission_material| {
            IntOnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(
                &filtered_submission_material,
                &vault_address,
            )
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}

pub fn maybe_sign_evm_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.erc20_on_int_int_tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no INT transactions to sign!");
        Ok(state)
    } else {
        state
            .int_on_evm_evm_tx_infos
            .to_evm_signed_txs(
                state.evm_db_utils.get_eth_account_nonce_from_db()?,
                &state.evm_db_utils.get_eth_chain_id_from_db()?,
                ERC777_MINT_WITH_DATA_GAS_LIMIT,
                state.evm_db_utils.get_eth_gas_price_from_db()?,
                &state.evm_db_utils.get_eth_private_key_from_db()?,
                &EthEvmTokenDictionary::get_from_db(state.db)?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_int_on_evm_evm_signed_txs(signed_txs)
            })
    }
}

pub fn maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.int_on_evm_evm_tx_infos.is_empty() {
        Ok(state)
    } else {
        info!("✔ Maybe diverting EVM txs to safe address if destination address is the token contract address...");
        let new_infos = state
            .int_on_evm_evm_tx_infos
            .divert_to_safe_address_if_destination_is_token_contract_address();
        state.replace_int_on_evm_evm_tx_infos(new_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_traits::EthTxInfoCompatible,
        dictionaries::eth_evm::test_utils::get_sample_eth_evm_dictionary,
        int_on_evm::test_utils::{
            get_eth_submission_material_n,
            get_sample_eth_evm_token_dictionary,
            get_sample_evm_private_key,
            get_sample_router_address,
            get_sample_vault_address,
        },
    };

    fn get_sample_tx_infos() -> IntOnEvmEvmTxInfos {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_eth_evm_token_dictionary();
        let router_address = get_sample_router_address();
        let origin_chain_id = EthChainId::Mainnet;
        IntOnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary, &router_address).unwrap()
    }

    fn get_sample_tx_info() -> IntOnEvmEvmTxInfo {
        get_sample_tx_infos()[0].clone()
    }

    #[test]
    fn should_filter_submission_info_for_supported_redeems() {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let result =
            IntOnEvmEvmTxInfos::filter_eth_submission_material_for_supported_peg_ins(&material, &vault_address)
                .unwrap();
        let expected_num_receipts = 1;
        assert_eq!(result.receipts.len(), expected_num_receipts);
    }

    #[test]
    fn should_get_erc20_on_evm_evm_tx_info_from_submission_material() {
        let material = get_eth_submission_material_n(1);
        let vault_address = get_sample_vault_address();
        let dictionary = get_sample_eth_evm_token_dictionary();
        let router_address = get_sample_router_address();
        let origin_chain_id = MetadataChainId::EthereumMainnet;
        let destination_chain_id = MetadataChainId::BscMainnet;
        let result =
            IntOnEvmEvmTxInfos::from_submission_material(&material, &vault_address, &dictionary, &router_address)
                .unwrap();
        let expected_num_results = 1;
        assert_eq!(result.len(), expected_num_results);
        let expected_result = IntOnEvmEvmTxInfos::new(vec![IntOnEvmEvmTxInfo {
            router_address,
            user_data: vec![],
            native_token_amount: U256::from_dec_str("1000000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            eth_token_address: EthAddress::from_slice(
                &hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap(),
            ),
            // NOTE It's the `SAFE_EVM_ADDRESS_HEX` ∵ @bertani accidentally included the `"`s in the pegin!
            destination_address: EthAddress::from_slice(
                &hex::decode("71a440ee9fa7f99fb9a697e96ec7839b8a1643b8").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("578670d0e08ca172eb8e862352e731814564fd6a12c3143e88bfb28292cd1535").unwrap(),
            ),
            origin_chain_id,
            destination_chain_id,
        }]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_signaures_from_evm_tx_info() {
        let dictionary = get_sample_eth_evm_dictionary();
        let pk = get_sample_evm_private_key();
        let infos = get_sample_tx_infos();
        let nonce = 0_u64;
        let chain_id = EthChainId::Rinkeby;
        let gas_limit = 300_000_usize;
        let gas_price = 20_000_000_000_u64;
        let signed_txs = infos
            .to_evm_signed_txs(nonce, &chain_id, gas_limit, gas_price, &pk, &dictionary)
            .unwrap();
        let expected_num_results = 1;
        assert_eq!(signed_txs.len(), expected_num_results);
        let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
        let expected_tx_hex = "f901cb808504a817c800830493e094daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af9280b90164dcdc7dd00000000000000000000000001a3496c18d558bd9c6c8f609e1b129f67ab081630000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000a001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080005fe7f9000000000000000000000000000000000000000000000000000000000000000000000000000000008127192c2e4703dfb47f087883cc3120fe061cb8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ba0fe68d226a6cfb842c64716190bec65f6a84598df7f7e96d3247da55a901f78eba02369d6d02220847934b2542336399c925752f9a288babedadb45d5154143c5ca"
;
        assert_eq!(tx_hex, expected_tx_hex);
    }

    #[test]
    fn should_calculate_eth_on_evm_evm_tx_info_fee() {
        let info = get_sample_tx_info();
        let fee_basis_points = 25;
        let result = info.calculate_fee(fee_basis_points);
        let expected_result = U256::from_dec_str("2500000000000000").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_amount_from_eth_on_evm_evm_tx_info() {
        let info = get_sample_tx_info();
        let subtrahend = U256::from(1337);
        let result = info.subtract_amount(subtrahend).unwrap();
        let expected_native_token_amount = U256::from_dec_str("999999999999998663").unwrap();
        assert_eq!(result.native_token_amount, expected_native_token_amount)
    }

    #[test]
    fn should_fail_to_subtract_too_large_amount_from_eth_on_evm_evm_tx_info() {
        let info = get_sample_tx_info();
        let subtrahend = U256::from(info.native_token_amount + 1);
        let result = info.subtract_amount(subtrahend);
        assert!(result.is_err());
    }

    #[test]
    fn should_divert_to_safe_address_if_destination_is_token_address() {
        let destination_address =
            EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let router_address = get_sample_router_address();
        let info = IntOnEvmEvmTxInfo {
            router_address,
            user_data: vec![],
            destination_address,
            native_token_amount: U256::from_dec_str("1000000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            eth_token_address: EthAddress::from_slice(
                &hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("578670d0e08ca172eb8e862352e731814564fd6a12c3143e88bfb28292cd1535").unwrap(),
            ),
            origin_chain_id: MetadataChainId::EthereumMainnet,
            destination_chain_id: MetadataChainId::BscMainnet,
        };
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_token_contract_address();
        assert_eq!(result.destination_address, *SAFE_EVM_ADDRESS);
    }
}
