// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages. 
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.


use lib_chain::block::{BlockNode, Transaction, BlockId, TxId, self};
use crate::netchannel::*;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::time::Duration;
use rand::Rng;
use std::convert;
use std::io::{Write, Read};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

/// The struct to represent statistics of a peer-to-peer network.
pub struct P2PNetwork {
    /// The number of messages sent by this node.
    pub send_msg_count: u64,
    /// The number of messages received by this node.
    pub recv_msg_count: u64,
    /// The address of this node.
    pub address: NetAddress,
    /// The addresses of the neighbors.
    pub neighbors: Vec<NetAddress>,
    /// Vec of TcpStream
    pub tcps: Vec<NetChannelTCP>,
    // Vec of blocks id sent before
    pub sent_blocks: HashSet<BlockId>,
    // Vec of transactions id sent before
    pub sent_trans: HashSet<TxId>
}


impl P2PNetwork {
    /// Creates a new P2PNetwork instance and associated FIFO communication channels.
    /// There are 5 FIFO channels. 
    /// Those channels are used for communication within the process.
    /// They abstract away the network and neighbor nodes. 
    /// More specifically, they are for communicating between `bin_nakamoto` threads 
    /// and threads that are responsible for TCP network communication.
    /// The usage of those five channels can be guessed from the type:
    /// 1. Receiver<BlockNode>: read from this FIFO channel to receive blocks from the network.
    /// 2. Receiver<Transaction>: read from this FIFO channel to receive transactions from the network.
    /// 3. Sender<BlockNode>: write to this FIFO channel to broadcast a block to the network.
    /// 4. Sender<Transaction>: write to this FIFO channel to broadcast a transaction to the network.
    /// 5. Sender<BlockId>: write to this FIFO channel to request a block from the network.
    pub fn create(address: NetAddress, neighbors: Vec<NetAddress>) -> (
        Arc<Mutex<P2PNetwork>>,
        Receiver<BlockNode>, 
        Receiver<Transaction>, 
        Sender<BlockNode>, 
        Sender<Transaction>,
        Sender<BlockId>
    ) {
        // Please fill in the blank
        // You might need to perform the following steps:
        // 1. create a P2PNetwork instance
        // 2. create mpsc channels for sending and receiving messages
        // 3. create a thread for accepting incoming TCP connections from neighbors
        // 4. create TCP connections to all neighbors
        // 5. create threads for each TCP connection to send messages
        // 6. create threads to listen to messages from neighbors
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
        // 8. return the created P2PNetwork instance and the mpsc channels
        // todo!();

        let mut ip_addr : String = address.ip.to_owned();
        let semi_colon : String = ":".to_owned();
        let port_no : String = address.port.to_string().to_owned();
        ip_addr.push_str(&semi_colon);
        ip_addr.push_str(&port_no);
        // 1. create a P2PNetwork instance
        let p2pnetwork = Arc::new(Mutex::new(P2PNetwork { send_msg_count: 0, recv_msg_count: 0, address, neighbors: neighbors.clone(), tcps: vec![], sent_blocks: HashSet::new(), sent_trans: HashSet::new()}));
        // 2. create mpsc channels for sending and receiving messages
        let (upd_block_in_tx, upd_block_in_rx) = mpsc::channel::<BlockNode>();
        let (upd_trans_in_tx, upd_trans_in_rx) = mpsc::channel::<Transaction>();
        let (block_out_tx, block_out_rx) = mpsc::channel::<BlockNode>();
        let (trans_out_tx, trans_out_rx) = mpsc::channel::<Transaction>();
        let (id_tx, id_rx) = mpsc::channel::<BlockId>();

        // 3. create a thread for accepting incoming TCP connections from neighbors
        let listener = TcpListener::bind(ip_addr.clone()).unwrap();     
        let p2pnetwork_clone = p2pnetwork.clone();
        let block_out_tx_clone = block_out_tx.clone();
        let trans_out_tx_clone = trans_out_tx.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut tcp = NetChannelTCP::from_stream(stream.unwrap());
                p2pnetwork_clone.lock().unwrap().tcps.push(tcp.clone_channel());
                let block_out_tx_clone2 = block_out_tx_clone.clone();
                let trans_out_tx_clone2 = trans_out_tx_clone.clone();
                // part of 6: listen from TCP channel of new neighbors
                thread::spawn(move || {
                    loop {
                        let message = tcp.read_msg().unwrap();
                        match message {
                            NetMessage::BroadcastBlock(block) => {
                                block_out_tx_clone2.send(block).unwrap();
                            }
                            NetMessage::BroadcastTx(trans) => {
                                trans_out_tx_clone2.send(trans).unwrap();
                            }
                            NetMessage::RequestBlock(_) => todo!(),
                            NetMessage::Unknown(_) => todo!(),
                        };
                    }
                });
            }
        });

        fn random_duration() -> Duration {
            let mut rng = rand::thread_rng();
            let msecs = rng.gen_range(1..=5000);
            Duration::from_millis(msecs)
        }

        // 4. create TCP connections to all neighbors
        for neighbor_netaddress in neighbors {
            // part of 6: listen from TCP channel of initial neighbors
            let p2pnetwork_clone2 = p2pnetwork.clone();
            let block_out_tx_clone = block_out_tx.clone();
            let trans_out_tx_clone = trans_out_tx.clone();
            thread::spawn(move || {
                //let mut tcp = NetChannelTCP::from_addr(&neighbor_netaddress).unwrap();
                
                if let Ok(mut tcp) = NetChannelTCP::from_addr_retry(&neighbor_netaddress, 5, random_duration()) {
                    // Handle successful connection
                    eprintln!("Successful connection to {}:{}",&neighbor_netaddress.ip,&neighbor_netaddress.port);
                    p2pnetwork_clone2
                        .lock()
                        .unwrap()
                        .tcps
                        .push(tcp.clone_channel());
                    loop {
                        let message = tcp.read_msg().unwrap();
                        // part of 7: broadcast message to all neighbors by sending to mpsc which will in turn send to neighbor
                        match message {
                            NetMessage::BroadcastBlock(block) => {
                                block_out_tx_clone.send(block).unwrap();

                            }
                            NetMessage::BroadcastTx(trans) => {
                                trans_out_tx_clone.send(trans).unwrap();
                            }
                            NetMessage::RequestBlock(_) => todo!(),
                            NetMessage::Unknown(_) => todo!(),
                        };
                    }
                } else {
                    eprintln!("Failed to connect to neighbor");
                    // Handle connection error
                }
            });
        }
        eprintln!("[P2PNetwork] All neighbors connected.");
        eprintln!("[P2PNetwork] Starting processing received messages thread.");
        // part of 6: listen from blocknode channel and broadcast received blocknode
        eprintln!("[P2PNetwork] Starting broadcasting blocks thread.");
        let p2pnetwork_clone3 = p2pnetwork.clone();
        thread::spawn(move || {
            //loop {
                for block in &block_out_rx {
                    // part of 7: broadcast block to all neighbors
                    let mut p2pnetwork_temp = p2pnetwork_clone3.lock().unwrap(); // acquire the lock on the Arc<Mutex<P2PNetwork>> to access its fields
                    p2pnetwork_temp.recv_msg_count += 1;
                    // broadcast only if never send this block before
                    if (!p2pnetwork_temp.sent_blocks.contains(&block.header.block_id.clone())) {
                        for tcp in p2pnetwork_temp.tcps.iter_mut() { // iterate through the `tcps` Vec
                            tcp.write_msg(NetMessage::BroadcastBlock(block.clone()));
                        }
                        upd_block_in_tx.send(block.clone()).unwrap();
                        p2pnetwork_temp.sent_blocks.insert(block.header.block_id.clone());
                        p2pnetwork_temp.send_msg_count += 1;
                    }
                }
            //}
        });
        
        // part of 6: listen from transaction channel and broadcast received transaction
        eprintln!("[P2PNetwork] Starting broadcasting transactions thread.");
        let p2pnetwork_clone4 = p2pnetwork.clone();
        thread::spawn(move || {
            //loop {
                for trans in &trans_out_rx {
                    //eprintln!("{:?}",trans);
                    // part of 7: broadcast trans to all neighbors
                    let mut p2pnetwork_temp = p2pnetwork_clone4.lock().unwrap(); // acquire the lock on the Arc<Mutex<P2PNetwork>> to access its fields
                    p2pnetwork_temp.recv_msg_count += 1;
                    // broadcast only if never send this trans before
                    if (!p2pnetwork_temp.sent_trans.contains(&trans.gen_hash())) {
                        for tcp in p2pnetwork_temp.tcps.iter_mut() { // iterate through the `tcps` Vec
                            tcp.write_msg(NetMessage::BroadcastTx(trans.clone()));
                        }
                        upd_trans_in_tx.send(trans.to_owned()).unwrap();
                        p2pnetwork_temp.sent_trans.insert(trans.gen_hash());
                        p2pnetwork_temp.send_msg_count += 1;
                    }
                }
            //}
        });


        // 6. create threads to listen to messages from neighbors
        // 5. create threads for each TCP connection to send messages
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)

        // 8. return the created P2PNetwork instance and the mpsc channels
        (p2pnetwork, upd_block_in_rx, upd_trans_in_rx, block_out_tx, trans_out_tx, id_tx)
    
    } 

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network. 
        // It should be displayed in the Client UI eventually.
        let mut status_map = BTreeMap::new();
        status_map.insert("#address".to_string(), format!("ip: {} port: {}", self.address.ip, self.address.port));
        status_map.insert("#recv_msg".to_string(), self.recv_msg_count.to_string());
        status_map.insert("#send_msg".to_string(), self.send_msg_count.to_string());
        status_map
    }

}


