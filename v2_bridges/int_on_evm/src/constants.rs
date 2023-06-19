use common::core_type::CoreType;
use common_eth::convert_hex_to_eth_address;
use ethereum_types::Address as EthAddress;

pub const CORE_TYPE: CoreType = CoreType::IntOnEvm;

lazy_static! {
    // NOTE: The v1 pTelos token contract on ETH is NOT upgradeable and thus requires special handling.
    pub static ref PTLOS_ADDRESS: EthAddress = {
        convert_hex_to_eth_address("0x7825e833d495f3d1c28872415a4aee339d26ac88").expect("this not to fail")
    };
    // NOTE: The v1 pLTC token contract on ETH is NOT upgradeable and thus requires special handling.
    pub static ref PLTC_ADDRESS: EthAddress = {
        convert_hex_to_eth_address("0x5979f50f1d4c08f9a53863c2f39a7b0492c38d0f").expect("this not to fail")
    };
}
