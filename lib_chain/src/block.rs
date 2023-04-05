// This file is part of the project for the module CS3235 by Prateek
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This file contains the definition of the BlockTree
/// The BlockTree is a data structure that stores all the blocks that have been mined by this node or received from other nodes.
/// The longest path in the BlockTree is the main chain. It is the chain from the root to the working_block_id.
use core::panic;
use std::rc;
use std::time::SystemTime;
use base64ct::{Base64, Encoding};

use rsa::{RsaPublicKey};
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::{VerifyingKey};
use rsa::signature::{Signature as rsaSignature, Verifier};


use serde::{Deserialize, Serialize};
use sha2::{digest::block_buffer::Block, Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert,
};

pub type UserId = String;
pub type BlockId = String;
pub type Signature = String;
pub type TxId = String;

/// The enum type for the IPC messages (requests) from this client to the bin_wallet process.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqWallet {
    Initialize(String),
    Quit,
    SignRequest(String),
    VerifyRequest(String, String),
    GetUserInfo,
}

/// Merkle tree is used to verify the integrity of transactions in a block.
/// It is generated from a list of transactions. It will be stored inside `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MerkleTree {
    /// A list of lists of hashes, where the first list is the list of hashes of the transactions,
    /// the second list is the list of hashes of the first list, and so on.
    /// See the `create_merkle_tree` function for more details.
    pub hashes: Vec<Vec<String>>,
}

impl MerkleTree {
    /// Create a merkle tree from a list of transactions.
    /// The merkle tree is a list of lists of hashes,
    /// where the first list is the list of hashes of the transactions.
    /// The last list is the list with only one hash, called the Merkle root.
    /// - `txs`: a list of transactions
    /// - The return value is the root hash of the merkle tree
    
    pub fn create_merkle_tree(txs: Vec<Transaction>) -> (String, MerkleTree) {
        if txs.len() == 0 {
            panic!("create_merkel_tree get empty Transaction Vector.");
        }
        // Please fill in the blank
        let mut merkle_tree_hashes: Vec<Vec<String>> = vec![vec![]]; // create a 2D vector representing merkle tree
        merkle_tree_hashes[0] = Self::get_tx_hash(txs); // add hashes of all txs into 1st layer of merkle tree

        // Get remaining lists of hashes of previous list
        while merkle_tree_hashes.last().unwrap().len() != 1 {
            // if odd number of nodes, duplicate the last node
            if merkle_tree_hashes.last().unwrap().len() % 2 != 0 { 
                let i = merkle_tree_hashes.len() - 1;
                let j = merkle_tree_hashes[i].len() - 1;
                let temp = merkle_tree_hashes[i][j].clone();
                merkle_tree_hashes[i].push(temp);
            }

            let hash_list = merkle_tree_hashes.last().unwrap().to_owned();
            let len = hash_list.len();
            let mut i = 0;
            merkle_tree_hashes.push(vec![]); // add a new layer to merkle tree
            while i < len {
                let hash_input = hash_list.get(i).to_owned().unwrap().to_string() + hash_list.get(i+1).to_owned().unwrap(); // combine 2 hashes into 1 string 
                let hash = Sha256::digest(hash_input.as_bytes()); 
                let hash_str = format!("{:x}", hash);
                merkle_tree_hashes.last_mut().unwrap().push(hash_str); // add hash of string into last layer
                i += 2;
            }
        }
        return (merkle_tree_hashes.last().unwrap().get(0).to_owned().unwrap().to_string(), MerkleTree { hashes : merkle_tree_hashes});
    }
    // Please fill in the blank
    // Depending on your implementation, you may need additional functions here.

    // Get the hashes of each tx.
    pub fn get_tx_hash(txs: Vec<Transaction>) -> (Vec<String>) {
        let mut txs_hashes: Vec<String> = vec![];
        for tx in txs {
            // get hash of tx
            let hash = tx.gen_hash();
            txs_hashes.push(hash);
        }
        return txs_hashes;
    }
}

/// The struct containing a list of transactions and the merkle tree of the transactions.
/// Each block will contain one `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transactions {
    /// The merkle tree of the transactions
    pub merkle_tree: MerkleTree,
    /// A list of transactions
    pub transactions: Vec<Transaction>,
}

/// The struct is used to store the information of one transaction.
/// The transaction id is not stored explicitly, but can be generated from the transaction using the `gen_hash` function.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transaction {
    /// The user_id of the sender
    pub sender: UserId,
    /// The user_id of the receiver
    pub receiver: UserId,
    /// The message of the transaction.
    /// The expected format is `SEND $300   // By Alice   // 1678173972743`,
    /// where `300` is the amount of money to be sent,
    /// and the part after the first `//` is the comment: `Alice` is the friendly name of the sender, and `1678173972743` is the timestamp of the transaction.
    /// The comment part does not affect the validity of the transaction nor the computation of the balance.
    pub message: String,
    /// The signature of the transaction in base64 format
    pub sig: Signature,
}

impl Transaction {
    /// Create a new transaction struct given the sender, receiver, message, and signature.
    pub fn new(sender: UserId, receiver: UserId, message: String, sig: Signature) -> Transaction {
        Transaction {
            sender,
            receiver,
            message,
            sig,
        }
    }

    /// Compute the transaction id from the transaction. The transaction id is the sha256 hash of the serialized transaction struct in hex format.
    pub fn gen_hash(&self) -> TxId {
        let mut hasher = Sha256::new();
        let hasher_str = serde_json::to_string(&self).unwrap();
        hasher.update(hasher_str);
        let result = hasher.finalize();
        let tx_hash: TxId = format!("{:x}", result);
        tx_hash
    }

    /// Verify the signature of the transaction. Return true if the signature is valid, and false otherwise.
    pub fn verify_sig(&self) -> bool {
        // Please fill in the blank
        // verify the signature using the sender_id as the public key (you might need to change the format into PEM)
        // You can look at the `verify` function in `bin_wallet` for reference. They should have the same functionality.
        
        // change format into PEM
        let mut sender_newl = String::new();
        for (i, c) in self.sender.chars().enumerate() {
            sender_newl.push(c);
            if i != 0 && i != self.sender.len()-1 && i % 63 == 0 { // every 64th char
                sender_newl.push_str("\n");
            }
        }

        // covert string public_key to RsaPublicKey 
        let pem = "-----BEGIN RSA PUBLIC KEY-----\n".to_owned() + &sender_newl + "\n-----END RSA PUBLIC KEY-----\n";
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&pem).unwrap();
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);

        // convert sig from type string to type Singature
        let signature = Base64::decode_vec(&self.sig).unwrap();
        let verify_signature = rsaSignature::from_bytes(&signature).unwrap();

        // create sign request
        let req = serde_json::to_string(&(self.sender.clone(), self.receiver.clone(), self.message.clone())).unwrap();

        // verify signature
        let verify_result = verifying_key.verify(&req.as_bytes(), &verify_signature);
        return match verify_result {
            Ok(()) => true,
            Err(e) => {
                println!("[Signature verification failed]: {}", e);
                false
            }
        }
    }

}


/// The struct representing a whole block tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockTree {
    /// A map from block id to the block node
    pub all_blocks: HashMap<BlockId, BlockNode>,
    /// A map from block id to the list of its children (as block ids)
    pub children_map: HashMap<BlockId, Vec<BlockId>>,
    /// A map from block id to the depth of the block. The genesis block has depth 0.
    pub block_depth: HashMap<BlockId, u64>,
    /// The id of the root block (the genesis block)
    pub root_id: BlockId,
    /// The id of the working block (the block at the end of the longest chain)
    pub working_block_id: BlockId,
    /// A map to bookkeep the orphan blocks.
    /// Orphan blocks are blocks whose parent are not in the block tree yet.
    /// They should be added to the block tree once they can be connected to the block tree.
    pub orphans: HashMap<BlockId, BlockNode>,
    /// The id of the latest finalized block
    pub finalized_block_id: BlockId,
    /// A map from the user id to its balance
    pub finalized_balance_map: HashMap<UserId, i64>,
    /// A set of transaction ids that have been finalized. It includes all the transaction ids in the finalized blocks.
    pub finalized_tx_ids: HashSet<TxId>,
}

impl BlockTree {
    /// Create a new block tree with the genesis block as the root.
    pub fn new() -> BlockTree {
        let mut bt = BlockTree {
            all_blocks: HashMap::new(),
            children_map: HashMap::new(),
            block_depth: HashMap::new(),
            root_id: String::new(),
            working_block_id: String::new(),
            orphans: HashMap::new(),
            finalized_block_id: String::new(),
            finalized_balance_map: HashMap::new(),
            finalized_tx_ids: HashSet::new(),
        };
        let genesis_block = BlockNode::genesis_block();
        bt.all_blocks.insert("0".to_string(), genesis_block.clone());
        bt.block_depth.insert("0".to_string(), 0);
        bt.root_id = "0".to_string();
        bt.working_block_id = "0".to_string();
        for tx in genesis_block.transactions_block.transactions {
            let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();
            bt.finalized_balance_map.insert(tx.receiver, amount);
        }
        bt.finalized_block_id = "0".to_string();
        bt
    }

    /// Add a block to the block tree. If the block is not valid to be added to the tree
    /// (i.e. it does not satsify the conditions below), ignore the block. Otherwise, add the block to the BlockTree.
    ///
    /// 1. The block must have a valid nonce and the hash in the puzzle solution satisfies the difficulty requirement.
    /// 2. The block_id of the block must be equal to the computed hash in the puzzle solution.
    /// 3. The block does not exist in the block tree or the orphan map.
    /// 4. The transactions in the block must be valid. See the `verify_sig` function in the `Transaction` struct for details.
    /// 5. The parent of the block must exist in the block tree.
    ///     Otherwise, it will be bookkeeped in the orphans map.
    ///     When the parent block is added to the block tree, the block will be removed from the orphan map and checked against the conditions again.
    /// 6. The transactions in the block must not be duplicated with any transactions in its ancestor blocks.
    /// 7. Each sender in the txs in the block must have enough balance to pay for the transaction.
    ///    Conceptually, the balance of one address is the sum of the money sent to the address minus the money sent from the address
    ///    when walking from the genesis block to this block, according to the order of the txs in the blocks.
    ///    Mining reward is a constant of $10 (added to the reward_receiver address **AFTER** considering transactions in the block).
    ///
    /// When a block is successfully added to the block tree, update the related fields in the BlockTree struct
    /// (e.g., working_block_id, finalized_block_id, finalized_balance_map, finalized_tx_ids, block_depth, children_map, all_blocks, etc)
    pub fn add_block(&mut self, block: BlockNode, leading_zero_len: u16) -> () {
        // Please fill in the blank
        // 1. The block must have a valid nonce and the hash in the puzzle solution satisfies the difficulty requirement.
        // 2. The block_id of the block must be equal to the computed hash in the puzzle solution.
        // 4. The transactions in the block must be valid. See the `verify_sig` function in the `Transaction` struct for details.
        println!("#################### ADD_BLOCK FUNCTION CALLED #############################");

        if !block.validate_block(leading_zero_len).0 {
            println!("invalid block found. block_id : {}", block.header.block_id.clone());
            return; // ignore block
        }
        // 3. The block does not exist in the block tree or the orphan map.
        if (self.all_blocks.contains_key(&block.header.block_id) || self.orphans.contains_key(&block.header.block_id)) {
            println!("duplicated block found. block_id : {}", block.header.block_id.clone());
            return;
        }
        // 5. The parent of the block must exist in the block tree.
        // Otherwise, it will be bookkeeped in the orphans map.
        if (!self.all_blocks.contains_key(&block.header.parent)) {
            println!("orphan block found. block_id : {}", block.header.block_id.clone());
            self.orphans.insert(block.header.block_id.clone(), block);
            return;
        }

        // 6. The transactions in the block must not be duplicated with any transactions in its ancestor blocks.
        let mut set = HashSet::new(); // create a set containing all txs in curr block
        for tx in &block.transactions_block.transactions {
            let tx_id = tx.gen_hash();
            set.insert(tx_id);
        }

        let mut curr_block_id = block.header.parent.clone();
        loop {
            for tx in self.get_block(curr_block_id.clone()).unwrap().transactions_block.transactions {
                let tx_id = tx.gen_hash();
                if set.contains(&tx_id) { // ignore block if it has a tx that is duplicated in an ancestor block
                    println!("duplicated tx found. tx_id : {}", tx_id.clone());
                    return;
                }
            }
            // exit loop if reached genesis block
            if (curr_block_id.eq("0")) {
                break;
            }
            // update curr_block_id
            curr_block_id = self.get_block(curr_block_id.clone()).unwrap().header.parent.clone();
        }

        // assertion: block is valid, not found in all_blocks & orphans, parent is in tree, and all transactions are not duplicated in ancestor blocks.

        // 7. Each sender in the txs in the block must have enough balance to pay for the transaction.
        // Conceptually, the balance of one address is the sum of the money sent to the address minus the money sent from the address
        // when walking from the genesis block to this block, according to the order of the txs in the blocks.
        // Mining reward is a constant of $10 (added to the reward_receiver address **AFTER** considering transactions in the block).
        let mut pending_finalization_blocks = self.get_pending_finalization_blocks();
        pending_finalization_blocks.push(block.clone());
        let mut balance_map_copy = self.finalized_balance_map.clone();
        for block in &pending_finalization_blocks {
            // carry out the txs in this block
            for tx in &block.transactions_block.transactions {
                let sender_id = tx.sender.clone();
                let receiver_id = tx.receiver.clone();
                let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();
                // all addresses not in the balance map, they have $0 by default
                if (!balance_map_copy.contains_key(&sender_id)) {
                    balance_map_copy.insert(sender_id.clone(), 0);
                }
                if (!balance_map_copy.contains_key(&receiver_id)) {
                    balance_map_copy.insert(receiver_id.clone(), 0);
                }
                let sender_balance = balance_map_copy.get(&sender_id).unwrap().clone();
                let new_sender_balance = sender_balance - amount;
                balance_map_copy.insert(sender_id.clone(), new_sender_balance);
                if (new_sender_balance < 0 && sender_id.ne("GENESIS")) { // sender does not have enough balance. if sender is genesis, ignore check for balance since we assume genesis has infinite money
                    println!("sender not enough balance found.\nsender_id : {}\namount to be sent: {}\nsender_balance before sending : {}\nsender_balance after sending : {}", sender_id.clone(), amount.clone(), sender_balance.clone(), new_sender_balance.clone());
                    return;
                }
                let receiver_balance = balance_map_copy.get(&receiver_id).unwrap().clone();
                let new_receiver_balance = receiver_balance + amount;
                balance_map_copy.insert(receiver_id, new_receiver_balance);
            }

            // give mining reward to reward_receiver
            let reward_receiver_id = block.header.reward_receiver.clone();
            if (!balance_map_copy.contains_key(&reward_receiver_id)) {
                balance_map_copy.insert(reward_receiver_id.clone(), 0);
            }
            let reward_receiver_balance = balance_map_copy.get(&reward_receiver_id).unwrap().clone();
            let new_reward_receiver_balance = reward_receiver_balance + 10;
            balance_map_copy.insert(reward_receiver_id, new_reward_receiver_balance);
        }


        // assertion: block can be added

        // When a block is successfully added to the block tree, update the related fields in the BlockTree struct
        // (e.g., working_block_id, finalized_block_id, finalized_balance_map, finalized_tx_ids, block_depth, children_map, all_blocks, etc)
        self.all_blocks.insert(block.header.block_id.clone(), block.to_owned());
        if (!self.children_map.contains_key(&block.header.parent)) {
            self.children_map.insert(block.header.parent.clone(), vec![]); // create entry for parent block with no child in children_map if no entry exists
        }
        self.children_map.get_mut(&block.header.parent.clone()).unwrap().push(block.header.block_id.clone()); // add curr block as a children to its parent block
        let depth = self.block_depth[&block.header.parent] + 1;
        self.block_depth.insert(block.header.block_id.clone(), depth);
        self.working_block_id = block.header.block_id.clone();
        if pending_finalization_blocks.len().eq(&7) &&  pending_finalization_blocks[0].header.block_id.ne(&self.root_id){ // there is 7 blocks pending finalization, thus the oldest one can be finalized. however, if the oldest one is the genesis block which is already finalized, ignore.
            self.finalized_block_id = pending_finalization_blocks[0].header.block_id.clone(); // update finalized_block_id
            // carry out the txs in newly finalized block
            for tx in &pending_finalization_blocks[0].transactions_block.transactions { // update finalized_balance_map
                self.finalized_tx_ids.insert(tx.gen_hash()); // update finalized_tx_ids
                let sender_id = tx.sender.clone();
                let receiver_id = tx.receiver.clone();
                let amount = tx.message.split(" ").collect::<Vec<&str>>()[1]
                .trim_start_matches('$')
                .parse::<i64>()
                .unwrap();
                // all addresses not in the balance map, they have $0 by default
                if (!self.finalized_balance_map.contains_key(&sender_id)) {
                    self.finalized_balance_map.insert(sender_id.clone(), 0);
                }
                if (!self.finalized_balance_map.contains_key(&receiver_id)) {
                    self.finalized_balance_map.insert(receiver_id.clone(), 0);
                }
                let sender_balance = self.finalized_balance_map.get(&sender_id).unwrap().clone();
                let new_sender_balance = sender_balance - amount;
                self.finalized_balance_map.insert(sender_id, new_sender_balance);
                
                let receiver_balance = self.finalized_balance_map.get(&receiver_id).unwrap().clone();
                let new_receiver_balance = receiver_balance + amount;
                self.finalized_balance_map.insert(receiver_id, new_receiver_balance);
            }
            // give mining reward to reward_receiver
            let reward_receiver_id = pending_finalization_blocks[0].header.reward_receiver.clone();
            println!("reward_receiver_id : {}", reward_receiver_id.clone());
            if (!self.finalized_balance_map.contains_key(&reward_receiver_id)) {
                self.finalized_balance_map.insert(reward_receiver_id.clone(), 0);
            }
            let reward_receiver_balance = self.finalized_balance_map.get(&reward_receiver_id).unwrap().clone();
            let new_reward_receiver_balance = reward_receiver_balance + 10;
            self.finalized_balance_map.insert(reward_receiver_id, new_reward_receiver_balance);
            println!("a block is newly finalized.\nfinalized_balance_map:");
            for (key, value) in &self.finalized_balance_map {
                println!("{}: {}", key, value);
            }
        }
        
        // When the parent block is added to the block tree, the block will be removed from the orphan map and checked against the conditions again.
        for (orphan_id, orphan_node) in &self.orphans.clone() {
            // if parent of orphan is the curr block that is added, removed orphan from the orphan map and checked against the conditions again.
            if orphan_node.header.parent.eq(&block.header.block_id) {
                self.orphans.remove(orphan_id);
                self.add_block(orphan_node.clone(), leading_zero_len);
            }
        }
    }

    // Return a vector of pending_finalization_blocks from oldest to most recent
    pub fn get_pending_finalization_blocks(&self) -> Vec<BlockNode> {
        let mut pending_finalization_blocks = vec![];
        let mut curr_block_id = self.working_block_id.clone();
        for _i in 0..6 { // repeat 6 times
            // add curr block to pending_finalization_blocks
            let curr_block = self.get_block(curr_block_id.to_string()).unwrap();
            pending_finalization_blocks.insert(0, curr_block.clone());
            // if reached genesis, exit loop early. This is done after adding of genesis' txs if genesis block is not finalized.
            if curr_block_id.eq("0") {
                break;
            }
            curr_block_id = curr_block.header.parent.clone();
        }
        return pending_finalization_blocks;
    }

    /// Get the block node by the block id if exists. Otherwise, return None.
    pub fn get_block(&self, block_id: BlockId) -> Option<BlockNode> {
        // Please fill in the blank
        // find block in all_blocks
        let result = self.all_blocks.get(&block_id);
        match result {
            Some(x) => return Some(x.clone()),
            None    => (),
        }
        // find block in orphans
        let result2 = self.orphans.get(&block_id);
        match result {
            Some(x) => return Some(x.clone()),
            None    => (),
        }
        return None;
    }

    /// Get the finalized blocks on the longest path after the given block id, from the oldest to the most recent.
    /// The given block id should be any of the ancestors of the current finalized block id or the current finalized block id itself.
    /// If it is not the case, the function will panic (i.e. we do not consider inconsistent block tree caused by attacks in this project)
    pub fn get_finalized_blocks_since(&self, since_block_id: BlockId) -> Vec<BlockNode> {
        // Please fill in the blank
        let mut finalized_blocks = vec![];
        // start from the finalized_block_id i.e. the latest finalized block. Traverse upwards.
        let mut curr_block_id = self.finalized_block_id.clone();
        while curr_block_id.ne(&since_block_id) {
            let curr_block = self.get_block(curr_block_id.to_string()).unwrap();
            finalized_blocks.insert(0, curr_block); // Add block to the front. Ensures blocks are ordered from oldest to most recent since we are traversing upwards.
            curr_block_id = finalized_blocks[0].header.parent.clone();
        }
        return finalized_blocks;
    }

    /// Get the pending transactions on the longest chain that are confirmed but not finalized.
    pub fn get_pending_finalization_txs(&self) -> Vec<Transaction> {
        // Please fill in the blank
        let mut pending_finalization_txs = vec![];
        let mut curr_block_id = self.working_block_id.clone();

        for _i in 0..6 { // repeat 6 times
            // add txs in curr block to pending_finalization_txs
            let curr_block = self.get_block(curr_block_id.to_string()).unwrap();
            let mut pending_finalization_txs_in_curr_block = curr_block.transactions_block.transactions.clone();
            // add txs of older blocks to the front. Ensures pending+finalization_txs is ordered from oldest txs to most recent.
            pending_finalization_txs_in_curr_block.append(&mut pending_finalization_txs);
            pending_finalization_txs = pending_finalization_txs_in_curr_block;
            // if reached genesis, exit loop early. This is done after adding of genesis' txs if genesis block is not finalized.
            if curr_block_id.eq("0") {
                break;
            }
            curr_block_id = curr_block.header.parent.clone();
        }

        return pending_finalization_txs;
    }

    /// Get status information of the BlockTree for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the BlockTree.
        // It should be displayed in the Client UI eventually.
        let mut status = BTreeMap::new();
        status.insert("all_blocks".to_string(), serde_json::to_string(&self.all_blocks).unwrap());
        status.insert("children_map".to_string(), serde_json::to_string(&self.children_map).unwrap());
        status.insert("block_depth".to_string(), serde_json::to_string(&self.block_depth).unwrap());
        status.insert("root_id".to_string(), self.root_id.clone());
        status.insert("working_block_id".to_string(), self.working_block_id.clone());
        status.insert("orphans".to_string(), serde_json::to_string(&self.orphans).unwrap());
        status.insert("finalized_block_id".to_string(), self.finalized_block_id.clone());
        status.insert("finalized_balance_map".to_string(), serde_json::to_string(&self.finalized_balance_map).unwrap());
        status.insert("finalized_tx_ids".to_string(), serde_json::to_string(&self.finalized_tx_ids).unwrap());
        return status;
    }
}

/// The struct representing a puzzle for the miner to solve. The puzzle is to find a nonce such that when concatenated
/// with the serialized json string of this `Puzzle` struct, the sha256 hash of the result has the required leading zero length.
#[derive(Serialize)]
pub struct Puzzle {
    pub parent: BlockId,
    pub merkle_root: String,
    pub reward_receiver: UserId,
}

/// The struct representing a block header. Each `BlockNode` has one `BlockNodeHeader`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNodeHeader {
    /// The block id of the parent block.
    pub parent: BlockId,
    /// The merkle root of the transactions in the block.
    pub merkle_root: String,
    /// The timestamp of the block. For genesis block, it is 0. For other blocks, greater or equal to 1 is considered valid.
    pub timestamp: u64,
    /// The block id of the block (the block id is the sha256 hash of the concatination of the nonce and a `Puzzle` derived from the block)
    pub block_id: BlockId,
    /// The nonce is the solution found by the miner for the `Puzzle` derived from this block.
    pub nonce: String,
    /// The reward receiver of the block.
    pub reward_receiver: UserId,
}

/// The struct representing a block node.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNode {
    /// The header of the block.
    pub header: BlockNodeHeader,
    /// The transactions in the block.
    pub transactions_block: Transactions,
}

impl BlockNode {
    /// Create the genesis block that contains the initial transactions
    /// (give $299792458 to the address of Alice `MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==`)
    pub fn genesis_block() -> BlockNode {
        let header = BlockNodeHeader {
            parent: "0".to_string(),
            merkle_root: "0".to_string(),
            timestamp: 0,
            block_id: "0".to_string(),
            nonce: "0".to_string(),
            reward_receiver: "GENESIS".to_string(),
        };

        let transactions_block = Transactions {
            transactions: vec![Transaction::new(
                "GENESIS".to_owned(),
                "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ=="
                    .to_string(),
                "SEND $299792458".to_owned(),
                "GENESIS".to_owned(),
            )],
            merkle_tree: MerkleTree { hashes: vec![] }, // Skip merkle tree generation for genesis block
        };

        BlockNode {
            header,
            transactions_block,
        }
    }

    /// Check for block validity based solely on this block (not considering its validity inside a block tree).
    /// Return a tuple of (bool, String) where the bool is true if the block is valid and false otherwise.
    /// The string is the re-computed block id.
    /// The following need to be checked:
    /// 1. The block_id in the block header is indeed the sha256 hash of the concatenation of the nonce and the serialized json string of the `Puzzle` struct derived from the block.
    /// 2. All the transactions in the block are valid.
    /// 3. The merkle root in the block header is indeed the merkle root of the transactions in the block.
    pub fn validate_block(&self, leading_zero_len: u16) -> (bool, BlockId) {
        // Please fill in the blank

        // Get serialized json string of the `Puzzle` struct derived from the block
        let puzzle = Puzzle {
            parent: self.header.parent.to_owned(),
            merkle_root: self.header.merkle_root.to_owned(),
            reward_receiver: self.header.reward_receiver.to_owned(),
        };
        let jsonstr: String = serde_json::to_string(&puzzle).unwrap();
        // sha256 hash of the concatenation of the nonce and the json string
        let hash_input = self.header.nonce.to_owned() + &jsonstr;
        let mut hasher = Sha256::new();
        hasher.update(hash_input);
        let result = hasher.finalize();
        let computed_block_id: TxId = format!("{:x}", result);

        // check if these conditions are true:
        //  1. block_id == computed_block_id
        //  2. block_id (ans for puzzle) has correct_leading_zero
        //  3. all transactions are valid
        //  4. merkle_root_in_header == merkle_root_of_transactions
        if (self.header.block_id.eq(&computed_block_id) && self.correct_leading_zero(&computed_block_id, leading_zero_len) && self.all_valid_transactions() && self.same_merkle_root()) {
            return (true, computed_block_id);
        }
        return (false, computed_block_id);
    
    }

    fn correct_leading_zero(&self, block_id: &String, leading_zero_len: u16) -> bool {
        for i in 0..leading_zero_len {
            if block_id.chars().nth(i.into()).unwrap() != '0' {
                return false;
            }
        }
        return true;
    }

    // Check if all txs are valid i.e. they have a valid signature
    fn all_valid_transactions(&self) -> bool {
        for tx in &self.transactions_block.transactions {
            if !tx.verify_sig() {
                return false;
            }
        }
        return true;
    }

    fn same_merkle_root(&self) -> bool {
        return self.header.merkle_root.eq(&self.transactions_block.merkle_tree.hashes.last().unwrap()[0]);
    }

}
