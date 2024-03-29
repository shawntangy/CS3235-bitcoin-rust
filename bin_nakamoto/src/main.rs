// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the main file of the bin_nakamoto executable.
/// It is a simple command-line program that can be used to interact with the Blockchain
/// It reads commands from stdin and writes responses to stdout to facilitate IPC communication with bin_client eventually.
/// However, you can also run it directly from the command line to test it.
/// You can see detailed instructions in the comments below.


mod nakamoto;
use lib_chain::block::{Transaction, Signature};
use nakamoto::Nakamoto;
use seccompiler::BpfMap;

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::fs;
use serde::{Serialize, Deserialize};

// Read a string from a file (to help you debug)
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath)
        .expect(&("Cannot read ".to_owned() + filepath));
    contents
}

// Append a string to a file (to help you debug)
fn append_string_to_file(filepath: &str, content: String) {
    // if not exists, create file
    if !std::path::Path::new(filepath).exists() {
        fs::File::create(filepath).unwrap();
    }
    fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
}

/// This enum represents IPC messsage requests from the stdin
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReq {
    /// Initialize the Nakamoto instance using the given (blocktree_json, tx_pool_json, config_json)
    Initialize(String, String, String),
    /// Get the balance of the given address (user_id)
    GetAddressBalance(String),
    /// Publish a transaction to the network (data_string, signature)
    PublishTx(String, Signature),
    /// Get the block data of the given block_id
    RequestBlock(String),
    /// Get the network status (for debugging)
    RequestNetStatus,
    /// Get the chain status (for debugging)
    RequestChainStatus,
    /// Get the miner status (for debugging)
    RequestMinerStatus,
    /// Get the tx pool status (for debugging)
    RequestTxPoolStatus,
    /// Get the state serialization (including BlockTree and TxPool)
    RequestStateSerialization,
    /// Quit the program
    Quit,
}

/// This enum represents IPC messsage responses to the stdout
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageResp {
    /// The Nakamoto instance has been initialized (responding to Initialize)
    Initialized,
    /// The transaction has been published (responding to PublishTx)
    PublishTxDone,
    /// The balance of the given address (user_id, balance)
    AddressBalance(String, i64),
    /// The block data of the given block_id (block_data)
    BlockData(String),
    /// The network status as a dictionary of strings (for debugging)
    NetStatus(BTreeMap<String, String>),
    /// The chain status as a dictionary of strings (for debugging)
    ChainStatus(BTreeMap<String, String>),
    /// The miner status as a dictionary of strings (for debugging)
    MinerStatus(BTreeMap<String, String>),
    /// The tx pool status as a dictionary of strings (for debugging)
    TxPoolStatus(BTreeMap<String, String>),
    /// The state serialization (blocktree_json_string, tx_pool_json_string)
    StateSerialization(String, String),
    /// The program is quitting (responding to Quit)
    Quitting,
    /// This is not an actual response, but an arbitrary notification message for debugging
    Notify(String), 
}

fn main() {
    // bin_nakamoto has only one optional argument: the path to the seccomp policy file
    // If the argument is provided, bin_nakamoto will read and apply the seccomp policy at the beginning of the program
    // Otherwise, it will proceed to the normal execution

    let maybe_policy_path = std::env::args().nth(1);
    if let Some(policy_path) = maybe_policy_path { 
        // Please fill in the blank
        // If the first param is provided, read the seccomp config and apply it
        let policy = read_string_from_file(&policy_path);
        let filter_map: BpfMap = seccompiler::compile_from_json(policy.as_bytes(), std::env::consts::ARCH.try_into().unwrap()).unwrap();
        let filter = filter_map.get("main_thread").unwrap();
        seccompiler::apply_filter(&filter).unwrap();
    }

    // The main logic of the bin_nakamoto starts here
    // It reads IPC calls from stdin and write IPC responses to stdout in a loop.
    // The first IPC call should be Initialize, whose parameters are serialized BlockTree, TxPool, and Config.
    // After that, there can be artitrary number of IPC calls, including GetAddressBalance, PublishTx, RequestBlock, RequestNetStatus, RequestChainStatus, RequestMinerStatus, RequestTxPoolStatus, RequestStateSerialization, etc.
    // Eventually, the program will quit when receiving a Quit IPC call.
    // Please fill in the blank
    // Loop over stdin and handle IPC messages
    
    let mut stdin_input = String::new();  
    io::stdin().read_line(&mut stdin_input).unwrap();
    let ipc_msg_req : IPCMessageReq = serde_json::from_str(&stdin_input).unwrap();
    match ipc_msg_req {
        IPCMessageReq::Initialize(blocktree_json, tx_pool_json, config_json) => {
            // do something with the three strings
            let mut nakamoto = Nakamoto::create_nakamoto(blocktree_json, tx_pool_json, config_json);
            //craft into string
            let resp_str = serde_json::to_string(&IPCMessageResp::Initialized).unwrap();
            println!("{}", resp_str);
            loop {
                let mut stdin_input = String::new();  
                io::stdin().read_line(&mut stdin_input).unwrap();
                let ipc_msg_req : IPCMessageReq = serde_json::from_str(&stdin_input).unwrap();
                let ipc_msg_resp = match ipc_msg_req {
                    IPCMessageReq::Initialize(blocktree_json, tx_pool_json, config_json) => {
                        eprintln!("nakamoto already initialized but will re-initalized again here!");
                        let nakamoto = Nakamoto::create_nakamoto(blocktree_json, tx_pool_json, config_json);
                        IPCMessageResp::Initialized
                    }
                    IPCMessageReq::GetAddressBalance(user_id) => {
                        match nakamoto.chain_p.lock().unwrap().finalized_balance_map.get(&user_id) {
                            Some(x) => {
                                IPCMessageResp::AddressBalance(user_id, x.clone())
                            }
                            None => {
                                IPCMessageResp::AddressBalance(user_id, 0)
                            }
                        }
                    }
                    IPCMessageReq::PublishTx(data_string, signature) => {
                        // what is signature used for?
                        let data: Vec<String> = serde_json::from_str(&data_string).unwrap();
                        let tx = Transaction {
                            sender : data[0].to_owned(),
                            receiver : data[1].to_owned(),
                            message : data[2].to_owned(),
                            sig : signature,
                        };
                        nakamoto.publish_tx(tx);
                        IPCMessageResp::PublishTxDone
                    }
                    IPCMessageReq::RequestBlock(block_id) => {
                        // is this correct?
                        let block = nakamoto.chain_p.lock().unwrap().get_block(block_id);
                        let block_data = serde_json::to_string(&block).unwrap();
                        IPCMessageResp::BlockData(block_data)
                    }
                    IPCMessageReq::RequestNetStatus => {
                        let status = nakamoto.get_network_status();
                        IPCMessageResp::NetStatus(status)
                    }
                    IPCMessageReq::RequestChainStatus => {
                        let status = nakamoto.get_chain_status();
                        IPCMessageResp::ChainStatus(status)
                    }
                    IPCMessageReq::RequestMinerStatus => {
                        let status = nakamoto.get_miner_status();
                        IPCMessageResp::MinerStatus(status)
                    }
                    IPCMessageReq::RequestTxPoolStatus => {
                        let status = nakamoto.get_txpool_status();
                        IPCMessageResp::TxPoolStatus(status)
                    }
                    IPCMessageReq::RequestStateSerialization => {
                        let serialized_chain = nakamoto.get_serialized_chain();
                        let serialized_txpool = nakamoto.get_serialized_txpool();
                        IPCMessageResp::StateSerialization(serialized_chain, serialized_txpool)
                    }
                    IPCMessageReq::Quit => {
                        break;
                    }
                };
                //craft into string
                let resp_str = serde_json::to_string(&ipc_msg_resp).unwrap();
                println!("{}", resp_str);
            }
            //craft into string
            let resp_str = serde_json::to_string(&IPCMessageResp::Quitting).unwrap();
            println!("{}", resp_str);
        }
        _ => {
            eprintln!("1st call must be initialize!");
        }
    }

}


// Syscall used

// strace -f -o ./syscall/test_nakamoto_follow.strace ./target/debug/bin_nakamoto < ./tests/cli_test_nakamoto/cli_test_nakamoto_0.input 
// cat ./syscall/test_nakamoto_follow.strace | grep -oE '^[^\(]*?\(' | sort | uniq | sed 's/.$//'

// 5933  access
// 5933  arch_prctl
// 5933  bind
// 5933  brk
// 5933  clone3
// 5933  close
// 5933  execve
// 5933  exit_group
// 5933  futex
// 5933  <... futex resumed>)              = -1 EAGAIN 
// 5933  getrandom
// 5933  listen
// 5933  mmap
// 5933  mprotect
// 5933  munmap
// 5933  newfstatat
// 5933  openat
// 5933  poll
// 5933  pread64
// 5933  prlimit64
// 5933  read
// 5933  rseq
// 5933  rt_sigaction
// 5933  rt_sigprocmask
// 5933  sched_getaffinity
// 5933  set_robust_list
// 5933  setsockopt
// 5933  set_tid_address
// 5933  sigaltstack
// 5933  socket
// 5933  write
// 5934  accept4
// 5934  mmap
// 5934  mprotect
// 5934  rseq
// 5934  rt_sigprocmask
// 5934  sched_getaffinity
// 5934  set_robust_list
// 5934  sigaltstack
// 5935  futex
// 5935  mmap
// 5935  mprotect
// 5935  rseq
// 5935  rt_sigprocmask
// 5935  sched_getaffinity
// 5935  set_robust_list
// 5935  sigaltstack
// 5936  futex
// 5936  <... futex resumed>)              = -1 EAGAIN 
// 5936  mmap
// 5936  mprotect
// 5936  munmap
// 5936  rseq
// 5936  rt_sigprocmask
// 5936  sched_getaffinity
// 5936  set_robust_list
// 5936  sigaltstack
// 5936  write
// 5937  exit
// 5937  getrandom
// 5937  madvise
// 5937  mmap
// 5937  mprotect
// 5937  munmap
// 5937  rseq
// 5937  rt_sigprocmask
// 5937  sched_getaffinity
// 5937  set_robust_list
// 5937  sigaltstack
// 5937  write
// 5938  exit
// 5938  futex
// 5938  <... futex resumed>)              = -1 EAGAIN 
// 5938  getrandom
// 5938  madvise
// 5938  mmap
// 5938  mprotect
// 5938  munmap
// 5938  rseq
// 5938  rt_sigprocmask
// 5938  sched_getaffinity
// 5938  set_robust_list
// 5938  sigaltstack
// 5938  write
// 5939  futex
// 5939  <... futex resumed>)              = -1 EAGAIN 
// 5939  getrandom
// 5939  mmap
// 5939  mprotect
// 5939  munmap
// 5939  rseq
// 5939  rt_sigprocmask
// 5939  sched_getaffinity
// 5939  set_robust_list
// 5939  sigaltstack
