use ethereum_types::{Address as EthAddress, U256};

use crate::{
    chains::{
        eos::{eos_action_proofs::EosActionProof, eos_chain_id::EosChainId, eos_state::EosState},
        eth::eth_database_utils::EthDbUtilsExt,
    },
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    int_on_eos::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos},
    metadata::{metadata_chain_id::MetadataChainId, metadata_traits::ToMetadataChainId},
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::convert_bytes_to_u64,
};

impl IntOnEosIntTxInfos {
    pub fn from_action_proofs(
        action_proofs: &[EosActionProof],
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EosChainId,
        int_vault_address: &EthAddress,
        router_address: &EthAddress,
    ) -> Result<Self> {
        Ok(Self::new(
            action_proofs
                .iter()
                .map(|action_proof| {
                    IntOnEosIntTxInfo::from_action_proof(
                        action_proof,
                        dictionary,
                        origin_chain_id,
                        int_vault_address,
                        router_address,
                    )
                })
                .collect::<Result<Vec<IntOnEosIntTxInfo>>>()?,
        ))
    }
}

impl IntOnEosIntTxInfo {
    // NOTE: We should be decoding this from the actual hex blob in the proof, but since we don't
    // yet have an EOS codec, we can't deserialize it. Instead we rely on the json.
    fn get_destination_address_from_proof(proof: &EosActionProof) -> String {
        proof.action_json.data.memo.clone().unwrap_or_default()
    }

    // NOTE: Ibid.
    fn get_destination_chain_id_from_proof(proof: &EosActionProof) -> Result<MetadataChainId> {
        Ok(MetadataChainId::from_bytes(&hex::decode(
            proof.action_json.data.chain_id.clone().unwrap_or_default(),
        )?)
        .unwrap_or_default())
    }

    // NOTE: Ibid.
    fn get_user_data_from_proof(proof: &EosActionProof) -> Result<Bytes> {
        Ok(hex::decode(
            proof.action_json.data.user_data.clone().unwrap_or_default(),
        )?)
    }

    fn get_redeem_amount_from_proof(
        proof: &EosActionProof,
        dictionary_entry: &EosEthTokenDictionaryEntry,
    ) -> Result<U256> {
        proof
            .check_proof_action_data_length(15, "Not enough data to parse `IntOnEosIntTxInfo` amount from proof!")
            .and_then(|_| {
                Ok(dictionary_entry.convert_u64_to_eos_asset(convert_bytes_to_u64(&proof.action.data[8..=15])?))
            })
            .and_then(|eos_asset| dictionary_entry.convert_eos_asset_to_eth_amount(&eos_asset))
    }

    pub fn from_action_proof(
        proof: &EosActionProof,
        dictionary: &EosEthTokenDictionary,
        origin_chain_id: &EosChainId,
        int_vault_address: &EthAddress,
        router_address: &EthAddress,
    ) -> Result<Self> {
        dictionary
            .get_entry_via_eos_address_and_symbol(
                &proof.get_eos_asset_symbol()?,
                &proof.get_action_eos_account().to_string(),
            )
            .and_then(|dictionary_entry| {
                let amount = Self::get_redeem_amount_from_proof(proof, &dictionary_entry)?;
                let eos_tx_amount = dictionary_entry.convert_u256_to_eos_asset_string(&amount)?;
                info!("✔ Converting action proof to `erc20-on-eos` redeem info...");
                Ok(Self {
                    amount,
                    eos_tx_amount,
                    originating_tx_id: proof.tx_id,
                    router_address: *router_address,
                    int_vault_address: *int_vault_address,
                    origin_address: proof.get_action_sender()?,
                    eos_token_address: dictionary_entry.eos_address,
                    int_token_address: dictionary_entry.eth_address,
                    user_data: Self::get_user_data_from_proof(proof)?,
                    global_sequence: proof.action_receipt.global_sequence,
                    origin_chain_id: origin_chain_id.to_metadata_chain_id(),
                    destination_address: Self::get_destination_address_from_proof(proof),
                    destination_chain_id: Self::get_destination_chain_id_from_proof(proof)?,
                })
            })
    }
}

pub fn maybe_parse_int_tx_infos_and_put_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Parsing `IntOnEosIntTxInfos` from actions data...");
    IntOnEosIntTxInfos::from_action_proofs(
        &state.action_proofs,
        state.get_eos_eth_token_dictionary()?,
        &state.eos_db_utils.get_eos_chain_id_from_db()?,
        &state.eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
        &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
    )
    .and_then(|infos| {
        info!("✔ Parsed {} `IntOnEosIntTxInfos`!", infos.len());
        state.add_int_on_eos_int_tx_infos(infos)
    })
}
