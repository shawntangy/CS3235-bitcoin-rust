// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file contains the definition of the transaction pool.
// The transaction pool `TxPool` is a data structure that stores all the valid transactions that are not yet finalized.
// It helps with filtering the transactions that can be included in a new block.
use std::{convert, collections::{HashMap, BTreeMap, HashSet}, hash::Hash};
use serde::{Serialize, Deserialize};
use lib_chain::block::{Signature, Transaction, TxId, BlockId, BlockNode};

/// The maximum number of transactions that can be stored in the pool. Extra transactions will be dropped.
const MAX_TX_POOL: usize = 10000;


/// A transaction pool that stores received transactions that are not yet finalized.
#[derive(Serialize, Deserialize, Debug, Clone)] 
pub struct TxPool {
    /// A list of transaction ids in the pool
    pub pool_tx_ids: Vec<TxId>,
    /// A map from transaction id (TxId) to transaction
    pub pool_tx_map: HashMap<TxId, Transaction>,
    /// A set of transaction ids that have been removed from the pool, so that duplicate transactions can be filtered out.
    pub removed_tx_ids: HashSet<TxId>,
    /// The id of the last finalized block. Transactions that are finalized will be removed from the pool and added to the removed_tx_ids set.
    pub last_finalized_block_id: BlockId
}


impl TxPool {
    /// Create a new transaction pool
    pub fn new () -> TxPool {
        TxPool { 
            pool_tx_ids: vec![], 
            pool_tx_map: HashMap::new(), 
            last_finalized_block_id: "0".to_string(),
            removed_tx_ids: HashSet::new()
        }
    }

    /// Add a transaction `tx` to the pool if it satisfies the following conditions:
    /// - The transaction is not already in the pool
    /// - The transaction is not already in the removed_tx_ids set
    /// - The pool size is less than MAX_TX_POOL
    /// - The transaction has valid signature
    /// It returns true if the transaction satisfies the conditions above and is successfully added to the pool, and false otherwise.
    pub fn add_tx(&mut self, tx: Transaction) -> bool {        
        // retrieve tx id, which is a sha256 hash string in hex
        let tx_id = tx.gen_hash(); 

        // lookup the map, ensure not found
        if self.pool_tx_map.contains_key(&tx_id) {
            return false;
        }

        // lookup the set, ensure not found
        if self.removed_tx_ids.contains(&tx_id) {
            return false;
        }

        // check vector size if >= 10k
        if self.pool_tx_ids.len() >= MAX_TX_POOL {
            return false;
        }
        // use transaction.verify_sig() which returns true/false
        if !tx.verify_sig() {
            return false;
        }
        // if all okay save and return true
        // add to vector and map
        self.pool_tx_ids.push(tx_id.clone());
        self.pool_tx_map.insert(tx_id, tx);
        true
        
    }

    /// Deleting a tx from the pool. This function is used by remove_txs_from_finalized_blocks and some unit tests.
    /// It should update pool_tx_ids, pool_tx_map, and removed_tx_ids.
    /// If the transaction does not exist in the pool, make sure it is added to removed_tx_ids.
    pub fn del_tx(&mut self, tx_id: TxId) -> () {

        // if it exists in the map, it returns value as some()
        if self.pool_tx_map.remove(&tx_id).is_some() {
            // remove from vector
            self.pool_tx_ids.retain(|id| id != &tx_id);
        } 
        // ensure added to hash set
        self.removed_tx_ids.insert(tx_id);

        
    }


    /// Filter `max_count` number of tx from the pool. It is used for creating puzzle.
    /// - `max_count`: the maximum number of transactions to be returned
    /// - `excluding_txs`: a list of transactions that should not be included in the returned list. 
    ///                    It is used to filter out those transactions on the longest chain but hasn't been finalized yet.
    pub fn filter_tx(&self, max_count: u16, excluding_txs: & Vec<Transaction>) -> Vec<Transaction> {
        let mut txs = Vec::new(); // final vec to return
        let mut excluded_txs_set = HashSet::new();
        for tx in excluding_txs {
            excluded_txs_set.insert(tx.gen_hash()); // put the id inside the set for faster lookup
        }

        for tx_id in &self.pool_tx_ids {
            // if tx_id not part of excluded transactions
            if !excluded_txs_set.contains(tx_id){
                // retrieve the transaction and push to vector
                if let Some(tx) = self.pool_tx_map.get(tx_id) {
                    txs.push(tx.clone());
                    // check if after adding it is maxed out to return
                    if txs.len() == max_count as usize {
                        break;
                    }
                }
            }
        }
        txs
    }

    /// Remove transactions from the pool given a list of finalized blocks. Update last_finalized_block_id as the last block in the list.
    pub fn remove_txs_from_finalized_blocks(&mut self, finalized_blocks: &Vec<BlockNode>) {
        for block in finalized_blocks {
            for tx in &block.transactions_block.transactions {
                self.del_tx(tx.gen_hash());
            }
            // Update last_finalized_block_id
            self.last_finalized_block_id = block.header.block_id.clone();
        }
        // yet to test this function
    }

    /// Get status information of the tx_pool for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the tx_pool. 
        // It should be displayed in the Client UI eventually.
        let mut miner_status = BTreeMap::new();
        miner_status.insert("#pool_tx_map".to_string(), self.pool_tx_map.len().to_string());
        miner_status
        
    }
}


