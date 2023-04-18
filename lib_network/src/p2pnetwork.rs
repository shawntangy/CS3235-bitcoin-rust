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
        let p2pnetwork = Arc::new(Mutex::new(P2PNetwork { send_msg_count: 0, recv_msg_count: 0, address, neighbors: neighbors.clone(), tcps: vec![] }));
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

        // 4. create TCP connections to all neighbors
        for neighbor_netaddress in neighbors {
            // part of 6: listen from TCP channel of initial neighbors
            let p2pnetwork_clone2 = p2pnetwork.clone();
            let block_out_tx_clone = block_out_tx.clone();
            let trans_out_tx_clone = trans_out_tx.clone();
            thread::spawn(move || {
                let mut tcp = NetChannelTCP::from_addr(&neighbor_netaddress).unwrap();
                p2pnetwork_clone2.lock().unwrap().tcps.push(tcp.clone_channel());
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
            });
        }

        // part of 6: listen from blocknode channel and broadcast received blocknode
        let p2pnetwork_clone3 = p2pnetwork.clone();
        thread::spawn(move || {
            for block in &block_out_rx {
                // part of 7: broadcast block to all neighbors
                let mut p2pnetwork_temp = p2pnetwork_clone3.lock().unwrap(); // acquire the lock on the Arc<Mutex<P2PNetwork>> to access its fields
                for tcp in p2pnetwork_temp.tcps.iter_mut() { // iterate through the `tcps` Vec
                    tcp.write_msg(NetMessage::BroadcastBlock(block.clone()));
                }
            }
    
        });
        
        // part of 6: listen from transaction channel and broadcast received transaction
        let p2pnetwork_clone4 = p2pnetwork.clone();
        thread::spawn(move || {
            for trans in &trans_out_rx {
                // part of 7: broadcast trans to all neighbors
                let mut p2pnetwork_temp = p2pnetwork_clone4.lock().unwrap(); // acquire the lock on the Arc<Mutex<P2PNetwork>> to access its fields
                for tcp in p2pnetwork_temp.tcps.iter_mut() { // iterate through the `tcps` Vec
                    tcp.write_msg(NetMessage::BroadcastTx(trans.clone()));
                }
            }
        });


        // 6. create threads to listen to messages from neighbors
        // 5. create threads for each TCP connection to send messages
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)

        // 8. return the created P2PNetwork instance and the mpsc channels
        (p2pnetwork, upd_block_in_rx, upd_trans_in_rx, block_out_tx, trans_out_tx, id_tx)
    
    } 

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        let mut status = BTreeMap::new();
        status.insert("send_msg_count".to_string(), self.send_msg_count.to_string());
        status.insert("recv_msg_count".to_string(), self.recv_msg_count.to_string());
        status.insert("address".to_string(), format!("{:?}", self.address));
        status.insert("neighbors_len".to_string(), format!("{:?}", self.neighbors.len()));
        status.insert("tcps_len".to_string(), format!("{:?}", self.tcps.len()));
        status
    }

}


