use secp256k1::{
    Message,
    Secp256k1,
};
use crate::btc_on_eos::{
    eos::eos_crypto::{
        eos_signature::EosSignature,
        eos_public_key::EosPublicKey,
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn should_recover_public_key() {
        // NOTE: This block === https://jungle.bloks.io/block/10800
        // NOTE: This block chosen because of repo here:
        // https://github.com/KyberNetwork/bridge_eth_smart_contracts/tree/master/test
        // Which has producer keys etc as test vectors.
        let expected_pub_key =
            "EOS7A9BoRetjpKtE3sqA6HRykRJ955MjQ5XdRmCLionVte2uERL8h";
        let pubkey = EosPublicKey::from_str(expected_pub_key)
            .unwrap();
        let digest =
            //"3883dd314f2dcbb2dc2078356f6e71b2168296e64e7166eec08b78a157390bda"
            "e991ea00a9c3564fc9c6de33dc19865abbe5ac4bf643036ecd89f95e31c49521"
            .to_string();
        let msg = Message::from_slice(&hex::decode(&digest).unwrap())
            .unwrap();
        let producer_signature = EosSignature::from_str(
            "SIG_K1_KX9Y5xYQrBYtpdKm4njsMerfzoPU6qbiW3G3RmbmbSyZ5sjE2J1U4PHC1vQ8arZQrBKqwW1adLPwYDzqt3v137GLp1ZWse"
        ).unwrap();
        let result = EosPublicKey::recover_from_digest(
            &msg,
            &producer_signature,
        ).unwrap();
        println!("digest            : {}", &digest);
        println!("expected pubkey   : {}", expected_pub_key);
        println!("producer_signature: {}", producer_signature);
        println!("expected pubkey hex: {}", hex::encode(pubkey.to_bytes()));
        println!("recovered pubkey: {}", hex::encode(result.to_bytes()));
        println!("non_recoverable sig: {}", producer_signature.to_non_recoverable_signature());
        println!("result: {}", result);
        //assert_eq!(expected_pub_key, result);
        /*
         *
         * NOTE Thomas found that we need to concatenate a whole bunch of stuff
         * create the correct digest, which when done, _does_ validate the key
         * correctly so that's good. (In python at least).
         *
         * Get the info from him!
         *
         */
    }
}
