pub const PUBLIC_KEY_SIZE: usize = 33;
pub const EOS_NUM_DECIMALS: usize = 4;
pub const EOS_NAME_BYTES_LEN: usize = 8;
pub const EOS_MAX_EXPIRATION_SECS: u32 = 3600;
pub const PUBLIC_KEY_CHECKSUM_SIZE: usize = 4;
pub const EOS_ADDRESS_CHAR_LENGTH: usize = 12;
pub const EOS_TOKEN_NAME: &str = "eosio.token";
pub const EOS_TRANSFER_ACTION: &str = "transfer";
pub const PEOS_ACCOUNT_NAME: &str = "provabletokn";
pub const PEOS_ACCOUNT_ACTOR: &str = PEOS_ACCOUNT_NAME;
pub const PEOS_ACCOUNT_PERMISSION_LEVEL: &str = "active";
pub const EOS_PROVABLE_SAFE_ACCOUNT: &str = "provablesafe";
pub const PEOS_TO_EOS_MEMO: &str = "pEOS -> EOS complete!";
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
// 0xcbd29a81186afbeb3af7e170ba5aad3b41426c3e81abc7562fa321f85426c6b3
pub static EOS_CHAIN_ID_DB_KEY: [u8; 32] = [
  203, 210, 154, 129, 24, 106, 251, 235,
  58, 247, 225, 112, 186, 90, 173, 59,
  65, 66, 108, 62, 129, 171, 199, 86,
  47, 163, 33, 248, 84, 38, 198, 179
];
// NOTE (javascript): new Uint8Array(
//   Buffer.from(web3.utils.keccak256('eos-private-key-db-key').slice(2), 'hex')
// )
// 0xd2d562ddd639ba2c7de122bc75f049a968ab759be57f66449c69d5f402723571
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
