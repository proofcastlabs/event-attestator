use eos_primitives::Checksum256;
use bitcoin_hashes::{
    Hash,
    sha256,
};
use crate::btc_on_eos::{
    errors::AppError,
    types::{
        Byte,
        Bytes,
        Result,
    },
    eos::eos_types::{
        MerklePath,
        MerkleProof,
    },
};

pub type CanonicalLeft = Bytes;
pub type CanonicalRight = Bytes;
pub type Sha256Hash = bitcoin_hashes::sha256::Hash;
pub type CanonicalPair = (CanonicalLeft, CanonicalRight);

fn set_first_bit_of_byte_to_zero(mut byte: Byte) -> Byte {
    byte &= 0b0111_1111;
    byte
}

fn set_first_bit_of_byte_to_one(mut byte: Byte) -> Byte {
    byte |= 0b1000_0000;
    byte
}

fn set_first_bit_of_hash_to_one(hash: &Bytes) -> Bytes {
    let mut new_hash = hash.clone();
    new_hash[0] = set_first_bit_of_byte_to_one(hash[0]);
    new_hash
}

fn set_first_bit_of_hash_to_zero(hash: &Bytes) -> Bytes {
    let mut new_hash = hash.clone();
    new_hash[0] = set_first_bit_of_byte_to_zero(hash[0]);
    new_hash
}

fn make_canonical_left(hash: &Bytes) -> CanonicalLeft {
    set_first_bit_of_hash_to_zero(hash)
}

fn make_canonical_right(hash: &Bytes) -> CanonicalRight {
    set_first_bit_of_hash_to_one(hash)
}

fn is_canonical_left(hash: &Bytes) -> bool {
    hash[0] & 0b1000_0000 == 0
}

fn is_canonical_right(hash: &Bytes) -> bool {
    !is_canonical_left(hash)
}

fn make_canonical_pair(l: &Bytes, r: &Bytes) -> CanonicalPair {
    (
        make_canonical_left(l),
        make_canonical_right(r),
    )
}

fn concatenate_canonical_pair(mut pair: CanonicalPair) -> Bytes {
    pair.0.append(& mut pair.1);
    pair.0
}

fn hash_canonical_pair(pair: CanonicalPair) -> Sha256Hash {
    sha256::Hash::hash(&concatenate_canonical_pair(pair))
}

fn make_and_hash_canonical_pair(l: &Bytes, r: &Bytes) -> Bytes {
    hash_canonical_pair(make_canonical_pair(l, r)).to_vec()
}

pub fn get_merkle_digest(mut leaves: Vec<Bytes>) -> Bytes {
    if leaves.len() == 0 {
        return vec![0x00]
    }
    while leaves.len() > 1 {
        if leaves.len() % 2 != 0 {
            let last = leaves[leaves.len() - 1].clone();
            leaves.push(last);
        }
        for i in 0..(leaves.len() / 2) {
            leaves[i] = hash_canonical_pair(
                make_canonical_pair(&leaves[2 * i], &leaves[(2 * i) + 1])
            ).to_vec();
        }
        leaves.resize(leaves.len() / 2, vec![0x00]);
    }
    leaves[0].clone()
}

pub fn verify_merkle_proof(merkle_proof: &MerkleProof) -> Result<bool> {
    let mut node = hex::decode(merkle_proof[0].clone())?;
    let leaves = merkle_proof[..merkle_proof.len() - 1]
        .iter()
        .map(|hex| Ok(hex::decode(hex)?))
        .collect::<Result<Vec<Bytes>>>()?;
    for i in 1..leaves.len() {
        match is_canonical_right(&leaves[i]) {
            true => {node = make_and_hash_canonical_pair(&node, &leaves[i]);}
            false => {node = make_and_hash_canonical_pair(&leaves[i], &node);}
        }
    };
    Ok(node == hex::decode(merkle_proof.last()?)?)
}

pub fn get_merkle_root_from_merkle_path(
    merkle_proof: &MerklePath,
) -> Result<Bytes> {
    let mut node = merkle_proof[0].clone();
    Ok(
        merkle_proof[1..]
            .iter()
            .map(|leaf| {
                match is_canonical_right(&leaf) {
                    true => node = make_and_hash_canonical_pair(&node, &leaf),
                    false => node = make_and_hash_canonical_pair(&leaf, &node),
                };
                node.clone()
            })
            .collect::<Vec<Bytes>>()
            .last()?
            .to_vec()
    )
}

// NOTE: Courtesy of: https://github.com/bifrost-codes/rust-eos/
#[derive(Clone, Default, Debug, PartialEq)]
pub struct IncrementalMerkle {
    _node_count: u64,
    _active_nodes: Vec<Checksum256>,
}
// NOTE: Ibid
impl IncrementalMerkle {

    fn make_canonical_left(val: &Checksum256) -> Checksum256 {
        let mut canonical_l: Checksum256 = *val;
        canonical_l.set_hash0(canonical_l.hash0() & 0xFFFFFFFFFFFFFF7Fu64);
        canonical_l
    }

    fn  make_canonical_right(val: &Checksum256) -> Checksum256 {
        let mut canonical_r: Checksum256 = *val;
        canonical_r.set_hash0(canonical_r.hash0() | 0x0000000000000080u64);
        canonical_r
    }

    pub fn make_canonical_pair(
        l: &Checksum256,
        r: &Checksum256
    ) -> (Checksum256, Checksum256) {
        (
            Self::make_canonical_left(l),
            Self::make_canonical_right(r)
        )
    }

    fn next_power_of_2(mut value: u64) -> u64 {
        value -= 1;
        value |= value >> 1;
        value |= value >> 2;
        value |= value >> 4;
        value |= value >> 8;
        value |= value >> 16;
        value |= value >> 32;
        value += 1;
        value
    }

    fn clz_power_2(value: u64) -> usize {
        let mut lz: usize = 64;

        if value != 0 { lz -= 1; }
        if (value & 0x00000000FFFFFFFF_u64) != 0 { lz -= 32; }
        if (value & 0x0000FFFF0000FFFF_u64) != 0 { lz -= 16; }
        if (value & 0x00FF00FF00FF00FF_u64) != 0 { lz -= 8; }
        if (value & 0x0F0F0F0F0F0F0F0F_u64) != 0 { lz -= 4; }
        if (value & 0x3333333333333333_u64) != 0 { lz -= 2; }
        if (value & 0x5555555555555555_u64) != 0 { lz -= 1; }

        lz
    }

    fn calculate_max_depth(node_count: u64) -> usize {
        if node_count == 0 {
            return 0;
        }
        let implied_count = Self::next_power_of_2(node_count);
        Self::clz_power_2(implied_count) + 1
    }

    pub fn new(node_count: u64, active_nodes: Vec<Checksum256>) -> Self {
        IncrementalMerkle {
            _node_count: node_count,
            _active_nodes: active_nodes,
        }
    }

    pub fn append(&mut self, digest: Checksum256) -> Result<Checksum256> {
        let mut partial = false;
        let max_depth = Self::calculate_max_depth(self._node_count + 1);
        let mut current_depth = max_depth - 1;
        let mut index = self._node_count;
        let mut top = digest;
        let mut active_iter = self._active_nodes.iter();
        let mut updated_active_nodes: Vec<Checksum256> = Vec::with_capacity(
            max_depth
        );

        while current_depth > 0 {
            if (index & 0x1) == 0 {
                if !partial {
                    updated_active_nodes.push(top);
                }

                top = Checksum256::hash(
                    Self::make_canonical_pair(&top, &top)
                )?;
                partial = true;
            } else {
                let left_value = active_iter.next().ok_or(
                    AppError::Custom("✘ Incremerkle error!".to_string())
                )?;

                if partial {
                    updated_active_nodes.push(*left_value);
                }

                top = Checksum256::hash(
                    Self::make_canonical_pair(left_value, &top)
                )?;
            }

            current_depth -= 1;
            index = index >> 1;
        }

        updated_active_nodes.push(top);

        self._active_nodes = updated_active_nodes;

        self._node_count += 1;

        return Ok(self._active_nodes[self._active_nodes.len() - 1]);
    }

    pub fn get_root(&self) -> Checksum256 {
        if self._node_count > 0 {
            return self._active_nodes[self._active_nodes.len() - 1];
        } else {
            return Default::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use hex;
    use super::*;
    use std::str::FromStr;
    use crate::btc_on_eos::{
        eos::eos_test_utils::get_sample_eos_submission_material_n,
    };
    use eos_primitives::{
        Action,
        ActionName,
        AccountName,
        AuthSequence,
        ActionReceipt,
        SerializeData,
        PermissionName,
        PermissionLevel,
    };
    use crate::btc_on_eos::eos::eos_test_utils::get_sample_action_digests;

    fn get_expected_digest_1() -> &'static str {
        "9b9babebfbdff48ce4002b5f3c7f999c0ee74707b6d121c47ef5db68c6be7262"
    }

    fn get_expected_digest_2() -> &'static str {
        "122cd09d66ca7df007a35bd9c9be5484833f1a69ad0c8527c3e2a56b6955e761"
    }

    fn get_expected_digest_bytes_1() -> Bytes {
        hex::decode(get_expected_digest_1()).unwrap()
    }

    fn get_expected_digest_bytes_2() -> Bytes {
        hex::decode(get_expected_digest_2()).unwrap()
    }

    fn get_expected_first_byte_1() -> Byte {
        0b0001_1011
    }

    fn get_expected_first_byte_2() -> Byte {
        0b1001_0010
    }

    fn get_sample_canonical_pair() -> CanonicalPair {
        make_canonical_pair(
            &get_expected_digest_bytes_1(),
            &get_expected_digest_bytes_2(),
        )
    }

    #[test]
    fn should_set_first_bit_of_byte_to_zero() {
        let byte = 0b1011_1011;
        let expected_result = 0b0011_1011;
        let result = set_first_bit_of_byte_to_zero(byte);
        assert!(result == expected_result);
    }

    #[test]
    fn should_set_first_bit_of_byte_to_one() {
        let byte = 0b0011_0011;
        let expected_result = 0b1011_0011;
        let result = set_first_bit_of_byte_to_one(byte);
        assert!(result == expected_result);
    }

    #[test]
    fn should_set_first_bit_of_hash_to_one() {
        let hash = get_expected_digest_bytes_2();
        let result = set_first_bit_of_hash_to_one(&hash);
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_2());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn should_set_first_bit_of_hash_to_zero() {
        let hash = get_expected_digest_bytes_1();
        let result = set_first_bit_of_hash_to_zero(&hash);
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_1());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn should_make_hash_canonical_right() {
        let hash = get_expected_digest_bytes_2();
        let result = make_canonical_right(&hash);
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_2());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn should_make_hash_canonical_left() {
        let hash = get_expected_digest_bytes_1();
        let result = make_canonical_left(&hash);
        for i in 0..hash.len() {
            if i == 0 {
                assert!(result[i] == get_expected_first_byte_1());
            } else {
                assert!(result[i] == hash[i]);
            }
        }
    }

    #[test]
    fn canonical_left_hash_should_be_canonical_left() {
        let hash = get_expected_digest_bytes_1();
        let canonical_left_hash = make_canonical_left(&hash);
        let is_left = is_canonical_left(&canonical_left_hash);
        let is_right = is_canonical_right(&canonical_left_hash);
        assert!(is_left);
        assert!(!is_right);
    }

    #[test]
    fn canonical_right_hash_should_be_canonical_right() {
        let hash = get_expected_digest_bytes_2();
        let canonical_right_hash = make_canonical_right(&hash);
        let is_left = is_canonical_left(&canonical_right_hash);
        let is_right = is_canonical_right(&canonical_right_hash);
        assert!(!is_left);
        assert!(is_right);
    }

    #[test]
    fn should_get_correct_action_digest() {
        let account_name = AccountName::from_str("provabletokn").unwrap();
        let action_name = ActionName::from_str("event").unwrap();
        let actor = AccountName::from_str("provabletokn").unwrap();
        let permission = PermissionName::from_str("active").unwrap();
        let permission_level = PermissionLevel { actor, permission };
        let authorization = vec![permission_level];
        let data = hex::decode(
            "e0d2b86b1a3962343021cd2a1eb3e9ad672b00000000000004454f53000000002a3078303236644336413433353631444138413641373735353338623139324133653933366330463239422301000000000000"
            ).unwrap();
        let action = Action::new(
            account_name,
            action_name,
            authorization,
            data,
        );
        let serialized_action = action.to_serialize_data();
        let result = sha256::Hash::hash(&serialized_action).to_string();
        assert!(result == get_expected_digest_1().to_string());
    }

    #[test]
    fn should_make_canonical_pair() {
        let digest_1 = get_expected_digest_bytes_1();
        let digest_2 = get_expected_digest_bytes_2();
        let result = make_canonical_pair(&digest_1, &digest_2);
        for i in 0..result.0.len() {
            if i == 0 {
                assert!(result.0[i] == get_expected_first_byte_1());
            } else {
                assert!(result.0[i] == digest_1[i]);
            }
        }
        for i in 0..result.1.len() {
            if i == 0 {
                assert!(result.1[i] == get_expected_first_byte_2());
            } else {
                assert!(result.1[i] == digest_2[i]);
            }
        }
    }

    #[test]
    fn should_hash_canonical_pair() {
        let expected_result =
            "a26284468e89fe4a5cce763ca3b3d3d37d5fcb35f289c63f0558487ec57ace28";
        let canonical_pair = get_sample_canonical_pair();
        let result = hash_canonical_pair(canonical_pair);
        assert!(result.to_string() == expected_result);
    }

    #[test]
    fn should_serialize_a_simple_action_receipt_correctly() {
        let expected_result =
            "6cd473b189a292bd520cac3430cc7934273da81cc3417376194a5d757b4abdc8"
                .to_string();
        let result = ActionReceipt::new(
            "eosio",
            "a6a370c6569034a4cc41935dd88f83d1c64e0414580872f29d87f69fe7a5d769",
            60725518,
            12,
            498637597,
            10,
            vec![AuthSequence::new("eosio", 59191700).unwrap()],
        )
            .unwrap()
            .to_digest();
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_even_number_of_action_receipts() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58316764
        let expected_result =
            "2f013d3ed57c89f1824772d18a4a74c043574bad47e9c6f088136e7595511810";
        let action_digest_1 = ActionReceipt::new(
            "eosio",
            "8e3e721a497dbae5e5fde0bb43e9086628809efaf102b763a3e9820adce9ce8f",
            62815613,
            12,
            503056735,
            10,
            vec![AuthSequence::new("eosio", 61275209).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_2 = ActionReceipt::new(
            "provabletokn",
            "4b991cebb3e6667b242aca3fb011623cd8ce2be2e8c24958da551c7b3ba68903",
            2884,
            48,
            503056736,
            80,
            vec![AuthSequence::new("provabletokn", 3090).unwrap()],
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![ action_digest_1, action_digest_2]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_odd_number_of_action_receipts_gt_one() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58319528
        let expected_result =
            "593f54cbc0b877b30cec5e510838b2b16ca00aca43e21d204d21eb8e8f947aa0";
        let action_digest_1 = ActionReceipt::new(
            "eosio",
            "23ab74b930cceea6061e1c4580ec988bf483a77e225cfca254d832928b4d1b36",
            62818486,
            12,
            503062766,
            10,
            vec![AuthSequence::new("eosio", 61277973).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_2 = ActionReceipt::new(
            "eosebetbullj",
             "b9243d8513e25705e89d7ccd0491f4a57d07b9866fd89d3446887af852cfed15",
             1440226,
             215,
             503062767,
             215,
            vec![AuthSequence::new("eosebetbullj", 1440687).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_digest_3 = ActionReceipt::new(
             "dvmh1tbb1him",
             "4bd1d3e987cd13e3d108a9a0cd185bf022cb1a826f69f163fcd109db54ba799f",
             804629,
             2,
             503062768,
             1,
            vec![AuthSequence::new("dvmh1tbb1him", 804649).unwrap()],
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![
            action_digest_1,
            action_digest_2,
            action_digest_3,
        ]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_action_mroot_when_action_has_gt_one_auth_sequence() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58345436
        let expected_result =
            "f93a91688d12170c24807d4bd507cf52dcde962ae4a41a86fe55231dee4df348";
        let action_receipt_1 = ActionReceipt::new(
            "eosio",
            "2d5371b958af052629f3fb62ede1bfcd94703675bc734535bf87fb615284dba3",
            62844592,
            12,
            503124645,
            10,
            vec![AuthSequence::new(
                "eosio",
                61303891
            ).unwrap()],
        )
            .unwrap()
            .to_digest();
        let action_receipt_2 = ActionReceipt::new(
            "authsequence",
            "ae341469a7b3936c70e9684a42ef8fc1975f1bb2fe1f3b0b1105eda7d3a6276a",
            10,
            1,
            503124646,
            1,
            vec![
                AuthSequence::new("othrsequence", 14).unwrap(),
                AuthSequence::new("rick11111111", 268).unwrap()
            ]
        )
            .unwrap()
            .to_digest();
        let result = get_merkle_digest(vec![
            action_receipt_1,
            action_receipt_2,
        ]);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_action_mroot_for_four_actions_correctly() {
        let digests = get_sample_action_digests();
        let expected_result =
            "8b4e5e5d3e7587065896d0076d65c72e03c11a9159d414eb3a2363b59108116a";
        let result = get_merkle_digest(digests);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_verify_merkle_proofs() {
        let num_proofs = 4;
        vec![0, num_proofs - 1]
            .iter()
            .enumerate()
            .map(|(_, i)| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material|
                 submission_material
                    .action_proofs[0]
                    .action_proof
                    .clone()
            )
            .map(|merkle_proof|
                 assert!(verify_merkle_proof(&merkle_proof).unwrap())
            )
            .for_each(drop);
    }

    #[test]
    fn should_get_merkle_root_from_merkle_path() {
        let expected_merkle_root =
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let block_id =
            "0000259a7cc27f04467b6c7362a936a143a5d9f324075b4c0d291c3974f80720";
        let merkle_path = vec![
            block_id,
            "0000259943aeb714e885c783bc79487cd025bb687b39d9de755d73a7fea000dd",
            "804c48aed6b4f21b9d13bd3cc260411dc8d7e442f0430659e9bbcc70af95c8aa",
            "80f39c9cda67aa2c1e4ec3a6c2ed6182dbb87b30d2d82b44a2a2a76d37f74aae",
            "29eb5e917272918a6da86be0aaec2275bef5b66062c7f717b738b92b01e24faa",
            "07d415864f60c2ca1318d4ebf4fd46e446697076d4f38abc3105531830da815e",
            "9006d928623a944863b1bef8a6df59fcb9c4790d8fe8b49c2fd4b0f88f48566c",
            "efc734fa150a9cfa74402a7d50fae265f36037c70af9b078bee7c3332fe62768",
            "3e2f1f8b53ec4b22ffe724ba11f1cb676a675a0a6cf097ed1d8a30d766008f76",
            "43e4b272895404d72bdb14f7a06c19342cbdaa132bf3538bb20be67b28db5fc8",
            "9e3a7f7e635ea41663de6855b81eda28320ae3d2ba669e2a8e1e1d4d8969cb5c",
            "2cba7c7ee5c1d8ba97ea1a841707fbb2147e883b56544ba821814aebe086383e",
            "a081325a023dd7018dd99d1d4192348c73d445f4a4fd4ca40a99c1914c3b30b3",
            "8394f7a83fda4dc1fb026aec143ccb4c9ce69c21f23ab3a8af0a741f8597df96",
            "2fa502d408f5bdf1660fa9fe3a1fcb432462467e7eb403a8499392ee5297d8d1",
        ]
            .iter()
            .map(|x| Ok(hex::decode(x)?))
            .collect::<Result<Vec<Bytes>>>()
            .unwrap();
        let result = get_merkle_root_from_merkle_path(&merkle_path)
            .unwrap();
        assert_eq!(hex::encode(result), expected_merkle_root);
    }

    #[test]
    fn should_get_incremental_merkle_root_from_blockroot_merkles() {
        let expected_incremerkle_root =
            "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let submission_material = get_sample_eos_submission_material_n(5);
        let active_nodes = submission_material
            .blockroot_merkle
            .clone();
        let node_count: u64 = submission_material
            .block_header
            .block_num()
            .into();
        let incremerkle = IncrementalMerkle::new(node_count, active_nodes);
        let incremerkle_root = hex::encode(
            &incremerkle
                .get_root()
                .to_bytes()
        );
        assert_eq!(incremerkle_root, expected_incremerkle_root);
    }
}
