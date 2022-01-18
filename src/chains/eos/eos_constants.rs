#[cfg(test)]
pub const EOS_ADDRESS_LENGTH_IN_BYTES: usize = 8;

pub const MEMO: &str = "";
pub const PRODUCER_REPS: u64 = 12;
pub const PUBLIC_KEY_SIZE: usize = 33;
pub const PEGIN_ACTION_NAME: &str = "pegin";
pub const PEGOUT_ACTION_NAME: &str = "pegout";
pub const REDEEM_ACTION_NAME: &str = "redeem";
pub const PUBLIC_KEY_CHECKSUM_SIZE: usize = 4;
pub const MAX_BYTES_FOR_EOS_USER_DATA: usize = 2000;
pub const EOS_SCHEDULE_DB_PREFIX: &str = "EOS_SCHEDULE_";
pub const EOS_ACCOUNT_PERMISSION_LEVEL: &str = "active";
pub const EOS_CORE_IS_INITIALIZED_JSON: &str = "{eos_core_initialized:true}";
pub const PUBLIC_KEY_WITH_CHECKSUM_SIZE: usize = PUBLIC_KEY_SIZE + PUBLIC_KEY_CHECKSUM_SIZE;
// NOTE: We use 59 minutes rather than 60 to give a little wiggle room for the clocks on the TEE devices.
pub const EOS_MAX_EXPIRATION_SECS: u32 = 3540;
