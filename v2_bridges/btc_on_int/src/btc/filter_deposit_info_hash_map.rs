use common::{traits::DatabaseInterface, types::Result};
use common_btc::{filter_deposit_infos_for_allowed_versions, BtcState, DepositAddressInfoVersion};

pub fn filter_out_wrong_version_deposit_address_infos<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking deposit infos are sufficient version...");
    filter_deposit_infos_for_allowed_versions(&[DepositAddressInfoVersion::V3], state.get_deposit_info_hash_map()?)
        .and_then(|filtered_map| state.update_deposit_info_hash_map(filtered_map))
}
