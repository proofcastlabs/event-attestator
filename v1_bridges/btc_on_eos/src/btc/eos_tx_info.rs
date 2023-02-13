use std::str::FromStr;

use bitcoin::{
    blockdata::transaction::Transaction as BtcTransaction,
    network::constants::Network as BtcNetwork,
    util::address::Address as BtcAddress,
    Txid,
};
use common::{
    chains::btc::{
        btc_constants::{BTC_NUM_DECIMALS, MINIMUM_REQUIRED_SATOSHIS},
        btc_metadata::ToMetadata,
        deposit_address_info::DepositInfoHashMap,
    },
    constants::FEE_BASIS_POINTS_DIVISOR,
    fees::fee_utils::sanity_check_basis_points_value,
    safe_addresses::SAFE_EOS_ADDRESS_STR,
    state::BtcState,
    traits::DatabaseInterface,
    types::{Byte, Bytes, NoneError, Result},
};
use common_eos::{convert_eos_asset_to_u64, get_symbol_from_eos_asset, EosDbUtils};
use derive_more::{Constructor, Deref, DerefMut};
use eos_chain::AccountName as EosAccountName;
use serde::{Deserialize, Serialize};

use crate::utils::convert_u64_to_x_decimal_eos_asset;

#[derive(Debug, Clone, PartialEq, Default, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnEosEosTxInfos(pub Vec<BtcOnEosEosTxInfo>);

#[derive(Clone, Debug, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct BtcOnEosEosTxInfo {
    pub destination_address: String,
    pub amount: String,
    pub originating_tx_hash: String,
    pub originating_tx_address: String,
    pub user_data: Option<Bytes>,
    pub eos_token_address: String,
}

pub fn parse_eos_tx_infos_from_p2sh_deposits_and_add_to_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Parsing `BtcOnEosEosTxInfos` from `p2sh` deposit txs in state...");
    let eos_db_utils = EosDbUtils::new(state.db);
    BtcOnEosEosTxInfos::from_btc_txs(
        state.get_p2sh_deposit_txs()?,
        state.get_deposit_info_hash_map()?,
        state.btc_db_utils.get_btc_network_from_db()?,
        &eos_db_utils.get_eos_token_symbol_from_db()?,
        &eos_db_utils.get_eos_account_name_string_from_db()?,
    )
    .and_then(|eos_tx_infos| eos_tx_infos.filter_params())
    .and_then(|filtered_params| filtered_params.to_bytes())
    .map(|bytes| state.add_tx_infos(bytes))
}

impl BtcOnEosEosTxInfos {
    #[cfg(test)]
    pub fn sum(&self) -> u64 {
        self.iter()
            .map(|infos| convert_eos_asset_to_u64(&infos.amount))
            .collect::<Result<Vec<u64>>>()
            .unwrap()
            .iter()
            .sum()
    }

    pub fn subtract_fees(&self, fee_basis_points: u64) -> Result<Self> {
        self.calculate_fees(sanity_check_basis_points_value(fee_basis_points)?)
            .and_then(|(fees, _)| {
                info!("`BtcOnEosEosTxInfos` fees: {:?}", fees);
                Ok(Self::new(
                    fees.iter()
                        .zip(self.iter())
                        .map(|(fee, infos)| infos.subtract_amount(*fee))
                        .collect::<Result<Vec<BtcOnEosEosTxInfo>>>()?,
                ))
            })
    }

    pub fn calculate_fees(&self, basis_points: u64) -> Result<(Vec<u64>, u64)> {
        info!("✔ Calculating fees in `BtcOnEosEosTxInfos`...");
        let fees = self
            .iter()
            .map(|eos_tx_infos| eos_tx_infos.calculate_fee(basis_points))
            .collect::<Result<Vec<u64>>>()?;
        let total_fee = fees.iter().sum();
        info!("✔      Fees: {:?}", fees);
        info!("✔ Total fee: {:?}", fees);
        Ok((fees, total_fee))
    }

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

    pub fn filter_out_value_too_low(&self) -> Result<Self> {
        info!(
            "✔ Filtering out any `BtcOnEosEosTxInfos` below a minimum of {} Satoshis...",
            MINIMUM_REQUIRED_SATOSHIS
        );
        Ok(BtcOnEosEosTxInfos::new(
            self.iter()
                .map(|infos| convert_eos_asset_to_u64(&infos.amount))
                .collect::<Result<Vec<u64>>>()?
                .into_iter()
                .zip(self.iter())
                .filter(|(amount, infos)| match amount >= &MINIMUM_REQUIRED_SATOSHIS {
                    true => true,
                    false => {
                        info!("✘ Filtering `BtcOnEosEosTxInfos` ∵ value too low: {:?}", infos);
                        false
                    },
                })
                .map(|(_, infos)| infos)
                .cloned()
                .collect::<Vec<BtcOnEosEosTxInfo>>(),
        ))
    }

    pub fn fix_params_with_too_short_account_names(&self) -> Result<Self> {
        Ok(BtcOnEosEosTxInfos::new(
            self.iter()
                .map(|info| match info.destination_address.is_empty() {
                    false => info.clone(),
                    true => {
                        info!(
                            "✘ Redirecting destination_address safe address {:?} ∵ name too short:",
                            info
                        );
                        let mut mutable_info = info.clone();
                        mutable_info.destination_address = SAFE_EOS_ADDRESS_STR.to_string();
                        mutable_info
                    },
                })
                .collect::<Vec<BtcOnEosEosTxInfo>>(),
        ))
    }

    pub fn filter_params(&self) -> Result<Self> {
        self.fix_params_with_too_short_account_names()
            .and_then(|infos| infos.filter_out_value_too_low())
    }

    fn from_btc_tx(
        p2sh_deposit_containing_tx: &BtcTransaction,
        deposit_info_hash_map: &DepositInfoHashMap,
        btc_network: BtcNetwork,
        eos_token_symbol: &str,
        eos_token_address: &str,
    ) -> Result<BtcOnEosEosTxInfos> {
        info!("✔ Parsing `BtcOnEosEosTxInfos` from single `p2sh` transaction...");
        Ok(BtcOnEosEosTxInfos::new(
            p2sh_deposit_containing_tx
                .output
                .iter()
                .filter(|tx_out| tx_out.script_pubkey.is_p2sh())
                .map(|p2sh_tx_out| -> Option<BtcOnEosEosTxInfo> {
                    match BtcAddress::from_script(&p2sh_tx_out.script_pubkey, btc_network) {
                        Err(_) => {
                            info!(
                                "✘ Could not derive BTC address from tx: {:?}",
                                p2sh_deposit_containing_tx
                            );
                            None
                        },
                        Ok(btc_address) => {
                            info!("✔ BTC address extracted from `tx_out`: {}", btc_address);
                            match deposit_info_hash_map.get(&btc_address) {
                                None => {
                                    info!("✘ BTC address {} not in deposit hash map!", btc_address);
                                    None
                                },
                                Some(deposit_info) => {
                                    info!("✔ Deposit info extracted from hash map: {:?}", deposit_info);
                                    Some(BtcOnEosEosTxInfo::new(
                                        p2sh_tx_out.value,
                                        deposit_info.address.clone(),
                                        p2sh_deposit_containing_tx.txid(),
                                        btc_address,
                                        eos_token_symbol,
                                        if deposit_info.user_data.is_empty() {
                                            None
                                        } else {
                                            Some(deposit_info.user_data.clone())
                                        },
                                        eos_token_address,
                                    ))
                                },
                            }
                        },
                    }
                })
                .filter(|maybe_eos_tx_infos| maybe_eos_tx_infos.is_some())
                .map(|maybe_eos_tx_infos| maybe_eos_tx_infos.ok_or(NoneError("Could not unwrap `BtcOnEosEosTxInfos`!")))
                .collect::<Result<Vec<BtcOnEosEosTxInfo>>>()?,
        ))
    }

    pub fn from_btc_txs(
        p2sh_deposit_containing_txs: &[BtcTransaction],
        deposit_info_hash_map: &DepositInfoHashMap,
        btc_network: BtcNetwork,
        eos_token_symbol: &str,
        eos_token_address: &str,
    ) -> Result<BtcOnEosEosTxInfos> {
        info!("✔ Parsing `BtcOnEosEosTxInfos` from `p2sh` transactions...");
        Ok(Self::new(
            p2sh_deposit_containing_txs
                .iter()
                .flat_map(|tx| {
                    Self::from_btc_tx(
                        tx,
                        deposit_info_hash_map,
                        btc_network,
                        eos_token_symbol,
                        eos_token_address,
                    )
                })
                .flat_map(|eos_tx_infos| eos_tx_infos.0)
                .collect::<Vec<BtcOnEosEosTxInfo>>(),
        ))
    }
}

impl BtcOnEosEosTxInfo {
    pub fn calculate_fee(&self, basis_points: u64) -> Result<u64> {
        convert_eos_asset_to_u64(&self.amount).map(|amount| (amount * basis_points) / FEE_BASIS_POINTS_DIVISOR)
    }

    pub fn subtract_amount(&self, subtrahend: u64) -> Result<Self> {
        info!("✔ Subtracting {} from `BtcOnEosEosTxInfo`...", subtrahend);
        let symbol = get_symbol_from_eos_asset(&self.amount);
        let amount = convert_eos_asset_to_u64(&self.amount)?;
        if subtrahend > amount {
            Err(format!("Cannot subtract {} from {}!", subtrahend, amount).into())
        } else {
            let amount_minus_fee = amount - subtrahend;
            info!(
                "✔ Subtracted amount of {} from current `BtcOnEosEosTxInfos` amount of {} destination_address get final amount of {}",
                subtrahend, amount, amount_minus_fee
            );
            let mut new_self = self.clone();
            new_self.amount = convert_u64_to_x_decimal_eos_asset(amount_minus_fee, BTC_NUM_DECIMALS, symbol);
            Ok(new_self)
        }
    }

    pub fn new(
        amount: u64,
        destination_address: String,
        originating_tx_hash: Txid,
        originating_tx_address: BtcAddress,
        symbol: &str,
        user_data: Option<Bytes>,
        eos_token_address: &str,
    ) -> BtcOnEosEosTxInfo {
        BtcOnEosEosTxInfo {
            user_data,
            destination_address: match EosAccountName::from_str(&destination_address) {
                Ok(_) => destination_address,
                Err(_) => {
                    info!(
                        "✘ Error converting '{}' destination_address EOS address!",
                        destination_address
                    );
                    info!(
                        "✔ Defaulting destination_address safe EOS address: '{}'",
                        SAFE_EOS_ADDRESS_STR
                    );
                    SAFE_EOS_ADDRESS_STR.to_string()
                },
            },
            originating_tx_hash: originating_tx_hash.to_string(),
            amount: convert_u64_to_x_decimal_eos_asset(amount, BTC_NUM_DECIMALS, symbol),
            originating_tx_address: originating_tx_address.to_string(),
            eos_token_address: eos_token_address.to_string(),
        }
    }
}

impl ToMetadata for BtcOnEosEosTxInfo {
    fn get_user_data(&self) -> Option<Bytes> {
        self.user_data.clone()
    }

    fn get_originating_tx_address(&self) -> String {
        self.originating_tx_address.clone()
    }
}

#[cfg(test)]
mod tests {
    use common::{
        chains::btc::btc_chain_id::BtcChainId,
        errors::AppError,
        metadata::metadata_protocol_id::MetadataProtocolId,
    };
    use common_eos::MAX_BYTES_FOR_EOS_USER_DATA;

    use super::*;
    use crate::test_utils::get_sample_btc_on_eos_eos_tx_infos;

    #[test]
    fn should_filter_eos_tx_infos() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let eos_tx_infos = get_sample_btc_on_eos_eos_tx_infos();
        let length_before = eos_tx_infos.len();
        assert_eq!(length_before, expected_length_before);
        let result = eos_tx_infos.filter_out_value_too_low().unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result
            .iter()
            .for_each(|infos| assert!(convert_eos_asset_to_u64(&infos.amount).unwrap() >= MINIMUM_REQUIRED_SATOSHIS));
    }

    #[test]
    fn should_subtract_amount_from_btc_on_eos_eos_tx_infos() {
        let infos = get_sample_btc_on_eos_eos_tx_infos()[0].clone();
        let subtrahend = 1337;
        let result = infos.subtract_amount(subtrahend).unwrap();
        let expected_result = "0.00003663 PBTC".to_string();
        assert_eq!(result.destination_address, infos.destination_address);
        assert_eq!(result.originating_tx_hash, infos.originating_tx_hash);
        assert_eq!(result.originating_tx_address, infos.originating_tx_address);
        assert_eq!(result.amount, expected_result);
    }

    #[test]
    fn should_calculate_fee_from_btc_on_eos_eos_tx_info() {
        let infos = get_sample_btc_on_eos_eos_tx_infos()[0].clone();
        let basis_points = 25;
        let result = infos.calculate_fee(basis_points).unwrap();
        let expected_result = 12;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fee_from_btc_on_eos_eos_tx_infos() {
        let infos = get_sample_btc_on_eos_eos_tx_infos();
        let basis_points = 25;
        let (fees, total) = infos.calculate_fees(basis_points).unwrap();
        let expected_fees = vec![12, 12, 12];
        let expected_total: u64 = expected_fees.iter().sum();
        assert_eq!(total, expected_total);
        assert_eq!(fees, expected_fees);
    }

    #[test]
    fn should_subtract_fees_from_btc_on_eos_eos_tx_infos() {
        let infos = get_sample_btc_on_eos_eos_tx_infos();
        let basis_points = 25;
        let result = infos.subtract_fees(basis_points).unwrap();
        let expected_amount_0 = 4988;
        let expected_amount_1 = 4989;
        assert_eq!(convert_eos_asset_to_u64(&result[0].amount).unwrap(), expected_amount_0);
        assert_eq!(convert_eos_asset_to_u64(&result[1].amount).unwrap(), expected_amount_1);
    }

    #[test]
    fn should_fail_to_subtact_too_large_an_amount_from_btc_on_eos_eos_tx_infos() {
        let infos = get_sample_btc_on_eos_eos_tx_infos()[0].clone();
        let amount = convert_eos_asset_to_u64(&infos.amount).unwrap();
        let subtrahend = amount + 1;
        let expected_err = format!("Cannot subtract {} from {}!", subtrahend, amount);
        match infos.subtract_amount(subtrahend) {
            Ok(_) => panic!("Should not have suceeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error received!"),
        };
    }

    #[test]
    fn should_serde_btc_on_eos_eos_tx_infos() {
        let eos_tx_infos = get_sample_btc_on_eos_eos_tx_infos();
        let bytes = eos_tx_infos.to_bytes().unwrap();
        let result = BtcOnEosEosTxInfos::from_bytes(&bytes).unwrap();
        assert_eq!(result, eos_tx_infos);
    }

    #[test]
    fn should_convert_btc_on_eos_eos_tx_infos_to_metadata_bytes() {
        let mut eos_tx_info_stuct = get_sample_btc_on_eos_eos_tx_infos()[0].clone();
        eos_tx_info_stuct.user_data = Some(hex::decode("d3caffc0ff33").unwrap());
        let expected_result = Some(hex::decode("0106d3caffc0ff330401ec97de4630783331333333363433353434353532363136663633366433383634346336323435373437613433363134363734346134613538333936613636343636383665343336383462").unwrap());
        let btc_chain_id = BtcChainId::Bitcoin;
        let destination_protocol_id = MetadataProtocolId::Eos;
        let result = eos_tx_info_stuct
            .maybe_to_metadata_bytes(&btc_chain_id, MAX_BYTES_FOR_EOS_USER_DATA, &destination_protocol_id)
            .unwrap();
        assert_eq!(result, expected_result);
    }
}
