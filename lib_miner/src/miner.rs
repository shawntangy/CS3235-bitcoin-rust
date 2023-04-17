// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Miner struct and related methods. 
// The miner has one key task: to solve a given puzzle (a string) with specified number of threads and difficulty levels.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the miner.

use std::{thread, convert};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};
use rand_pcg::Pcg32;
use rand::{Rng, SeedableRng, distributions::{Alphanumeric, DistString}};
use sha2::{Sha256, Digest};


// A miner that solve puzzles.
pub struct Miner {
    /// number of threads used to solve the puzzle in parallel
    pub thread_count: u16,

    /// number of leading "0"s expected in the resulting hash string in hex format.
    /// e.g. if leading_zero_len is 3, then the hash string should start with "000"
    /// and the difficulty level is 3.
    pub leading_zero_len: u16,

    /// whether the miner is running or not
    pub is_running: bool
}

type BlockId = String;

/// The struct to represent a puzzle solution returned by the miner.
pub struct PuzzleSolution {
    /// the puzzle string
    pub puzzle: String,
    /// the nonce string that should be prepended to the puzzle string for computing the hash
    pub nonce: String,
    /// the sha256 hash of (nonce || puzzle) in hex format
    pub hash: BlockId
}

impl Miner {
    // constructor
    pub fn new () -> Miner {
        Miner { 
            thread_count: 0,
            leading_zero_len: 0,
            is_running: false
        }
    }
    
    /// The method to solve a puzzle with specified number of threads and difficulty levels.
    /// This method is a function on the class (without `self` as the 1st argument). The first parameter is a smart pointer to a miner instance.
    /// - `miner_p`: the smart pointer to the miner instance
    /// - `puzzle`: the puzzle string
    /// - `nonce_len`: the length of the nonce string in the solution. The nonce string should be randomly generated from the alphanumeric characters A-Z, a-z and 0-9.
    /// - `leading_zero_len`: the number of leading "0"s expected in the resulting hash string in hex format.
    /// - `thread_count`: the number of threads to be used for solving the puzzle in parallel.
    /// - `thread_0_seed`: the seed for the random number generator for the first thread. The seed for the second thread should be `thread_0_seed + 1`, and so on.
    /// - `cancellation_token`: a smart pointer to a boolean value. If the value is set to true, all threads should stop even if they have not found a solution.
    /// - return: an optional value with the solution if the puzzle is solved, or None if the puzzle is cancelled.
    pub fn solve_puzzle(miner_p: Arc<Mutex<Miner>>, puzzle: String, nonce_len: u16, leading_zero_len: u16, thread_count: u16, thread_0_seed: u64, cancellation_token: Arc<RwLock<bool>>) -> Option<PuzzleSolution> {
        
        // In this function, you are expected to start multiple threads for solving the puzzle.
        // The threads should be spawned and joined in this function.
        // If any of the threads finds a solution, other threads should stop.
        // Additionally, if the cancellation_token is set to true, all threads should stop.
        // The purpose of the cancellation_token is to allow the miner to stop the computation when other nodes have already solved the exact same puzzle.
        let mut handles = vec![];

        // Create a channel to communicate between threads.
        let (tx, rx) = std::sync::mpsc::channel::<Option<PuzzleSolution>>();
        let rx = Arc::new(Mutex::new(rx));
        // Spawn threads to search for a solution.
        for i in 0..thread_count {
            let tx = tx.clone();
            //let rx = rx.clone();
            let miner_p = miner_p.clone();
            let cancellation_token = cancellation_token.clone();
            let puzzle = puzzle.clone();
            // set rng seed from function
            let mut rng = Pcg32::seed_from_u64(thread_0_seed + u64::from(i));
            let handle = thread::spawn(move || {                
                loop {
                    
                    // Check if the cancellation_token is true to break the loop
                    if *cancellation_token.read().unwrap() {
                        break;
                    }

                    let nonce = Alphanumeric.sample_string(&mut rng, nonce_len.into());

                    // Combine nonce and puzzle to get hash
                    let combined_hash = format!("{}{}", nonce, puzzle);
                    let hash = Sha256::digest(combined_hash.as_bytes());
                    
                    // Check if the hash satisfies the difficulty level.
                    let hash_str = format!("{:x}", hash);
                    //println!("supposed hash_str: {}",hash_str);
                    if hash_str.starts_with(&"0".repeat(leading_zero_len as usize)) {
                        println!("Matched: {}",hash_str);
                        let mut token = cancellation_token.write().unwrap();
                        *token = true;
                        let ans = PuzzleSolution{
                            puzzle,
                            nonce,
                            hash: hash_str
                        };
                        tx.send(Some(ans))
                        .unwrap();
                        break;
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for the threads to finish.
        for handle in handles {
            handle.join().unwrap();
        }

        // Get the first solution found by any thread.
        let x = if let Ok(solution) = rx.lock().unwrap().try_recv() {
            solution
        }else{
            None
        }; x
        
        
    } 
    
    /// Get status information of the miner for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // For debugging purpose, you can return any dictionary of strings as the status of the miner. 
        // It should be displayed in the Client UI eventually.
        let mut miner_status = BTreeMap::new();
        miner_status.insert("#thread".to_string(), self.thread_count.to_string());
        miner_status.insert("difficulty".to_string(), self.leading_zero_len.to_string());
        miner_status.insert("is_running".to_string(), self.is_running.to_string());
        miner_status
        // this is not done cause idk what to put inside
        
    }
}


