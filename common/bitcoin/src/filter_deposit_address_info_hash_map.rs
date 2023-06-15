use common::types::Result;

use crate::{
    deposit_address_info::{DepositAddressInfo, DepositAddressInfoVersion, DepositInfoHashMap},
    get_deposit_info_hash_map::create_hash_map_from_deposit_info_list,
};

pub fn filter_deposit_infos_for_allowed_versions(
    allowed_versions: &[DepositAddressInfoVersion],
    desposit_info_hash_map: &DepositInfoHashMap,
) -> Result<DepositInfoHashMap> {
    create_hash_map_from_deposit_info_list(
        &desposit_info_hash_map
            .iter()
            .filter(|(_, deposit_info)| {
                if allowed_versions.contains(&deposit_info.version) {
                    true
                } else {
                    info!(
                        "✘ Filtering out deposit info for address '{}' because it's version is disallowed!",
                        deposit_info.address
                    );
                    false
                }
            })
            .map(|(_, deposit_info)| deposit_info)
            .cloned()
            .collect::<Vec<DepositAddressInfo>>(),
    )
}

#[cfg(all(test, not(feature = "ltc")))]
mod test {
    use serde_json::json;

    use super::*;
    use crate::get_deposit_info_hash_map::create_hash_map_from_deposit_info_list;

    #[test]
    fn should_filter_deposit_info_hash_map_for_allowed_versions() {
        let deposit_info_v0 = DepositAddressInfo::from_str(
            &json!({
              "btc_deposit_address": "3ENRCnakWpTGakLRXCjiGuYrhPFrtjUzo8",
              "eth_address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
              "eth_address_and_nonce_hash": "0xdaeca04a96476cf04ec497f470b50fdf1e5bbb1334fe7db0573f49b1025f5455",
              "nonce": 1645556894,
              "public_key": "03fd539c728597e774040bda920ea7112257422442dcd7d9fc12e04e578e0af91a",
              "tool_version": "1.9.0",
              "version": "0"
            })
            .to_string(),
        )
        .unwrap();
        let deposit_info_v1 = DepositAddressInfo::from_str(
            &json!({
              "address": "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC",
              "address_and_nonce_hash": "0xe885056d57d97bfcc93ec47317dc17fe2da3e1268569a5f083a4e1fab9f8ecde",
              "btc_deposit_address": "39qNycjgTyTJPYeWT7QFQbKML84RTYYqyT",
              "nonce": 1645556828,
              "public_key": "03fd539c728597e774040bda920ea7112257422442dcd7d9fc12e04e578e0af91a",
              "tool_version": "1.9.0",
              "version": "1"
            })
            .to_string(),
        )
        .unwrap();
        let deposit_info_v2 = DepositAddressInfo::from_str(
            &json!({
              "address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
              "address_and_nonce_hash": "0x1e3645520c612ca5e4a7dab5239a80fe07a77e02ab77bb5f5419f0b65a8293ce",
              "btc_deposit_address": "2N8UazaKCgdRdXPcccsTEFuGcW5CpHwyBZW",
              "chain_id": "EthereumRopsten",
              "chain_id_hex": "0x0069c322",
              "nonce": 1645460600,
              "public_key": "03fd539c728597e774040bda920ea7112257422442dcd7d9fc12e04e578e0af91a",
              "tool_version": "1.9.0",
              "user_data": "0x",
              "version": "2"
            })
            .to_string(),
        )
        .unwrap();
        let deposit_info_v3 = DepositAddressInfo::from_str(
            &json!({
              "address": "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC",
              "address_and_nonce_hash": "0x0da764696a160c2dc197a5d9cf258df7e19416d537965c74c523a2a3128bec81",
              "btc_deposit_address": "2NAohu4wa7CBRzVqdCJZd3RyVe9Wv9PxkUJ",
              "chain_id": "EthereumRopsten",
              "chain_id_hex": "0x0069c322",
              "nonce": 1645460600,
              "public_key": "03fd539c728597e774040bda920ea7112257422442dcd7d9fc12e04e578e0af91a",
              "tool_version": "1.9.0",
              "user_data": "0x",
              "version": "3"
            })
            .to_string(),
        )
        .unwrap();
        let map = create_hash_map_from_deposit_info_list(&[
            deposit_info_v0.clone(),
            deposit_info_v1.clone(),
            deposit_info_v2.clone(),
            deposit_info_v3.clone(),
        ])
        .unwrap();
        assert_eq!(map.len(), 4);
        let allowed_versions = vec![DepositAddressInfoVersion::V1, DepositAddressInfoVersion::V3];
        let result = filter_deposit_infos_for_allowed_versions(&allowed_versions, &map).unwrap();
        assert_eq!(result.len(), map.len() - allowed_versions.len());
        let result_infos = result.into_values().collect::<Vec<DepositAddressInfo>>();
        assert!(!result_infos.contains(&deposit_info_v0));
        assert!(result_infos.contains(&deposit_info_v1));
        assert!(!result_infos.contains(&deposit_info_v2));
        assert!(result_infos.contains(&deposit_info_v3));
    }
}
