use eos_chain::{
    symbol::symbol_to_string as eos_symbol_to_string,
    AccountName as EosAccountName,
    Name as EosName,
    Symbol as EosSymbol,
};
use ethereum_types::Address as EthAddress;

use crate::{
    chains::{
        eos::{eos_action_proofs::EosActionProof, eos_chain_id::EosChainId, eos_state::EosState},
        eth::eth_database_utils::EthDbUtilsExt,
    },
    dictionaries::eos_eth::EosEthTokenDictionary,
    eos_on_int::eos::int_tx_info::{EosOnIntIntTxInfo, EosOnIntIntTxInfos},
    metadata::{MetadataChainId, ToMetadataChainId},
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::convert_bytes_to_u64,
};

const REQUIRED_ACTION_NAME: &str = "pegin";

impl EosOnIntIntTxInfo {
    fn get_token_sender_from_proof(proof: &EosActionProof) -> Result<EosAccountName> {
        let end_index = 7;
        proof
            .check_proof_action_data_length(
                end_index,
                "Not enough data to parse `EosOnIntIntTxInfo` sender from proof!",
            )
            .and_then(|_| {
                let result = EosAccountName::new(convert_bytes_to_u64(&proof.action.data[..=end_index])?);
                debug!("✔ Token sender parsed from action proof: {}", result);
                Ok(result)
            })
    }

    fn get_token_account_name_from_proof(proof: &EosActionProof) -> Result<EosAccountName> {
        let end_index = 15;
        let start_index = 8;
        proof
            .check_proof_action_data_length(
                end_index,
                "Not enough data to parse `EosOnIntIntTxInfo` account from proof!",
            )
            .and_then(|_| {
                let result = EosAccountName::new(convert_bytes_to_u64(&proof.action.data[start_index..=end_index])?);
                debug!("✔ Token account name parsed from action proof: {}", result);
                Ok(result)
            })
    }

    fn get_action_name_from_proof(proof: &EosActionProof) -> Result<EosName> {
        let end_index = 15;
        let start_index = 8;
        let serialized_action = proof.get_serialized_action()?;
        if serialized_action.len() < end_index + 1 {
            Err("Not enough data to parse `EosOnIntIntTxInfo` action name from proof!".into())
        } else {
            let result = EosName::new(convert_bytes_to_u64(
                &proof.get_serialized_action()?[start_index..=end_index],
            )?);
            debug!("✔ Action name parsed from action proof: {}", result);
            Ok(result)
        }
    }

    fn get_action_sender_account_name_from_proof(proof: &EosActionProof) -> Result<EosAccountName> {
        let end_index = 7;
        let serialized_action = proof.get_serialized_action()?;
        if serialized_action.len() < end_index + 1 {
            Err("Not enough data to parse `EosOnIntIntTxInfo` action sender from proof!".into())
        } else {
            let result = EosAccountName::new(convert_bytes_to_u64(&serialized_action[..=end_index])?);
            debug!("✔ Action sender account name parsed from action proof: {}", result);
            Ok(result)
        }
    }

    fn get_eos_symbol_from_proof(proof: &EosActionProof) -> Result<EosSymbol> {
        let start_index = 24;
        let end_index = 31;
        proof
            .check_proof_action_data_length(
                end_index,
                "Not enough data to parse `EosOnIntIntTxInfo` symbol from proof!",
            )
            .and_then(|_| {
                let result = EosSymbol::new(convert_bytes_to_u64(&proof.action.data[start_index..=end_index])?);
                debug!("✔ Eos symbol parsed from action proof: {}", result);
                Ok(result)
            })
    }

    fn get_token_symbol_from_proof(proof: &EosActionProof) -> Result<String> {
        let result = eos_symbol_to_string(Self::get_eos_symbol_from_proof(proof)?.as_u64());
        debug!("✔ Token symbol parsed from action proof: {}", result);
        Ok(result)
    }

    fn get_eos_amount_from_proof(proof: &EosActionProof) -> Result<u64> {
        let start_index = 16;
        let end_index = 23;
        proof
            .check_proof_action_data_length(
                end_index,
                "Not enough data to parse `EosOnIntIntTxInfo` amount from proof!",
            )
            .and_then(|_| convert_bytes_to_u64(&proof.action.data[start_index..=end_index]))
    }

    fn get_destination_address_from_proof(proof: &EosActionProof) -> String {
        // FIXME We need to parse this from the hex data for validation reasons!
        proof.action_json.data.destination_address.clone().unwrap_or_default()
    }

    fn get_destination_chain_id_from_proof(proof: &EosActionProof) -> Result<MetadataChainId> {
        // FIXME We need to parse this from the hex data for validation reasons!
        MetadataChainId::from_bytes(&hex::decode(
            proof
                .action_json
                .data
                .destination_chain_id
                .clone()
                .unwrap_or_else(|| MetadataChainId::default().to_string()),
        )?)
    }

    fn get_user_data_from_proof(proof: &EosActionProof) -> Result<Bytes> {
        // FIXME We need to parse this from the hex data for validation reasons!
        Ok(hex::decode(
            proof.action_json.data.destination_chain_id.clone().unwrap_or_default(),
        )?)
    }

    fn get_asset_num_decimals_from_proof(proof: &EosActionProof) -> Result<usize> {
        Self::get_eos_symbol_from_proof(proof).and_then(|symbol| {
            let symbol_string = symbol.to_string();
            let pieces = symbol_string.split(',').collect::<Vec<&str>>();
            if pieces.is_empty() {
                Err("Error getting number of decimals from `EosSymbol`!".into())
            } else {
                Ok(pieces[0].parse::<usize>()?)
            }
        })
    }

    fn check_proof_is_for_action(proof: &EosActionProof, required_action_name: &str) -> Result<()> {
        Self::get_action_name_from_proof(proof).and_then(|action_name| {
            if action_name.to_string() != required_action_name {
                return Err(format!("Proof does not appear to be for a '{}' action!", REQUIRED_ACTION_NAME).into());
            }
            Ok(())
        })
    }

    fn check_proof_is_from_contract(proof: &EosActionProof, contract: &EosAccountName) -> Result<()> {
        Self::get_action_sender_account_name_from_proof(proof).and_then(|ref action_sender| {
            if action_sender != contract {
                return Err(format!(
                    "Proof does not appear to be for an action from the EOS smart-contract: {}!",
                    contract
                )
                .into());
            }
            Ok(())
        })
    }

    pub fn from_eos_action_proof(
        proof: &EosActionProof,
        token_dictionary: &EosEthTokenDictionary,
        eos_smart_contract: &EosAccountName,
        router_address: &EthAddress,
        eos_chain_id: &EosChainId,
    ) -> Result<Self> {
        Self::check_proof_is_from_contract(proof, eos_smart_contract)
            .and_then(|_| Self::check_proof_is_for_action(proof, REQUIRED_ACTION_NAME))
            .and_then(|_| {
                info!("✔ Converting action proof to `eos-on-eth` eos tx info...");
                let token_address = Self::get_token_account_name_from_proof(proof)?;
                let dictionary_entry = token_dictionary.get_entry_via_eos_address_symbol_and_decimals(
                    &token_address,
                    &Self::get_token_symbol_from_proof(proof)?,
                    Self::get_asset_num_decimals_from_proof(proof)?,
                )?;
                let eos_amount = dictionary_entry.convert_u64_to_eos_asset(Self::get_eos_amount_from_proof(proof)?);
                let eth_amount = dictionary_entry.convert_eos_asset_to_eth_amount(&eos_amount)?;
                let tx_info = Self {
                    amount: eth_amount,
                    eos_tx_amount: eos_amount,
                    originating_tx_id: proof.tx_id,
                    router_address: *router_address,
                    vault_address: EthAddress::zero(), // NOTE: There is no EVM vault on this bridge.
                    global_sequence: proof.get_global_sequence(),
                    eos_token_address: dictionary_entry.eos_address,
                    user_data: Self::get_user_data_from_proof(proof)?,
                    origin_chain_id: eos_chain_id.to_metadata_chain_id(),
                    origin_address: Self::get_token_sender_from_proof(proof)?,
                    destination_address: Self::get_destination_address_from_proof(proof),
                    destination_chain_id: Self::get_destination_chain_id_from_proof(proof)?,
                    int_token_address: token_dictionary.get_eth_address_via_eos_address(&token_address)?,
                };
                debug!("Tx info parsed: {:?}", tx_info);
                Ok(tx_info)
            })
    }
}

impl EosOnIntIntTxInfos {
    pub fn from_eos_action_proofs(
        action_proofs: &[EosActionProof],
        token_dictionary: &EosEthTokenDictionary,
        eos_smart_contract: &EosAccountName,
        router_address: &EthAddress,
        eos_chain_id: &EosChainId,
    ) -> Result<Self> {
        Ok(EosOnIntIntTxInfos::new(
            action_proofs
                .iter()
                .map(|proof| {
                    EosOnIntIntTxInfo::from_eos_action_proof(
                        proof,
                        token_dictionary,
                        eos_smart_contract,
                        router_address,
                        eos_chain_id,
                    )
                })
                .collect::<Result<Vec<EosOnIntIntTxInfo>>>()?,
        ))
    }
}

pub fn maybe_parse_eos_on_int_int_tx_infos_and_put_in_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Parsing tx infos from actions...");
    EosOnIntIntTxInfos::from_eos_action_proofs(
        &state.action_proofs,
        state.get_eos_eth_token_dictionary()?,
        &state.eos_db_utils.get_eos_account_name_from_db()?,
        &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
        &state.eos_db_utils.get_eos_chain_id_from_db()?,
    )
    .and_then(|tx_infos| {
        info!("✔ Parsed {} sets of redeem info!", tx_infos.len());
        state.add_eos_on_int_int_tx_infos(tx_infos)
    })
}
