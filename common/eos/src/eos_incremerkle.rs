use bitcoin::hashes::sha256::Hash as Sha256Hash;
use common::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    errors::AppError,
    traits::DatabaseInterface,
    types::{Bytes, NoneError, Result},
};
use derive_more::{Constructor, Deref, DerefMut};
use eos_chain::Checksum256;
use serde::{Deserialize, Serialize};

use crate::{
    bitcoin_crate_alias::hashes::{sha256, Hash},
    eos_utils::convert_hex_to_checksum256,
    EosDbUtils,
    EosState,
};

const MAX_NUM_INCREMERKLES: usize = 10;

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct Incremerkles(Vec<Incremerkle>);

impl Incremerkles {
    fn empty() -> Self {
        Self::default()
    }

    fn get_from_db<D: DatabaseInterface>(db_utils: &EosDbUtils<D>) -> Result<Self> {
        debug!("getting EOS incremerkles from db...");
        db_utils
            .get_db()
            .get(db_utils.get_eos_incremerkle_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
    }

    fn put_in_db<D: DatabaseInterface>(&self, db_utils: &EosDbUtils<D>) -> Result<()> {
        debug!("putting EOS incremerkles in db...");
        db_utils.get_db().put(
            db_utils.get_eos_incremerkle_key(),
            serde_json::to_vec(&self)?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn add(&mut self, incremerkle: Incremerkle) {
        if incremerkle.block_num() > self.latest_block_num() || self.is_empty() {
            info!("adding new incremerkle to list");
            self.insert(0, incremerkle);
            self.truncate(MAX_NUM_INCREMERKLES);
        }
        warn!("not adding incremerkle to list because its block num is behind chain tip")
    }

    fn block_nums(&self) -> Vec<u64> {
        self.iter().map(|i| i.block_num()).collect()
    }

    fn latest_block_num(&self) -> u64 {
        if self.is_empty() {
            warn!("no incremerkles to get latest block number from");
            0
        } else {
            self.0[0].block_num()
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incremerkle {
    node_count: u64,
    active_nodes: Vec<Checksum256>,
}

impl Incremerkle {
    fn block_num(&self) -> u64 {
        self.node_count()
    }

    fn node_count(&self) -> u64 {
        self.node_count
    }

    pub fn get_incremerkle_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
        info!("getting eos incremerkle from db...");
        Self::get_from_db(&state.eos_db_utils).map(|i| state.add_incremerkle(i))
    }

    pub fn save_incremerkle_from_state_to_db<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
        info!("saving incremerkle from state to db...");
        state.incremerkle.put_in_db(&state.eos_db_utils).and(Ok(state))
    }

    fn get_from_db<D: DatabaseInterface>(db_utils: &EosDbUtils<D>) -> Result<Self> {
        db_utils.get_incremerkle_from_db()
    }

    fn put_in_db<D: DatabaseInterface>(&self, db_utils: &EosDbUtils<D>) -> Result<()> {
        db_utils.put_incremerkle_in_db(self)
    }

    fn make_canonical_left(val: &Checksum256) -> Checksum256 {
        let mut canonical_l: Checksum256 = *val;
        canonical_l.set_hash0(canonical_l.hash0() & 0xFFFF_FFFF_FFFF_FF7F_u64);
        canonical_l
    }

    fn make_canonical_right(val: &Checksum256) -> Checksum256 {
        let mut canonical_r: Checksum256 = *val;
        canonical_r.set_hash0(canonical_r.hash0() | 0x0000_0000_0000_0080_u64);
        canonical_r
    }

    fn make_canonical_pair(l: &Checksum256, r: &Checksum256) -> (Checksum256, Checksum256) {
        (Self::make_canonical_left(l), Self::make_canonical_right(r))
    }

    // NOTE: Some logic in here courtesy of: https://github.com/bifrost-codes/rust-eos/
    // NOTE: Given a power-of-2 (assumed to be correct) return the number of leading zeros.
    //
    // This is a classic count-leading-zeros in parallel without the necessary
    // math to make it safe for anything that is not already a power-of-2
    //
    // Note that `count_leading_zeroes_of_a_power_of_2(0)` returns 64 because the values are
    // 64 bit integers, and so in the case of 0, there are indeed 64 leading zeroes, despite `0`
    // not actually being a power of 2.
    //
    // @param value - a power-of-2 (unchecked)
    // @return the number of leading zeros
    fn count_leading_zeroes_of_a_power_of_2(value: u64) -> Result<usize> {
        /*
        if value > 9223372036854775808 {
            Err(format!("Cannot count leading zeres of {} without overflowing!", value).into())
        };
        */
        let mut leading_zeroes: usize = 64;
        if value != 0 {
            leading_zeroes -= 1;
        }
        if (value & 0x0000_0000_FFFF_FFFF_u64) != 0 {
            leading_zeroes -= 32;
        }
        if (value & 0x0000_FFFF_0000_FFFF_u64) != 0 {
            leading_zeroes -= 16;
        }
        if (value & 0x00FF_00FF_00FF_00FF_u64) != 0 {
            leading_zeroes -= 8;
        }
        if (value & 0x0F0F_0F0F_0F0F_0F0F_u64) != 0 {
            leading_zeroes -= 4;
        }
        if (value & 0x3333_3333_3333_3333_u64) != 0 {
            leading_zeroes -= 2;
        }
        if (value & 0x5555_5555_5555_5555_u64) != 0 {
            leading_zeroes -= 1;
        }
        Ok(leading_zeroes)
    }

    // NOTE: Given a number of nodes return the depth required to store them
    // in a fully balanced binary tree.
    //
    // @param node_count - the number of nodes in the implied tree
    // @return the max depth of the minimal tree that stores them
    fn calculate_max_depth(node_count: u64) -> Result<usize> {
        if node_count == 0 {
            return Ok(0);
        }
        let implied_count =
            u64::checked_next_power_of_two(node_count).ok_or(NoneError("Next power of two has overflowed!"))?;
        Ok(Self::count_leading_zeroes_of_a_power_of_2(implied_count)? + 1)
    }

    pub(crate) fn new(node_count: u64, active_nodes: Vec<Checksum256>) -> Self {
        Incremerkle {
            node_count,
            active_nodes,
        }
    }

    pub(crate) fn append(&mut self, digest: Checksum256) -> Result<Checksum256> {
        let mut partial = false;
        let max_depth = Self::calculate_max_depth(self.node_count + 1)?;
        let mut current_depth = max_depth - 1;
        let mut index = self.node_count;
        let mut top = digest;
        let mut active_iter = self.active_nodes.iter();
        let mut updated_active_nodes: Vec<Checksum256> = Vec::with_capacity(max_depth);

        while current_depth > 0 {
            if (index & 0x1) == 0 {
                if !partial {
                    updated_active_nodes.push(top);
                }

                top = Checksum256::hash(Self::make_canonical_pair(&top, &top))?;
                partial = true;
            } else {
                let left_value = active_iter
                    .next()
                    .ok_or_else(|| AppError::Custom("✘ Incremerkle error!".into()))?;

                if partial {
                    updated_active_nodes.push(*left_value);
                }

                top = Checksum256::hash(Self::make_canonical_pair(left_value, &top))?;
            }

            current_depth -= 1;
            index >>= 1;
        }

        updated_active_nodes.push(top);

        self.active_nodes = updated_active_nodes;

        self.node_count += 1;

        Ok(self.active_nodes[self.active_nodes.len() - 1])
    }

    pub(crate) fn get_root(&self) -> Checksum256 {
        if self.node_count > 0 {
            self.active_nodes[self.active_nodes.len() - 1]
        } else {
            Default::default()
        }
    }

    fn is_canonical_left(hash: &Checksum256) -> bool {
        hash.0[0] & 0b1000_0000 == 0
    }

    fn is_canonical_right(hash: &Checksum256) -> bool {
        !Self::is_canonical_left(hash)
    }

    fn concatenate_canonical_pair(pair: (Checksum256, Checksum256)) -> Bytes {
        [pair.0 .0, pair.1 .0].concat()
    }

    fn hash_canonical_pair(pair: (Checksum256, Checksum256)) -> Sha256Hash {
        sha256::Hash::hash(&Self::concatenate_canonical_pair(pair))
    }

    fn make_and_hash_canonical_pair(l: &Checksum256, r: &Checksum256) -> Result<Checksum256> {
        convert_hex_to_checksum256(hex::encode(
            &Self::hash_canonical_pair(Self::make_canonical_pair(l, r)).to_vec(),
        ))
    }

    pub(crate) fn verify_merkle_proof(merkle_proof: &[String]) -> Result<bool> {
        let mut node = convert_hex_to_checksum256(&merkle_proof[0])?;
        let leaves = merkle_proof[..merkle_proof.len() - 1]
            .iter()
            .map(|h| convert_hex_to_checksum256(h))
            .collect::<Result<Vec<Checksum256>>>()?;
        for leaf in leaves.iter().skip(1) {
            if Self::is_canonical_right(leaf) {
                node = Self::make_and_hash_canonical_pair(&node, leaf)?;
            } else {
                node = Self::make_and_hash_canonical_pair(leaf, &node)?;
            }
        }
        let last_str = match merkle_proof.last() {
            Some(s) => s.to_string(),
            _ => "".to_string(),
        };
        let last = convert_hex_to_checksum256(&last_str)?;
        Ok(node == last)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use std::str::FromStr;

    use common::{test_utils::get_test_database, types::Byte};
    use eos_chain::{AccountName, Action, ActionName, PermissionLevel, PermissionName, SerializeData};

    use super::*;
    use crate::{
        eos_action_receipt::{AuthSequence, EosActionReceipt},
        eos_test_utils::{get_sample_action_digests, get_sample_eos_submission_material_n},
    };

    fn get_expected_digest_hex_1() -> &'static str {
        "9b9babebfbdff48ce4002b5f3c7f999c0ee74707b6d121c47ef5db68c6be7262"
    }

    fn get_expected_digest_hex_2() -> &'static str {
        "122cd09d66ca7df007a35bd9c9be5484833f1a69ad0c8527c3e2a56b6955e761"
    }

    fn get_expected_digest_1() -> Checksum256 {
        convert_hex_to_checksum256(get_expected_digest_hex_1()).unwrap()
    }

    fn get_expected_digest_2() -> Checksum256 {
        convert_hex_to_checksum256(get_expected_digest_hex_2()).unwrap()
    }

    fn get_expected_first_byte_1() -> Byte {
        0b0001_1011
    }

    fn get_expected_first_byte_2() -> Byte {
        0b1001_0010
    }

    fn get_sample_canonical_pair() -> (Checksum256, Checksum256) {
        Incremerkle::make_canonical_pair(&get_expected_digest_1(), &get_expected_digest_2())
    }

    pub(crate) fn get_merkle_digest(mut leaves: Vec<Bytes>) -> Bytes {
        if leaves.is_empty() {
            return vec![0x00];
        }
        while leaves.len() > 1 {
            if leaves.len() % 2 != 0 {
                let last = leaves[leaves.len() - 1].clone();
                leaves.push(last);
            }
            for i in 0..(leaves.len() / 2) {
                leaves[i] = Incremerkle::hash_canonical_pair(Incremerkle::make_canonical_pair(
                    &convert_hex_to_checksum256(&hex::encode(&leaves[2 * i])).unwrap(),
                    &convert_hex_to_checksum256(&hex::encode(&leaves[(2 * i) + 1])).unwrap(),
                ))
                .to_vec();
            }
            leaves.resize(leaves.len() / 2, vec![0x00]);
        }
        leaves[0].clone()
    }

    #[test]
    fn should_make_hash_canonical_right() {
        let hash = get_expected_digest_2();
        let result = Incremerkle::make_canonical_right(&hash);
        for i in 0..hash.0.len() {
            if i == 0 {
                assert_eq!(result.0[i], get_expected_first_byte_2());
            } else {
                assert_eq!(result.0[i], hash.0[i]);
            }
        }
    }

    #[test]
    fn should_make_hash_canonical_left() {
        let hash = get_expected_digest_1();
        let result = Incremerkle::make_canonical_left(&hash);
        for i in 0..hash.0.len() {
            if i == 0 {
                assert_eq!(result.0[i], get_expected_first_byte_1());
            } else {
                assert_eq!(result.0[i], hash.0[i]);
            }
        }
    }

    #[test]
    fn canonical_left_hash_should_be_canonical_left() {
        let hash = get_expected_digest_1();
        let canonical_left_hash = Incremerkle::make_canonical_left(&hash);
        let is_left = Incremerkle::is_canonical_left(&canonical_left_hash);
        let is_right = Incremerkle::is_canonical_right(&canonical_left_hash);
        assert!(is_left);
        assert!(!is_right);
    }

    #[test]
    fn canonical_right_hash_should_be_canonical_right() {
        let hash = get_expected_digest_2();
        let canonical_right_hash = Incremerkle::make_canonical_right(&hash);
        let is_left = Incremerkle::is_canonical_left(&canonical_right_hash);
        let is_right = Incremerkle::is_canonical_right(&canonical_right_hash);
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
        let action = Action {
            account: account_name,
            name: action_name,
            authorization,
            data,
        };
        let serialized_action = action.to_serialize_data().unwrap();
        let result = sha256::Hash::hash(&serialized_action).to_vec();
        assert_eq!(result, get_expected_digest_1().0.to_vec());
    }

    #[test]
    fn should_make_canonical_pair() {
        let digest_1 = get_expected_digest_1();
        let digest_2 = get_expected_digest_2();
        let (left, right) = Incremerkle::make_canonical_pair(&digest_1, &digest_2);

        for i in 0..left.0.len() {
            if i == 0 {
                assert_eq!(left.0[i], get_expected_first_byte_1());
            } else {
                assert_eq!(left.0[i], digest_1.0[i]);
            }
        }
        for i in 0..right.0.len() {
            if i == 0 {
                assert_eq!(right.0[i], get_expected_first_byte_2());
            } else {
                assert_eq!(right.0[i], digest_2.0[i]);
            }
        }
    }

    #[test]
    fn should_hash_canonical_pair() {
        let expected_result = "a26284468e89fe4a5cce763ca3b3d3d37d5fcb35f289c63f0558487ec57ace28";
        let canonical_pair = get_sample_canonical_pair();
        let result = Incremerkle::hash_canonical_pair(canonical_pair);
        assert_eq!(result.to_string(), expected_result);
    }

    #[test]
    fn should_serialize_a_simple_action_receipt_correctly() {
        let expected_result = "6cd473b189a292bd520cac3430cc7934273da81cc3417376194a5d757b4abdc8".to_string();
        let result = EosActionReceipt::new(
            "eosio",
            "a6a370c6569034a4cc41935dd88f83d1c64e0414580872f29d87f69fe7a5d769",
            60725518,
            12,
            498637597,
            10,
            vec![AuthSequence::new("eosio", 59191700).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_even_number_of_action_receipts() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58316764
        let expected_result = "2f013d3ed57c89f1824772d18a4a74c043574bad47e9c6f088136e7595511810";
        let action_digest_1 = EosActionReceipt::new(
            "eosio",
            "8e3e721a497dbae5e5fde0bb43e9086628809efaf102b763a3e9820adce9ce8f",
            62815613,
            12,
            503056735,
            10,
            vec![AuthSequence::new("eosio", 61275209).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_digest_2 = EosActionReceipt::new(
            "provabletokn",
            "4b991cebb3e6667b242aca3fb011623cd8ce2be2e8c24958da551c7b3ba68903",
            2884,
            48,
            503056736,
            80,
            vec![AuthSequence::new("provabletokn", 3090).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let result = get_merkle_digest(vec![action_digest_1, action_digest_2]);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_odd_number_of_action_receipts_gt_one() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58319528
        let expected_result = "593f54cbc0b877b30cec5e510838b2b16ca00aca43e21d204d21eb8e8f947aa0";
        let action_digest_1 = EosActionReceipt::new(
            "eosio",
            "23ab74b930cceea6061e1c4580ec988bf483a77e225cfca254d832928b4d1b36",
            62818486,
            12,
            503062766,
            10,
            vec![AuthSequence::new("eosio", 61277973).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_digest_2 = EosActionReceipt::new(
            "eosebetbullj",
            "b9243d8513e25705e89d7ccd0491f4a57d07b9866fd89d3446887af852cfed15",
            1440226,
            215,
            503062767,
            215,
            vec![AuthSequence::new("eosebetbullj", 1440687).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_digest_3 = EosActionReceipt::new(
            "dvmh1tbb1him",
            "4bd1d3e987cd13e3d108a9a0cd185bf022cb1a826f69f163fcd109db54ba799f",
            804629,
            2,
            503062768,
            1,
            vec![AuthSequence::new("dvmh1tbb1him", 804649).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let result = get_merkle_digest(vec![action_digest_1, action_digest_2, action_digest_3]);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_action_mroot_when_action_has_gt_one_auth_sequence() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58345436
        let expected_result = "f93a91688d12170c24807d4bd507cf52dcde962ae4a41a86fe55231dee4df348";
        let action_receipt_1 = EosActionReceipt::new(
            "eosio",
            "2d5371b958af052629f3fb62ede1bfcd94703675bc734535bf87fb615284dba3",
            62844592,
            12,
            503124645,
            10,
            vec![AuthSequence::new("eosio", 61303891).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_receipt_2 = EosActionReceipt::new(
            "authsequence",
            "ae341469a7b3936c70e9684a42ef8fc1975f1bb2fe1f3b0b1105eda7d3a6276a",
            10,
            1,
            503124646,
            1,
            vec![
                AuthSequence::new("othrsequence", 14).unwrap(),
                AuthSequence::new("rick11111111", 268).unwrap(),
            ],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let result = get_merkle_digest(vec![action_receipt_1, action_receipt_2]);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_action_mroot_for_four_actions_correctly() {
        let digests = get_sample_action_digests().unwrap();
        let expected_result = "8b4e5e5d3e7587065896d0076d65c72e03c11a9159d414eb3a2363b59108116a";
        let result = get_merkle_digest(digests);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_verify_merkle_proofs() {
        let num_proofs = 4;
        [0, num_proofs - 1]
            .iter()
            .enumerate()
            .map(|(_, i)| get_sample_eos_submission_material_n(i + 1))
            .map(|submission_material| submission_material.action_proofs[0].action_proof.clone())
            .for_each(|merkle_proof| assert!(Incremerkle::verify_merkle_proof(&merkle_proof).unwrap()));
    }

    #[test]
    fn should_get_incremerkle_root_from_interim_block_idss() {
        let expected_incremerkle_root = "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let submission_material = get_sample_eos_submission_material_n(5);
        let active_nodes = submission_material.interim_block_ids.clone();
        let node_count: u64 = submission_material.block_header.block_num().into();
        let incremerkle = Incremerkle::new(node_count, active_nodes);
        let incremerkle_root = hex::encode(incremerkle.get_root().to_bytes());
        assert_eq!(incremerkle_root, expected_incremerkle_root);
    }

    #[test]
    fn should_count_leading_zeroes_of_powers_of_2() {
        let num_digits_in_u64: u32 = 64;
        let mut results: Vec<u32> = vec![];
        let mut expected_results: Vec<u32> = vec![];
        for i in 0..num_digits_in_u64 {
            let two_to_the_power_of_i = 2_u64.pow(i);
            let result = Incremerkle::count_leading_zeroes_of_a_power_of_2(two_to_the_power_of_i).unwrap();
            results.push(result.try_into().unwrap());
            expected_results.push(i)
        }
        results
            .iter()
            .zip(expected_results.iter())
            .for_each(|(result, expected_result)| assert_eq!(result, expected_result))
    }

    #[test]
    fn should_put_and_get_incremerkle_in_db() {
        let db = get_test_database();
        let eos_db_utils = EosDbUtils::new(&db);
        let expected_incremerkle_root = "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let submission_material = get_sample_eos_submission_material_n(5);
        let active_nodes = submission_material.interim_block_ids.clone();
        let node_count: u64 = submission_material.block_header.block_num().into();
        let incremerkle = Incremerkle::new(node_count, active_nodes);
        let incremerkle_root = hex::encode(incremerkle.get_root().to_bytes());
        assert_eq!(incremerkle_root, expected_incremerkle_root);
        incremerkle.put_in_db(&eos_db_utils).unwrap();
        let incremerkle_from_db = Incremerkle::get_from_db(&eos_db_utils).unwrap();
        assert_eq!(incremerkle_from_db, incremerkle);
    }

    #[test]
    fn should_only_allow_max_num_incremerkles() {
        let mut incremerkles = Incremerkles::empty();
        for i in 0..MAX_NUM_INCREMERKLES + 10 {
            incremerkles.add(Incremerkle::new(i as u64, vec![]));
            assert_eq!(incremerkles.latest_block_num(), i as u64);
            if i < MAX_NUM_INCREMERKLES {
                assert_eq!(incremerkles.len(), i + 1)
            } else {
                assert_eq!(incremerkles.len(), MAX_NUM_INCREMERKLES)
            }
        }
    }

    #[test]
    fn should_put_and_get_incremerkles_in_db() {
        let mut incremerkles = Incremerkles::empty();
        for i in 0..MAX_NUM_INCREMERKLES {
            incremerkles.add(Incremerkle::new(i as u64, vec![]));
        }
        let db = get_test_database();
        let db_utils = EosDbUtils::new(&db);
        incremerkles.put_in_db(&db_utils).unwrap();
        let incremerkles_from_db = Incremerkles::get_from_db(&db_utils).unwrap();
        assert_eq!(incremerkles, incremerkles_from_db);
        assert_eq!(incremerkles.block_nums(), incremerkles_from_db.block_nums());
    }

    #[test]
    fn should_only_add_subsequent_incremerkles() {
        let mut incremerkles = Incremerkles::empty();
        assert_eq!(incremerkles.len(), 0);
        let i1 = Incremerkle::new(1, vec![]);
        let i2 = Incremerkle::new(2, vec![]);
        let i3 = Incremerkle::new(3, vec![]);
        incremerkles.add(i2);
        assert_eq!(incremerkles.len(), 1);
        assert_eq!(incremerkles.latest_block_num(), 2);
        incremerkles.add(i3);
        assert_eq!(incremerkles.len(), 2);
        assert_eq!(incremerkles.latest_block_num(), 3);
        incremerkles.add(i1);
        assert_eq!(incremerkles.len(), 2);
        assert_eq!(incremerkles.latest_block_num(), 3);
    }
}
