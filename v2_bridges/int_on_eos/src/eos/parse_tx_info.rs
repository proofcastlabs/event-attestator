use std::str::FromStr;

use common::{
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::convert_bytes_to_u64,
};
use common_chain_ids::EosChainId;
use common_eos::{EosActionProof, EosState};
use common_eth::{EthDbUtils, EthDbUtilsExt};
use common_metadata::MetadataChainId;
use ethereum_types::{Address as EthAddress, U256};

use crate::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos};

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
                let eos_address = dictionary_entry.eos_address.clone();
                let amount = Self::get_redeem_amount_from_proof(proof, &dictionary_entry)?;
                let eos_tx_amount = dictionary_entry.convert_u256_to_eos_asset_string(&amount)?;

                let destination_chain_id = if proof.is_v2_redeem() {
                    Self::get_destination_chain_id_from_proof(proof)
                } else {
                    // NOTE: If in future we need more granular control here, we can also match on
                    // the eos addresses in the dictionary. That would however require hard coding
                    // addresses which is less than ideal.
                    match origin_chain_id {
                        EosChainId::UltraMainnet => {
                            // NOTE: Ultra currently has some restrictions meaning `redeem2` actions cannot be used
                            // when upgrading from a v1 bridge. Instead, we listen for _both_ v1 and v2 actions in
                            // here, and in the case of the former, we default to ETH mainnet as the destination.
                            warn!("ULTRA v1 redeem detected, defaulting to ETH mainnet as destination chain ID");
                            Ok(MetadataChainId::EthereumMainnet)
                        },
                        EosChainId::EosMainnet => {
                            match eos_address.as_ref() {
                                "wmbt.ptokens" => {
                                    warn!("pWOMBAT on EOS v1 redeem action detected, using ETH mainnet as destination chain ID");
                                    Ok(MetadataChainId::EthereumMainnet)
                                },
                                "btc.ptokens" => {
                                    warn!("pBTC on EOS v1 redeem action detected, using BTC mainnet as destination chain ID");
                                    Ok(MetadataChainId::BitcoinMainnet)
                                },
                                _ => Err(format!("cannot handle EOS v1 redeem action from address: {eos_address}").into()),
                            }
                        },
                        _ => Err(format!("cannot handle v1 redeem action from origin chain: {origin_chain_id}").into()),
                    }
                }?;

                info!("✔ Converting action proof to `erc20-on-eos` redeem info...");
                Ok(Self {
                    amount,
                    eos_tx_amount,
                    destination_chain_id,
                    eos_token_address: eos_address,
                    originating_tx_id: proof.tx_id,
                    router_address: *router_address,
                    int_vault_address: *int_vault_address,
                    origin_address: proof.get_action_sender()?,
                    int_token_address: dictionary_entry.eth_address,
                    user_data: Self::get_user_data_from_proof(proof)?,
                    global_sequence: proof.action_receipt.global_sequence,
                    destination_address: Self::get_destination_address_from_proof(proof),
                    origin_chain_id: MetadataChainId::from_str(&origin_chain_id.to_string())?,
                })
            })
    }
}

pub fn maybe_parse_int_tx_infos_and_put_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Parsing `IntOnEosIntTxInfos` from actions data...");
    let eth_db_utils = EthDbUtils::new(state.db);
    IntOnEosIntTxInfos::from_action_proofs(
        &state.action_proofs,
        state.get_eos_eth_token_dictionary()?,
        &state.eos_db_utils.get_eos_chain_id_from_db()?,
        &eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
        &eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
    )
    .and_then(|infos| {
        let global_sequences = infos.get_global_sequences();
        info!("✔ Parsed {} `IntOnEosIntTxInfos`!", infos.len());
        let bytes = infos.to_bytes()?;
        Ok(state.add_tx_infos(bytes).add_global_sequences(global_sequences))
    })
}
