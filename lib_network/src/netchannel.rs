// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file mainly implements the NetChannelTCP struct and related methods.
// The NetChannelTCP struct is used to send and receive messages over the network.
// The message format is defined in the NetMessage enum.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the NetChannelTCP.


use std::str::FromStr;
use std::{io::BufRead};
use lib_chain::block::{BlockNode, Transaction, BlockId};
use std::{hash::Hash};
use serde::{Serialize, Deserialize};
use std::net::{TcpStream, SocketAddr, SocketAddrV4};
use std::io::{Read, Write};
use std::io::BufReader;

/// The struct to represent a network address.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct NetAddress {
    /// the ip address. Example: "127.0.0.1"
    pub ip: String,
    /// the port number. Example: 8000
    pub port: i32
}

impl NetAddress {
    pub fn new(ip: String, port: i32) -> NetAddress {
        NetAddress { ip, port }
    }
}


/// The enum to represent a network message that is sent or received using `NetChannelTCP`.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum NetMessage {
    /// The message to broadcast a block to the neighbor.
    BroadcastBlock(BlockNode),
    /// The message to broadcast a transaction to the neighbor.
    BroadcastTx(Transaction),
    /// The message to request a block (i.e. missing in the local block tree) from neighbor.
    RequestBlock(BlockId),
    /// The message to represent other temporary messages (e.g. for debugging).
    Unknown(String)
}

/// The struct to represent a network channel that is used to send and receive messages to a neighbor node.
pub struct NetChannelTCP {
    /// The TCP stream
    stream: TcpStream,
    /// The reader to read from the TCP stream
    reader: BufReader<TcpStream>
}

impl NetChannelTCP {
    /// Create a new NetChannelTCP from a NetAddress and establish the TCP connection.
    /// Return an error string if the connection fails.
    pub fn from_addr(addr: &NetAddress) -> Result<Self,String> {
        // Please fill in the blank
        //todo!();
        match TcpStream::connect(format!("{}:{}", addr.ip, addr.port)) {
            Ok(stream) => {
                let reader = BufReader::new(stream.try_clone().unwrap());
                Ok(Self { stream, reader })
            },
            Err(e) => Err(e.to_string())
        }
    }

    /// Create a new NetChannelTCP from a TcpStream. 
    /// This is useful for creating a NetChannelTCP instance from the listener side.
    pub fn from_stream(stream: TcpStream) -> Self {
        // Please fill in the blank
        // todo!();
        let buf_reader = BufReader::new(stream.try_clone().unwrap());
        NetChannelTCP {
            stream: stream,
            reader: buf_reader,
        }
    }

    /// Clone the NetChannelTCP instance.
    /// This is useful if you have multiple threads dealing with reading and writing to the TCP channel.
    pub fn clone_channel(&mut self) -> Self {
        // Please fill in the blank
        // todo!();
        NetChannelTCP {
            stream: self.stream.try_clone().unwrap(),
            reader: BufReader::new(self.stream.try_clone().unwrap())
        }
    }

    /// Read one line of message from the TCP stream.
    /// Return None if the stream is closed.
    /// Otherwise, parse the line as a NetMessage and return it.
    pub fn read_msg(&mut self) -> Option<NetMessage> {
        // Please fill in the blank
        //todo!();
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                match serde_json::from_str::<NetMessage>(&line) {
                    Ok(msg) => Some(msg),
                    Err(_) => Some(NetMessage::Unknown(line)),
                }
            },
            Err(_) => None,
        }
    }

    /// Write a NetMessage to the TCP stream.
    /// The message is serialized to a one-line JSON string and a newline is appended in the end.
    pub fn write_msg(&mut self, msg: NetMessage) -> () {
        // Please fill in the blank
        //todo!();
        let mut jsonstr: String = serde_json::to_string(&msg).unwrap();
        jsonstr.push('\n');
        self.stream.write_all(jsonstr.as_bytes()).unwrap();
    }
}



