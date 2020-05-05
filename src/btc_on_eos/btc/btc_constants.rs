#[cfg(not(test))]
pub const PTOKEN_P2SH_SCRIPT_BYTES: usize = 101;

#[cfg(test)] // NOTE Because of real BTC tx test-vectors
pub const PTOKEN_P2SH_SCRIPT_BYTES: usize = 0;

pub const BTC_TAIL_LENGTH: u64 = 10;
pub const DEFAULT_BTC_SEQUENCE: u32 = 4_294_967_295; // NOTE: 0xFFFFFFFF

// NOTE: Keccak256 hash of: 'btc-on-eos-btc-difficulty'
// 84f5182962ac8586f52ed82dd741b60277a36fa62bd712b2edb293aa0d299a8a
pub static BTC_DIFFICULTY_THRESHOLD: [u8; 32] = [
     132, 245, 24, 41, 98, 172, 133, 134,
     245, 46, 216, 45, 215, 65, 182, 2,
     119, 163, 111, 166, 43, 215, 18, 178,
     237, 178, 147, 170, 13, 41, 154, 138,

];
// NOTE: Keccak256 hash of: 'btc-on-eos-btc-address-key'
// 3c7be3aeea8a5e50c65b1d915e9f4134e81ea00150abca6af6a7f93342b20a70
pub static BTC_ADDRESS_KEY: [u8; 32] = [
     60, 123, 227, 174, 234, 138, 94, 80,
     198, 91, 29, 145, 94, 159, 65, 52,
     232, 30, 160, 1, 80, 171, 202, 106,
     246, 167, 249, 51, 66, 178, 10, 112,
];
// NOTE: Keccak256 hash of: 'btc-on-eos-btc-canon-block'
// 3b9f77d819a54fa0efec77baa53e4d1f3f578a9489668b9b0400324f23298efb
pub static BTC_CANON_BLOCK_HASH_KEY: [u8; 32] = [
     59, 159, 119, 216, 25, 165, 79, 160,
     239, 236, 119, 186, 165, 62, 77, 31,
     63, 87, 138, 148, 137, 102, 139, 155,
     4, 0, 50, 79, 35, 41, 142, 251,
];
// NOTE: Keccak256 hash of: 'btc-on-eos-btc-latest-block'
// 1ffbbc434bbd4c52f1399e39ac2e98bdeb8ecfaacabd0ac1de507a76a23dc001
pub static BTC_LATEST_BLOCK_HASH_KEY: [u8; 32] = [
     31, 251, 188, 67, 75, 189, 76, 82,
     241, 57, 158, 57, 172, 46, 152, 189,
     235, 142, 207, 170, 202, 189, 10, 193,
     222, 80, 122, 118, 162, 61, 192, 1,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-linker-hash'
// 5bce7b2fef3886cf00bbb255290d2ca2272b3846fd82c70eed836b295ebf2629
pub static BTC_LINKER_HASH_KEY: [u8; 32] = [
     91, 206, 123, 47, 239, 56, 134, 207,
     0, 187, 178, 85, 41, 13, 44, 162,
     39, 43, 56, 70, 253, 130, 199, 14,
     237, 131, 107, 41, 94, 191, 38, 41,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-anchor-block'
// c39cc52cb0f285effaa18b1e5875dde3eb0c7119731b7655a56d32fc3d5c23bc
pub static BTC_ANCHOR_BLOCK_HASH_KEY: [u8; 32] = [
     195, 156, 197, 44, 176, 242, 133, 239,
     250, 161, 139, 30, 88, 117, 221, 227,
     235, 12, 113, 25, 115, 27, 118, 85,
     165, 109, 50, 252, 61, 92, 35, 188,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-private-key'
// 0904995590eae77780c1fd1644a2a28d58ea0baa2c4ca99a57b497b71ea78428
pub static BTC_PRIVATE_KEY_DB_KEY: [u8; 32] = [
     9, 4, 153, 85, 144, 234, 231, 119,
     128, 193, 253, 22, 68, 162, 162, 141,
     88, 234, 11, 170, 44, 76, 169, 154,
     87, 180, 151, 183, 30, 167, 132, 40,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-canon-to-tip-length'
// f9ffc7ca216da72306045a02d983cfece50587c0a0fa5261725035a054e53034
pub static BTC_CANON_TO_TIP_LENGTH_KEY: [u8; 32] = [
     249, 255, 199, 202, 33, 109, 167, 35,
     6, 4, 90, 2, 217, 131, 207, 236,
     229, 5, 135, 192, 160, 250, 82, 97,
     114, 80, 53, 160, 84, 229, 48, 52,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-provable-ptoken'
// a2addefebfcd68ac908f89facd59dbc12124bc295e0bfc76e15ff2063aa8cd89
pub static PTOKEN_GENESIS_HASH: [u8; 32] = [
     162, 173, 222, 254, 191, 205, 104, 172,
     144, 143, 137, 250, 205, 89, 219, 193,
     33, 36, 188, 41, 94, 11, 252, 118,
     225, 95, 242, 6, 58, 168, 205, 137,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-network-key'
// 9645ae91e6564b91f128fd8fa100f77280bdb0d3f4b4833730ab964fdaf2c782
pub static BTC_NETWORK_KEY: [u8; 32] = [
     150, 69, 174, 145, 230, 86, 75, 145,
     241, 40, 253, 143, 161, 0, 247, 114,
     128, 189, 176, 211, 244, 180, 131, 55,
     48, 171, 150, 79, 218, 242, 199, 130,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-fee-key'
// a7f7862e056c18f0dc8851802e89b5e15735d1e0d4ba81b36ac35bc27f2d2d1a
pub static BTC_FEE_KEY: [u8; 32] = [
     167, 247, 134, 46, 5, 108, 24, 240,
     220, 136, 81, 128, 46, 137, 181, 225,
     87, 53, 209, 224, 212, 186, 129, 179,
     106, 195, 91, 194, 127, 45, 45, 26,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-account-nonce-key'
// 192da73fcc968b8022a3bd80873b1dbace075a50ef53ddf5059e7459a459f3e1
pub static BTC_ACCOUNT_NONCE_KEY: [u8; 32] = [
     25, 45, 167, 63, 204, 150, 139, 128,
     34, 163, 189, 128, 135, 59, 29, 186,
     206, 7, 90, 80, 239, 83, 221, 245,
     5, 158, 116, 89, 164, 89, 243, 225,
];
// NOTE: Keccak256 hash of:: 'btc-on-eos-btc-tail-block-hash-key'
// bc6cdffc2b733d2758db92cab0b704c8d7a523bcdec8fa612d055b6edf4aacc4
pub static BTC_TAIL_BLOCK_HASH_KEY: [u8; 32] = [
     188, 108, 223, 252, 43, 115, 61, 39,
     88, 219, 146, 202, 176, 183, 4, 200,
     215, 165, 35, 188, 222, 200, 250, 97,
     45, 5, 91, 110, 223, 74, 172, 196,
];
