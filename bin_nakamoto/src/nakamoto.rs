// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Nakamoto struct, related data structs and methods.
// The Nakamoto leverages lib_chain, lib_miner, lib_tx_pool and lib_network to implement the Nakamoto consensus algorithm.
// You can see detailed instructions in the comments below.

use std::sync::mpsc::Sender;
use std::{thread, time::Duration};
use std::sync::{Arc, Mutex, RwLock};
use serde::{Deserialize, Serialize};
use lib_chain::block::{BlockTree, Puzzle, Transactions, MerkleTree, BlockNode, BlockNodeHeader, Transaction, self};
use lib_miner::miner::{Miner, PuzzleSolution};
use lib_tx_pool::pool::TxPool;
use lib_network::p2pnetwork::{P2PNetwork};
use lib_network::netchannel::{NetAddress};
use std::collections::{HashMap, BTreeMap};

type UserId = String;

/// The struct to represent configuration of the Nakamoto instance.
/// The configuration does not contain any user information. The Nakamoto algorithm is user-independent.
/// The configuration sets information about neighboring nodes, miner, block creation, etc.
#[derive(Serialize, Deserialize, Debug, Clone)] 
pub struct Config {
    /// the list of addresses of neighboring nodes
    pub neighbors: Vec<NetAddress>,
    /// the address of this node
    pub addr: NetAddress,
    /// the number of threads used to mine a new block (for miner)
    pub miner_thread_count: u16,
    /// the length of the nonce string (for miner)
    pub nonce_len: u16,
    // difficulty to mine a new block (for miner)
    pub difficulty_leading_zero_len: u16,
    // difficulty to accept a new block (for verifying the block)
    pub difficulty_leading_zero_len_acc: u16,
    // the seed for the miner thread 0 (for miner)
    pub miner_thread_0_seed: u64,
    // the reward receiver (for mined blocks)
    pub mining_reward_receiver: UserId,
    // the max number of transactions in one block (for creating a new block)
    pub max_tx_in_one_block: u16
}


/// Create a puzzle for the miner given a chain and a tx pool (as smart pointers). 
/// It returns the puzzle (serialization of the Puzzle struct) and the corresponding incomplete block (nonce and block_id not filled)
fn create_puzzle(chain_p: Arc<Mutex<BlockTree>>, tx_pool_p: Arc<Mutex<TxPool>>, tx_count: u16, reward_receiver: UserId) -> (String, BlockNode) {
    // Please fill in the blank
    // Filter transactions from tx_pool and get the last node of the longest chain.
    let pending_finalization_txs : Vec<Transaction> = vec![];
    let mut filtered_txs : Vec<Transaction> = vec![];
    let mut filtered_txs_p = Arc::new(filtered_txs);
    let last_block_id = "";

    loop {
        &filtered_txs_p = tx_pool_p.lock().unwrap().filter_tx(tx_count, &vec![]);
        let last_block_id = chain_p.lock().unwrap().working_block_id.clone();
        if filtered_txs.len() >= 1 {
            break;
        }
    }
    eprintln!("outer tx vec: {:?}", filtered_txs);

    // while filtered_txs.len() < 1 {
    //     thread::sleep(Duration::from_millis(1000));
    //     // eprint!("{}", i.clone());
    //     // i += 1;
    //     // eprintln!("no txs filtered");
    //     // let pending_finalization_txs = chain_p.lock().unwrap().get_pending_finalization_txs();
    //     // let filtered_txs = tx_pool_p.lock().unwrap().filter_tx(tx_count, &pending_finalization_txs);
    //     let mut filtered_txs = tx_pool_p.lock().unwrap().filter_tx(tx_count, &vec![]);
    //     eprintln!("{:?}", filtered_txs);
        
    //     let last_block_id = chain_p.lock().unwrap().working_block_id.clone();
    //     eprintln!("len: {}", filtered_txs.len());
    // }
    
    // Please fill in the blank
    // Create a block node with the transactions and the merkle root.
    // Leave the nonce and the block_id empty (to be filled after solving the puzzle).
    // The timestamp can be set to any positive interger.
    let (merkle_root, merkle_tree) = MerkleTree::create_merkle_tree(filtered_txs.clone());
    let pre_block = BlockNode {
        header: BlockNodeHeader {
            parent: last_block_id.clone().to_owned(),
            merkle_root: merkle_root.clone(),
            timestamp: 14,
            block_id: "".to_owned(),
            nonce: "".to_owned(),
            reward_receiver: reward_receiver.clone()
        },
        transactions_block: Transactions {
            merkle_tree: merkle_tree,
            transactions: filtered_txs
        }
    };

    // build the puzzle
    let puzzle = Puzzle {
        // Please fill in the blank
        // Create a puzzle with the block_id of the parent node and the merkle root of the transactions.
        parent: last_block_id.clone().to_owned(),
        merkle_root: merkle_root.clone(),
        reward_receiver: reward_receiver.clone()
    };
    let puzzle_str = serde_json::to_string(&puzzle).unwrap().to_owned();

    // In the end, it returns  (puzzle_str, pre_block);
    (puzzle_str, pre_block)
    
}

/// The struct to represent the Nakamoto instance.
/// The Nakamoto instance contains the chain, the miner, the network and the tx pool as smart pointers.
/// It also contains a FIFO channel for sending transactions to the Blockchain
pub struct Nakamoto {
    /// the chain (BlockTree)
    pub chain_p: Arc<Mutex<BlockTree>>,
    /// the miner
    pub miner_p: Arc<Mutex<Miner>>,
    /// the p2pnetwork
    pub network_p: Arc<Mutex<P2PNetwork>>,
    /// the transaction pool
    pub tx_pool_p: Arc<Mutex<TxPool>>,
    /// the FIFO channel for sending transactions to the Blockchain
    trans_tx: Sender<Transaction>
}

impl Nakamoto {

    /// A function to send notification messages to stdout (For debugging purpose only)
    pub fn stdout_notify(msg: String) {
        let msg = HashMap::from([("Notify".to_string(), msg.clone())]);
        println!("{}", serde_json::to_string(&msg).unwrap());
    }

    /// Create a Nakamoto instance given the serialized chain, tx pool and config as three json strings.
    pub fn create_nakamoto (chain_str: String, tx_pool_str: String, config_str: String) -> Nakamoto {
        // Please fill in the blank
        // Deserialize the config from the given json string.
        let config = serde_json::from_str::<Config>(&config_str).unwrap();
        // Deserialize the chain and the tx pool from the given json strings.
        let chain = serde_json::from_str::<BlockTree>(&chain_str).unwrap();
        let chain_p = Arc::new(Mutex::new(chain));
        let tx_pool = serde_json::from_str::<TxPool>(&tx_pool_str).unwrap();
        let tx_pool_p = Arc::new(Mutex::new(tx_pool));
        // Create the miner and the network according to the config.
        let miner = Miner {
            thread_count: config.miner_thread_count.clone(),
            leading_zero_len: config.difficulty_leading_zero_len.clone(),
            is_running: false
        };
        let miner_p = Arc::new(Mutex::new(miner));
        let (
            network,
            upd_block_in_rx, 
            upd_trans_in_rx,
            block_out_tx,
            trans_out_tx,
            req_block_id_out_tx,
        ) = P2PNetwork::create(config.addr, config.neighbors);

        // Start necessary threads that read from and write to FIFO channels provided by the network.
        // listen to incoming blocks
        let block_out_tx_clone = block_out_tx.clone();
        let chain_p_clone2 = chain_p.clone();
        let cancellation_token_p = Arc::new(RwLock::new(false));
        let cancellation_token_p_clone2 = cancellation_token_p.clone();
        thread::spawn(move || {
            Self::stdout_notify("# Waiting for IPC Requests ...".to_owned());
            let mut cancellation_token_writer = cancellation_token_p_clone2.write().unwrap();
            for block in upd_block_in_rx {
                // it would first check if it is valid including checking whether the block has enough prefix 0 for SHA256(nonce || puzzle)
                let (is_valid, block_id) = block.validate_block(config.difficulty_leading_zero_len_acc);
                // If not valid, drop it
                if (!is_valid) {
                    continue;
                }
                // If valid, it would add it to the blocktree and broadcast and check the current longest path
                // If two paths have the same length, here we consider the one whose last block has the larger hash number as the longest path.
                let prev_working_id = chain_p_clone2.lock().unwrap().working_block_id.clone();
                chain_p_clone2.lock().unwrap().add_block(block.clone(), config.difficulty_leading_zero_len_acc);
                block_out_tx_clone.send(block).unwrap();
                // If the longest path and the last block on this path do not change, it continues its mining
                // If not, it switches to the new longest path, creates a puzzle, and starts solving it by calling cancellation token
                if chain_p_clone2.lock().unwrap().working_block_id.ne(&prev_working_id) {
                    *cancellation_token_writer = true;
                }
            }
        });
        // listen to incoming trans
        let trans_out_tx_clone = trans_out_tx.clone();
        let tx_pool_p_clone2 = tx_pool_p.clone();
        thread::spawn(move || {
            Self::stdout_notify("[Main] Start receiving trans thread".to_owned());
            for trans in upd_trans_in_rx {
                // if tx added successfully, 
                tx_pool_p_clone2.lock().unwrap().add_tx(trans.clone());
                trans_out_tx_clone.send(trans).unwrap();
            }
        });
        // Start necessary thread(s) to control the miner.
        let cancellation_token_p_clone = cancellation_token_p.clone();
        let chain_p_clone = chain_p.clone();
        let miner_p_clone = miner_p.clone();
        let tx_pool_p_clone = tx_pool_p.clone();
        thread::spawn(move || {
            let mut cancellation_token_writer = cancellation_token_p_clone.write().unwrap();

            loop {
                let puzzle_block = create_puzzle(chain_p_clone.clone(), tx_pool_p_clone.clone(), config.max_tx_in_one_block.clone(), config.mining_reward_receiver.clone());
                let puzzle = puzzle_block.0;
                let mut block = puzzle_block.1;
                let solution = Miner::solve_puzzle(miner_p_clone.clone(), puzzle, config.nonce_len.clone(), config.difficulty_leading_zero_len.clone(), config.miner_thread_count.clone(), config.miner_thread_0_seed.clone(), cancellation_token_p_clone.clone());
                match solution {
                    None => {
                        // solution not found aka cancellation_token set to true
                        // reset cancellation_token to false and  gointo the next iteration of the loop to solve the new puzzle
                        *cancellation_token_writer = false;
                    }
                    Some(puzzle_solution) => {
                        // solution found
                        let nonce = puzzle_solution.nonce;
                        let block_id = puzzle_solution.hash;
                        block.header.nonce = nonce;
                        block.header.block_id = block_id;
                        // add it to the local blocktree
                        chain_p_clone.lock().unwrap().add_block(block.clone(), config.difficulty_leading_zero_len_acc);
                        // broadcast it by sending to sender mspc
                        block_out_tx.send(block).unwrap();
                        // create a new puzzle, and solve the new puzzle by going into the next iteration of the loop
                    }
                }
            }
        });
        // Return the Nakamoto instance that holds pointers to the chain, the miner, the network and the tx pool.
        Nakamoto { chain_p: chain_p, miner_p: miner_p, network_p: network, tx_pool_p: tx_pool_p, trans_tx: trans_out_tx }
        // todo!()        
    }

    /// Get the status of the network as a dictionary of strings. For debugging purpose.
    pub fn get_network_status(&self) -> BTreeMap<String, String> {
        self.network_p.lock().unwrap().get_status()
    }

    /// Get the status of the chain as a dictionary of strings. For debugging purpose.
    pub fn get_chain_status(&self) -> BTreeMap<String, String> {
        self.chain_p.lock().unwrap().get_status()
    }

    /// Get the status of the transaction pool as a dictionary of strings. For debugging purpose.
    pub fn get_txpool_status(&self) -> BTreeMap<String, String> {
        self.tx_pool_p.lock().unwrap().get_status()
    }

    /// Get the status of the miner as a dictionary of strings. For debugging purpose.
    pub fn get_miner_status(&self) -> BTreeMap<String, String> {
        self.miner_p.lock().unwrap().get_status()
    }

    /// Publish a transaction to the Blockchain
    pub fn publish_tx(&mut self, transaction: Transaction) -> () {
        // Please fill in the blank
        // Add the transaction to the transaction pool and send it to the broadcast channel
        let trans_clone = transaction.clone();
        self.tx_pool_p.lock().unwrap().add_tx(trans_clone);
        self.trans_tx.send(transaction).unwrap();
    }

    /// Get the serialized chain as a json string. 
    pub fn get_serialized_chain(&self) -> String {
        let chain = self.chain_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&chain).unwrap()
    }

    /// Get the serialized transaction pool as a json string.
    pub fn get_serialized_txpool(&self) -> String {
        let tx_pool = self.tx_pool_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&tx_pool).unwrap()
    }
}

