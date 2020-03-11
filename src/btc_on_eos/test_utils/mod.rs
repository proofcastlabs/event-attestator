#![cfg(test)]
pub const ROPSTEN_CONTRACT_ADDRESS: &str =
    "0x1Ee4D5f444d0Ab291D748049231dC9331b2f04C8";
pub const TEMPORARY_DATABASE_PATH: &str = "src/test_utils/temporary_database";

pub fn get_sample_message_to_sign() -> &'static str {
    "Provable pToken!"
}

pub fn get_sample_message_to_sign_bytes() -> &'static [u8] {
    get_sample_message_to_sign()
        .as_bytes()
}
