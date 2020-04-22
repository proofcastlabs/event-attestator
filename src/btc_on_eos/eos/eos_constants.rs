pub const PRODUCER_REPS: u64 = 12;
pub const PUBLIC_KEY_SIZE: usize = 33;
pub const EOS_NUM_DECIMALS: usize = 4;
pub const EOS_NAME_BYTES_LEN: usize = 8;
pub const PBTC_MINT_FXN_NAME: &str = "issue";
pub const REDEEM_ACTION_NAME: &str = "redeem";
pub const EOS_MAX_EXPIRATION_SECS: u32 = 3600;
pub const PUBLIC_KEY_CHECKSUM_SIZE: usize = 4;
pub const EOS_ADDRESS_CHAR_LENGTH: usize = 12;
pub const MEMO: &str = "BTC -> pBTC complete!";
pub const EOS_SCHEDULE_DB_PREFIX: &str = "EOS_SCHEDULE_";
pub const PEOS_ACCOUNT_PERMISSION_LEVEL: &str = "active";
pub const EOS_PROVABLE_SAFE_ACCOUNT: &str = "provablesafe";
pub const PUBLIC_KEY_WITH_CHECKSUM_SIZE: usize = PUBLIC_KEY_SIZE + PUBLIC_KEY_CHECKSUM_SIZE;
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-network-key').slice(2), 'hex')
// )
// 0x2833cc9fcbba1da54af6f047408d75277961fbd9237b49389f378bd7cde0f3fd
pub static EOS_NETWORK_KEY: [u8; 32] = [
  40, 51, 204, 159, 203, 186, 29, 165,
  74, 246, 240, 71, 64, 141, 117, 39,
  121, 97, 251, 217, 35, 123, 73, 56,
  159, 55, 139, 215, 205, 224, 243, 253
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-chain-id-key').slice(2), 'hex')
// )
// cbd29a81186afbeb3af7e170ba5aad3b41426c3e81abc7562fa321f85426c6b3
pub static EOS_CHAIN_ID_DB_KEY: [u8; 32] = [
  203, 210, 154, 129, 24, 106, 251, 235,
  58, 247, 225, 112, 186, 90, 173, 59,
  65, 66, 108, 62, 129, 171, 199, 86,
  47, 163, 33, 248, 84, 38, 198, 179
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-private-key-db-key').slice(2), 'hex')
// )
// d2d562ddd639ba2c7de122bc75f049a968ab759be57f66449c69d5f402723571
pub static EOS_PRIVATE_KEY_DB_KEY: [u8; 32] = [
  210, 213, 98, 221, 214, 57, 186, 44,
  125, 225, 34, 188, 117, 240, 73, 169,
  104, 171, 117, 155, 229, 127, 102, 68,
  156, 105, 213, 244, 2, 114, 53, 113
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-tx-ids').slice(2), 'hex')
// )
// 61b33e8588f6b6caa691d584efe8d3afadea0d16125650f85386b13e1f66e2e1
pub static PROCESSED_TX_IDS_KEY: [u8; 32] = [
  97, 179, 62, 133, 136, 246, 182, 202,
  166, 145, 213, 132, 239, 232, 211, 175,
  173, 234, 13, 22, 18, 86, 80, 248,
  83, 134, 177, 62, 31, 102, 226, 225
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-chain-id').slice(2), 'hex')
// )
// 27d6a57b5570c501ff1ce72ec96b12ad6f460a070f5cbe45651e055161ffe5dea
pub static EOS_CHAIN_ID: [u8; 32] = [
  39, 214, 165, 123, 85, 112, 197, 1,
  255, 28, 231, 46, 201, 107, 18, 173,
  111, 70, 10, 7, 15, 92, 190, 69,
  101, 30, 5, 81, 97, 255, 229, 218
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-account-name').slice(2), 'hex')
// )
// 8b9fd4b3e0a8263466a8fe52661124c424725ce71c62e0ac211f5ff022ada9a4
pub static EOS_ACCOUNT_NAME_KEY: [u8; 32] = [
  139, 159, 212, 179, 224, 168, 38, 52,
  102, 168, 254, 82, 102, 17, 36, 196,
  36, 114, 92, 231, 28, 98, 224, 172,
  33, 31, 95, 240, 34, 173, 169, 164
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-token-ticker').slice(2), 'hex')
// )
// 71c8980fe3f6e8b3cdcbd4dce5f1a13af16e1980e3a7d4a570007c24d3691271
pub static EOS_TOKEN_SYMBOL_KEY: [u8; 32] = [
  113, 200, 152, 15, 227, 246, 232, 179,
  205, 203, 212, 220, 229, 241, 161, 58,
  241, 110, 25, 128, 227, 167, 212, 165,
  112, 0, 124, 36, 211, 105, 18, 113
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-account-nonce').slice(2), 'hex')
// )
// 165307417cab4f19b70e593876098df498c34ed3d38abedfc2a908eea4feaa82
pub static EOS_ACCOUNT_NONCE: [u8; 32] = [
  22, 83, 7, 65, 124, 171, 79, 25,
  183, 14, 89, 56, 118, 9, 141, 244,
  152, 195, 78, 211, 211, 138, 190, 223,
  194, 169, 8, 238, 164, 254, 170, 130
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-schedule-list').slice(2), 'hex')
// )
// d24e8320db81859d6e8ee6fa3ed7312155e489a2e8269c4ae8a2fa32a1ac5095
pub static EOS_SCHEDULE_LIST: [u8; 32] = [
  210, 78, 131, 32, 219, 129, 133, 157,
  110, 142, 230, 250, 62, 215, 49, 33,
  85, 228, 137, 162, 232, 38, 156, 74,
  232, 162, 250, 50, 161, 172, 80, 149
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-incremerkle').slice(2), 'hex')
// )
// 9a46679091d5f3b5f56e200451de1650c1bfbd3358c23293e1decfb5ab0d0fb1
pub static EOS_INCREMERKLE: [u8; 32] = [
  154, 70, 103, 144, 145, 213, 243, 181,
  245, 110, 32, 4, 81, 222, 22, 80,
  193, 191, 189, 51, 88, 194, 50, 147,
  225, 222, 207, 181, 171, 13, 15, 177
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-last-seen-block-id').slice(2), 'hex')
// )
// 5f38e7e4da08610c7d63bd371b28581a22f90ec9564079c4e2ce4322a0b4c8c3
pub static EOS_LAST_SEEN_BLOCK_ID: [u8; 32] = [
  95, 56, 231, 228, 218, 8, 97, 12,
  125, 99, 189, 55, 27, 40, 88, 26,
  34, 249, 14, 201, 86, 64, 121, 196,
  226, 206, 67, 34, 160, 180, 200, 195
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-last-seen-block-num').slice(2), 'hex')
// )
// 1ed3e38d13ec2aecc6ba97ca94ba1336a6cafeb105a8b45265dada291f05f369
pub static EOS_LAST_SEEN_BLOCK_NUM: [u8; 32] = [
  30, 211, 227, 141, 19, 236, 42, 236,
  198, 186, 151, 202, 148, 186, 19, 54,
  166, 202, 254, 177, 5, 168, 180, 82,
  101, 218, 218, 41, 31, 5, 243, 105
];
