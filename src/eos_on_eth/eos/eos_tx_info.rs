use crate::{
    chains::{
        eos::{
            eos_action_proofs::EosActionProof,
            eos_eth_token_dictionary::EosEthTokenDictionary,
            eos_global_sequences::{GlobalSequence, GlobalSequences, ProcessedGlobalSequences},
            eos_state::EosState,
        },
        eth::{
            eth_constants::ZERO_ETH_VALUE,
            eth_contracts::erc777::{encode_erc777_mint_with_no_data_fxn, ERC777_MINT_WITH_NO_DATA_GAS_LIMIT},
            eth_crypto::{eth_private_key::EthPrivateKey, eth_transaction::EthTransaction},
            eth_database_utils::{
                get_eth_account_nonce_from_db,
                get_eth_chain_id_from_db,
                get_eth_gas_price_from_db,
                get_eth_private_key_from_db,
            },
            eth_types::EthTransactions,
        },
    },
    constants::SAFE_ETH_ADDRESS_HEX,
    eos_on_eth::constants::MINIMUM_WEI_AMOUNT,
    traits::DatabaseInterface,
    types::Result,
    utils::{convert_bytes_to_u64, maybe_strip_hex_prefix},
};
use derive_more::{Constructor, Deref};
use eos_primitives::{
    symbol::symbol_to_string as eos_symbol_to_string,
    AccountName as EosAccountName,
    Checksum256,
    Name as EosName,
    SerializeData,
    Symbol as EosSymbol,
};
use ethereum_types::{Address as EthAddress, U256};
use std::str::{from_utf8, FromStr};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct EosOnEthEosTxInfo {
    pub amount: U256,
    pub from: EosAccountName,
    pub recipient: EthAddress,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub eth_token_address: EthAddress,
}

impl EosOnEthEosTxInfo {
    fn get_token_sender_from_proof(proof: &EosActionProof) -> Result<EosAccountName> {
        let result = EosAccountName::new(convert_bytes_to_u64(&proof.action.data[..8])?);
        debug!("✔ Token sender parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_token_account_name_from_proof(proof: &EosActionProof) -> Result<EosAccountName> {
        let result = EosAccountName::new(convert_bytes_to_u64(&proof.action.data[8..16])?);
        debug!("✔ Token account name parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_action_name_from_proof(proof: &EosActionProof) -> Result<EosName> {
        let result = EosName::new(convert_bytes_to_u64(&proof.get_serialized_action()[8..16])?);
        debug!("✔ Action name parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_action_sender_account_name_from_proof(proof: &EosActionProof) -> Result<EosName> {
        let result = EosName::new(convert_bytes_to_u64(&proof.get_serialized_action()[..8])?);
        debug!("✔ Action sender account name parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_eos_symbol_from_proof(proof: &EosActionProof) -> Result<EosSymbol> {
        let result = EosSymbol::new(convert_bytes_to_u64(&proof.action.data[24..32])?);
        debug!("✔ Eos symbol parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_token_symbol_from_proof(proof: &EosActionProof) -> Result<String> {
        let result = eos_symbol_to_string(Self::get_eos_symbol_from_proof(proof)?.as_u64());
        debug!("✔ Token symbol parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_eos_amount_from_proof(proof: &EosActionProof) -> Result<u64> {
        convert_bytes_to_u64(&proof.action.data[16..24].to_vec())
    }

    fn get_eth_address_from_proof(proof: &EosActionProof) -> Result<EthAddress> {
        let result = EthAddress::from_slice(&hex::decode(maybe_strip_hex_prefix(&from_utf8(
            &proof.action.data[33..75],
        )?)?)?);
        debug!("✔ ETH address parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_eth_address_from_proof_or_revert_to_safe_eth_address(proof: &EosActionProof) -> Result<EthAddress> {
        let safe_address = EthAddress::from_slice(&hex::decode(SAFE_ETH_ADDRESS_HEX)?);
        match Self::get_eth_address_from_proof(proof) {
            Ok(eth_address) => Ok(eth_address),
            Err(_) => {
                info!(
                    "✘ Error getting ETH addess from proof! Default to `SAFE_ETH_ADDRESS`: {}",
                    safe_address
                );
                Ok(safe_address)
            },
        }
    }

    pub fn from_eos_action_proof(proof: &EosActionProof, token_dictionary: &EosEthTokenDictionary) -> Result<Self> {
        info!("✔ Converting action proof to `eos-on-eth` eos tx info...");
        let eos_address = Self::get_token_account_name_from_proof(&proof)?;
        let dictionary_entry = token_dictionary.get_entry_via_eos_address(&eos_address)?;
        let eos_asset = dictionary_entry.convert_u64_to_eos_asset(Self::get_eos_amount_from_proof(proof)?)?;
        let eth_amount = dictionary_entry.convert_eos_asset_to_eth_amount(&eos_asset.to_string())?;
        Ok(Self {
            amount: eth_amount,
            originating_tx_id: proof.tx_id,
            global_sequence: proof.get_global_sequence(),
            from: Self::get_token_sender_from_proof(proof)?,
            recipient: Self::get_eth_address_from_proof_or_revert_to_safe_eth_address(proof)?,
            eth_token_address: token_dictionary.get_eth_address_via_eos_address(&eos_address)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosOnEthEosTxInfos(pub Vec<EosOnEthEosTxInfo>);

impl EosOnEthEosTxInfos {
    pub fn from_eos_action_proofs(
        action_proofs: &[EosActionProof],
        token_dictionary: &EosEthTokenDictionary,
    ) -> Result<Self> {
        Ok(EosOnEthEosTxInfos::new(
            action_proofs
                .iter()
                .map(|ref proof| EosOnEthEosTxInfo::from_eos_action_proof(proof, token_dictionary))
                .collect::<Result<Vec<EosOnEthEosTxInfo>>>()?,
        ))
    }

    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedGlobalSequences) -> Result<Self> {
        Ok(EosOnEthEosTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<EosOnEthEosTxInfo>>(),
        ))
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        GlobalSequences::new(
            self.iter()
                .map(|infos| infos.global_sequence)
                .collect::<Vec<GlobalSequence>>(),
        )
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnEthEosTxInfos::new(
            self.iter()
                .filter(|info| {
                    if info.amount >= min_amount {
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

    pub fn to_eth_signed_txs(
        &self,
        eth_account_nonce: u64,
        chain_id: u8,
        gas_price: u64,
        eth_private_key: EthPrivateKey,
    ) -> Result<EthTransactions> {
        info!("✔ Getting ETH signed transactions from `erc20-on-eos` redeem infos...");
        self.iter()
            .enumerate()
            .map(|(i, tx_info)| {
                info!(
                    "✔ Signing ETH tx for amount: {}, to address: {}",
                    tx_info.amount, tx_info.recipient
                );
                EthTransaction::new_unsigned(
                    encode_erc777_mint_with_no_data_fxn(&tx_info.recipient, &tx_info.amount)?,
                    eth_account_nonce + i as u64,
                    ZERO_ETH_VALUE,
                    tx_info.eth_token_address,
                    chain_id,
                    ERC777_MINT_WITH_NO_DATA_GAS_LIMIT,
                    gas_price,
                )
                .sign(eth_private_key.clone())
            })
            .collect::<Result<EthTransactions>>()
    }
}

pub fn maybe_parse_eos_on_eth_eos_tx_infos_and_put_in_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Parsing redeem params from actions data...");
    EosOnEthEosTxInfos::from_eos_action_proofs(&state.action_proofs, state.get_eos_eth_token_dictionary()?).and_then(
        |tx_infos| {
            info!("✔ Parsed {} sets of redeem info!", tx_infos.len());
            state.add_eos_on_eth_eos_tx_info(tx_infos)
        },
    )
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out already processed tx IDs...");
    state
        .eos_on_eth_eos_tx_infos
        .filter_out_already_processed_txs(&state.processed_tx_ids)
        .and_then(|filtered| state.add_eos_on_eth_eos_tx_info(filtered))
}

pub fn maybe_filter_out_value_too_low_txs_from_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Filtering out value too low txs from state...");
    state
        .eos_on_eth_eos_tx_infos
        .filter_out_those_with_value_too_low()
        .and_then(|filtered| state.replace_eos_on_eth_eos_tx_infos(filtered))
}

pub fn maybe_sign_normal_eth_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if state.erc20_on_eos_redeem_infos.len() == 0 {
        info!("✔ No EOS tx info in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        state
            .eos_on_eth_eos_tx_infos
            .to_eth_signed_txs(
                get_eth_account_nonce_from_db(&state.db)?,
                get_eth_chain_id_from_db(&state.db)?,
                get_eth_gas_price_from_db(&state.db)?,
                get_eth_private_key_from_db(&state.db)?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_eth_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eos::{
            eos_eth_token_dictionary::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
            eos_utils::convert_hex_to_checksum256,
        },
        eos_on_eth::eos::eos_test_utils::get_eos_on_eth_submission_material_n,
        utils::convert_u64_to_bytes,
    };

    fn get_sample_eos_eth_token_dictionary() -> EosEthTokenDictionary {
        EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(&
        "{\"eos_token_decimals\":4,\"eth_token_decimals\":18,\"eos_symbol\":\"EOS\",\"eth_symbol\":\"PEOS\",\"eos_address\":\"eosio.token\",\"eth_address\":\"fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\"}").unwrap()])
    }

    fn get_sample_proof() -> EosActionProof {
        get_eos_on_eth_submission_material_n(1).unwrap().action_proofs[0].clone()
    }

    #[test]
    fn should_get_token_sender_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_token_sender_from_proof(&proof).unwrap();
        let expected_result = EosAccountName::from_str("oraclizetest").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_token_account_name_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_token_account_name_from_proof(&proof).unwrap();
        let expected_result = EosAccountName::from_str("eosio.token").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_action_name_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_action_name_from_proof(&proof).unwrap();
        let expected_result = EosName::from_str("pegin").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_action_sender_account_name_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_action_sender_account_name_from_proof(&proof).unwrap();
        let expected_result = EosName::from_str("t11ppntoneos").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eos_symbol_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_eos_symbol_from_proof(&proof).unwrap();
        let expected_result = EosSymbol::from_str("4,EOS").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_token_symbol_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_token_symbol_from_proof(&proof).unwrap();
        let expected_result = "EOS".to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eos_amount_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_eos_amount_from_proof(&proof).unwrap();
        let expected_result = 1 as u64;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eth_address_from_proof() {
        let proof = get_sample_proof();
        let result = EosOnEthEosTxInfo::get_eth_address_from_proof(&proof).unwrap();
        let expected_result = EthAddress::from_slice(&hex::decode("5fDAEf0a0B11774dB68C38aB36957De8646aF1B5").unwrap());
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eos_on_eth_eth_tx_info_from_action_proof() {
        let proof = get_sample_proof();
        let dictionary = get_sample_eos_eth_token_dictionary();
        let result = EosOnEthEosTxInfo::from_eos_action_proof(&proof, &dictionary).unwrap();
        let expected_amount = U256::from_dec_str("100000000000000").unwrap();
        let expected_from = EosAccountName::from_str("oraclizetest").unwrap();
        let expected_recipient =
            EthAddress::from_slice(&hex::decode("5fDAEf0a0B11774dB68C38aB36957De8646aF1B5").unwrap());
        let expected_originating_tx_id =
            convert_hex_to_checksum256("cb2e6fbd5c82fb50b3c2e0658a887aa359f9f6b398457448322d86968a28e794").unwrap();
        let expected_global_sequence = 323917921677;
        let expected_eth_token_address =
            EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap());
        assert_eq!(result.amount, expected_amount);
        assert_eq!(result.from, expected_from);
        assert_eq!(result.recipient, expected_recipient);
        assert_eq!(result.global_sequence, expected_global_sequence);
        assert_eq!(result.originating_tx_id, expected_originating_tx_id);
        assert_eq!(result.eth_token_address, expected_eth_token_address);
    }
}
