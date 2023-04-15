// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages. 
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.


use lib_chain::block::{BlockNode, Transaction, BlockId, TxId};
use crate::netchannel::*;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::convert;
use std::io::{Write, Read};
use std::net::TcpListener;
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
        todo!();

        /* Yi Da's poor man attemp 
        let mut ip_addr : String = address.ip.to_owned();
        let semi_colon : String = ":".to_owned();
        let port_no : String = address.port.to_string().to_owned();
        ip_addr.push_str(&semi_colon);
        ip_addr.push_str(&port_no);
        // 1. create a P2PNetwork instance
        let p2pnetwork = P2PNetwork { send_msg_count: 0, recv_msg_count: 0, address, neighbors: neighbors.clone() };
        // 2. create mpsc channels for sending and receiving messages
        let (upd_block_in_tx, upd_block_in_rx) = mpsc::channel::<BlockNode>();
        let (upd_trans_in_tx, upd_trans_in_rx) = mpsc::channel::<Transaction>();
        let (block_out_tx, block_out_rx) = mpsc::channel::<BlockNode>();
        let (trans_out_tx, trans_out_rx) = mpsc::channel::<Transaction>();
        let (id_tx, id_rx) = mpsc::channel::<BlockId>();

        // 4. create TCP connections to all neighbors
        let mut tcps = vec![];
        let mut tcps_2 = vec![];
        for neighbor_netaddress in neighbors {
            let tcp = NetChannelTCP::from_addr(&neighbor_netaddress).unwrap();
            tcps.push(tcp.clone_channel());
            tcps_2.push(tcp.clone_channel());
        }
        

        // 3. create a thread for accepting incoming TCP connections from neighbors
        // let listener = TcpListener::bind(ip_addr.clone()).unwrap();     
        // thread::spawn(move || {
        //     for stream in listener.incoming() {
        //         let tcp = NetChannelTCP::from_stream(stream.unwrap());
        //         tcps.push(tcp);
        //     }
        // });

        // 6. create threads to listen to messages from neighbors
        // part of 6: listen from TCP channel
        /*
        let stream_copy = stream.unwrap().try_clone().unwrap();
        let mut net_channel_tcp = NetChannelTCP::from_stream(stream_copy);
        thread::spawn(move || {
            loop {
                let message = net_channel_tcp.read_msg().unwrap();
            }
        });
        */

        // part of 6: listen from blocknode channel and broadcast received blocknode
        thread::spawn(move || {
            loop {
                for block in &block_out_rx {
                    // broadcast block to all neighbors 
                    for mut tcp in tcps {
                        let msg = NetMessage::BroadcastBlock(block.clone());
                        tcp.write_msg(msg);
                    }
                }
            }
        });
        let listener = TcpListener::bind(ip_addr).unwrap();
        // part of 6: listen from transaction channel and broadcast received transaction
        thread::spawn(move || {
            loop {
                for trans in &trans_out_rx {
                    // broadcast trans to all neighbors
                    let mut jsonstr: String = serde_json::to_string(&trans).unwrap();
                    jsonstr.push('\n');
                    for stream in listener.incoming() {
                        stream.unwrap().write_all(jsonstr.clone().as_bytes());
                    }
                }
            }
        });
            
        // 5. create threads for each TCP connection to send messages
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
        // 8. return the created P2PNetwork instance and the mpsc channels
        (Arc::new(Mutex::new(p2pnetwork)), upd_block_in_rx, upd_trans_in_rx, block_out_tx, trans_out_tx, id_tx)
        */
    } 

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network. 
        // It should be displayed in the Client UI eventually.
        todo!();
        
    }

}


