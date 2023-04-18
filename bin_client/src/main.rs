// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the client program that covers the following tasks:
/// 1. File I/O. Read the config file and state files for initialization, dump the state files, etc.
/// 2. Read user input (using terminal UI) about transaction creation or quitting.
/// 3. Display the status and logs to the user (using terminal UI).
/// 4. IPC communication with the bin_nakamoto and the bin_wallet processes.

use seccompiler;
use seccompiler::{BpfProgram, BpfMap};

use tui::{backend::CrosstermBackend, Terminal};
use tui_textarea::{Input, Key};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::fs::{File, read};
use std::io::{self, Read, Write, BufReader, BufRead};
use std::process::{Command, Stdio};
use std::collections::BTreeMap;
use std::time::SystemTime;

use std::{thread, time::{Duration, Instant}};
use std::sync::{Mutex, Arc};
use serde::{Serialize, Deserialize};
use serde_json;

use std::{fs, env};

mod app;

/// The enum type for the IPC messages (requests) from this client to the bin_nakamoto process.
/// It is the same as the `IPCMessageRequest` enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReqNakamoto {
    Initialize(String, String, String),
    GetAddressBalance(String),
    PublishTx(String, String),
    RequestBlock(String),
    RequestNetStatus,
    RequestChainStatus,
    RequestMinerStatus,
    RequestTxPoolStatus,
    RequestStateSerialization,
    Quit,
}

/// The enum type for the IPC messages (responses) from the bin_nakamoto process to this client.
/// It is the same as the enum type in the bin_nakamoto process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespNakamoto {
    Initialized,
    PublishTxDone,
    AddressBalance(String, i64),
    BlockData(String),
    NetStatus(BTreeMap<String, String>),
    ChainStatus(BTreeMap<String, String>),
    MinerStatus(BTreeMap<String, String>),
    TxPoolStatus(BTreeMap<String, String>),
    StateSerialization(String, String),
    Quitting,
    Notify(String), 
}

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

/// The enum type for the IPC messages (responses) from the bin_wallet process to this client.
/// It is the same as the enum type in the bin_wallet process.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageRespWallet {
    Initialized,
    Quitting,
    SignResponse(String, String),
    VerifyResponse(bool, String),
    UserInfo(String, String),
}

/// The enum type representing bot commands for controlling the client automatically.
/// The commands are read from a file or a named pipe and then executed by the client.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum BotCommand {
    /// Send a transaction message from the default user_id of the client to the given receiver_user_id, e.g, Send(`receiver_user_id`, `transaction_message`)
    Send(String, String),
    /// Wait for the given number of milliseconds, e.g., SleepMs(`milliseconds`)
    SleepMs(u64),
}

/// Read a file and return the content as a string.
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath)
        .expect(&("Cannot read ".to_owned() + filepath));
    contents
}

/// A flag indicating whether to disable the UI thread if you need to check some debugging outputs that is covered by the UI. 
/// Eventually this should be set to false and you shouldn't output debugging information directly to stdout or stderr.
const NO_UI_DEBUG_NODE: bool = false;

fn main() {
    // The usage of bin_client is as follows:
    // bin_client <client_seccomp_path> <nakamoto_config_path> <nakamoto_seccomp_path> <wallet_config_path> <wallet_seccomp_path> [<bot_command_path>]
    // - `client_seccomp_path`: The path to the seccomp file for this client process for Part B. (You can set this argument to any value during Part A.)
    // - `nakamoto_config_path`: The path to the config folder for the bin_nakamoto process. For example, `./tests/nakamoto_config1`. Your program should read the 3 files in the config folder (`BlockTree.json`, `Config.json`, `TxPool.json`) for initializing bin_nakamoto.
    // - `nakamoto_seccomp_path`: The path to the seccomp file for the bin_nakamoto process for Part B. (You can set this argument to any value during Part A.)
    // - `wallet_config_path`: The path to the config file for the bin_wallet process. For example, `./tests/_secrets/Walley.A.json`. Your program should read the file for initializing bin_wallet.
    // - `wallet_seccomp_path`: The path to the seccomp file for the bin_wallet process for Part B. (You can set this argument to any value during Part A.)
    // - [`bot_command_path`]: *Optional* argument. The path to the file or named pipe for the bot commands. If this argument is provided, your program should read commands line-by-line from the file.
    //                         an example file of the bot commands can be found at `./tests/_bots/botA-0.jsonl`. You can also look at `run_four.sh` for an example of using the named pipe version of this argument.
    //                         The bot commands are executed by the client in the order they are read from the file or the named pipe. 
    //                         The bot commands should be executed in a separate thread so that the UI thread can still be responsive.
    
    // Please fill in the blank
    // - Create bin_nakamoto process:  Command::new("./target/debug/bin_nakamoto")...
    let mut bin_nakamoto = Command::new("./target/debug/bin_nakamoto").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    // - Create bin_wallet process:  Command::new("./target/debug/bin_wallet")...
    let mut bin_wallet = Command::new("./target/debug/bin_wallet").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    
    // - Get stdin and stdout of those processes
    let bin_nakamoto_stdin = bin_nakamoto.stdin.take().unwrap();
    let bin_nakamoto_stdout = bin_nakamoto.stdout.take().unwrap();

    let bin_wallet_stdin = bin_wallet.stdin.take().unwrap();
    let bin_wallet_stdout = bin_wallet.stdout.take().unwrap();

    // - Create buffer readers if necessary
    let mut bin_nakamoto_buf_reader = BufReader::new(bin_nakamoto_stdout);
    let mut bin_wallet_buf_reader = BufReader::new(bin_wallet_stdout);

    // - Send initialization requests to bin_nakamoto and bin_wallet
    // - Init request code for bin_nakamoto
    let nakamoto_config_path = std::env::args().nth(2).expect("PLease specify nakamoto config path");
    
    let blocktree_path = nakamoto_config_path.clone();
    blocktree_path.push_str("/BlockTree.json");
    let blocktree_json = read_string_from_file(&blocktree_path);
    
    let tx_pool_path = nakamoto_config_path.clone();
    tx_pool_path.push_str("/TxPool.json");
    let tx_pool_json = read_string_from_file(&tx_pool_path);

    let config_path = nakamoto_config_path.clone();
    config_path.push_str("/Config.json");
    let config_json = read_string_from_file(&config_path);

    let nakamoto_init_req = IPCMessageReqNakamoto::Initialize(blocktree_json, tx_pool_json, config_json);
    let mut nakamoto_init_req_str = serde_json::to_string(&nakamoto_init_req).unwrap();
    nakamoto_init_req_str.push('\n');
    bin_nakamoto_stdin.write_all(nakamoto_init_req_str.as_bytes()).unwrap();

    // - Init request code for bin_wallet
    let wallet_config_path = std::env::args().nth(4).expect("Please specify wallet config path");
    let wallet_json = read_string_from_file(&wallet_config_path);
    let wallet_init_req = IPCMessageReqWallet::Initialize(wallet_json);
    let mut wallet_init_req_str = serde_json::to_string(&wallet_init_req).unwrap();
    wallet_init_req_str.push('\n');
    bin_wallet_stdin.write_all(wallet_init_req_str.as_bytes()).unwrap();

    let client_seccomp_path = std::env::args().nth(1).expect("Please specify client seccomp path");
    // Please fill in the blank
    // sandboxing the bin_client (For part B). Leave it blank for part A.
    

    // Please fill in the blank
    // Read the user info from wallet
    let mut user_name: String;
    let mut user_id: String;

    let user_info_req = IPCMessageReqWallet::GetUserInfo;
    let mut user_info_req_str = serde_json::to_string(&user_info_req).unwrap();
    user_info_req_str.push('\n');
    bin_wallet_stdin.write_all(user_info_req_str.as_bytes()).unwrap();
    
    let mut resp = String::new();
    bin_wallet_buf_reader.read_line(&mut resp).unwrap();
    let ipc_msg_resp : IPCMessageRespWallet = serde_json::from_str(&resp).unwrap();
    match ipc_msg_resp {
        IPCMessageRespWallet::UserInfo(username, uid) => {
            user_name = username;
            user_id = uid;
        }

        _ => {

        }
    }

    // Create the Terminal UI app
    let app_arc = Arc::new(Mutex::new(app::App::new(
        user_name.clone(), 
        user_id.clone(), 
        "".to_string(), 
        format!("SEND $100   // By {}", user_name))));


    // An enclosure func to generate signing requests when creating new transactions. 
    let create_sign_req = |sender: String, receiver: String, message: String| {
        let timestamped_message = format!("{}   // {}", message, SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
        let sign_req = IPCMessageReqWallet::SignRequest(serde_json::to_string(&(sender, receiver, timestamped_message)).unwrap());
        let mut sign_req_str = serde_json::to_string(&sign_req).unwrap();
        sign_req_str.push('\n');
        return sign_req_str;
    };


    if std::env::args().len() != 6 {
        // Then there must be 7 arguments provided. The last argument is the bot commands path
        // Please fill in the blank
        // Create a thread to read the bot commands from `bot_command_path`, execute those commands and update the UI
        // Notice that the `SleepMs(1000)` doesn't mean that the all threads in the whole process should sleep for 1000ms. It means that 
        // The next bot command that fakes the user interaction should be processed 1000ms later. 
        // It should not block the execution of any other threads or the main thread.
        let app_ui_ref_0 = app_arc.clone();
        let bot_config_path = std::env::args().nth(7).unwrap();
        let handle_bot = thread::spawn(move || {
            let file = File::open(bot_config_path).unwrap();
            let reader = BufReader::new(file);
            let mut read = String::new();

            for line in reader.lines() {
                read = line.unwrap();
                let bot_command : BotCommand = serde_json::from_str(&read).unwrap();
                match bot_command {
                    BotCommand::Send(receiver_user_id, transaction_message) => {
                        let sign_req_str = create_sign_req(user_id, receiver_user_id, transaction_message);
                        bin_wallet_stdin.write_all(sign_req_str.as_bytes()).unwrap();

                        let mut wallet_resp = String::new();
                        bin_wallet_buf_reader.read_line(&mut wallet_resp);
                        let ipc_wallet_msg_resp : IPCMessageRespWallet = serde_json::from_str(&wallet_resp);
                        let pub_tx_req: IPCMessageReqNakamoto = match ipc_wallet_msg_resp {
                            IPCMessageRespWallet::SignResponse(DataString, Signature) => {
                                IPCMessageReqNakamoto::PublishTx(DataString, Signature)
                            }

                            _ =>{

                            }
                        };
                        let mut pub_tx_req_str = serde_json::to_string(&pub_tx_req).unwrap();
                        pub_tx_req_str.push('\n');
                        bin_nakamoto_stdin.write_all(pub_tx_req_str.as_bytes()).unwrap();

                        let mut nakamoto_resp = String::new();
                        bin_nakamoto_buf_reader.read_line(&mut nakamoto_resp);
                        let ipc_nakamoto_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&nakamoto_resp);
                        let mut app_0 = app_ui_ref_0.lock().unwrap();
                        match ipc_nakamoto_msg_resp {
                            IPCMessageRespNakamoto::PublishTxDone => {
                                app_0.notify_log.push(format!("[Tx_pool] Add trans to the pool"));
                            }

                            _ => {

                            }
                        }
                    }

                    BotCommand::SleepMs(milliseconds) => {
                        thread::sleep(Duration::from_millis(milliseconds));
                    }
                }
                read.clear();
            }
        });
        handle_bot.join().unwrap();
    }


    // Please fill in the blank
    // - Spawn threads to read/write from/to bin_nakamoto/bin_wallet. (Through their piped stdin and stdout)
    // - You should request for status update from bin_nakamoto periodically (every 500ms at least) to update the App (UI struct) accordingly.
    // - You can also create threads to read from stderr of bin_nakamoto/bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.
    let app_ui_ref_1 = app_arc.clone();
    let handle_chain_status_update = thread::spawn(move || {
        loop {
            let chain_status_req = IPCMessageReqNakamoto::RequestChainStatus;
            let mut chain_status_req_str = serde_json::to_string(&chain_status_req).unwrap();
            chain_status_req_str.push('\n');
            bin_nakamoto_stdin.write_all(chain_status_req_str.as_bytes()).unwrap();

            let mut chain_status_resp = String::new();
            bin_nakamoto_buf_reader.read_line(&mut chain_status_resp).unwrap();
            let ipc_chain_status_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&chain_status_resp);
            let mut app_1 = app_ui_ref_1.lock().unwrap();
            match ipc_chain_status_msg_resp {
                IPCMessageRespNakamoto::ChainStatus(BTreeMap) => {
                    app_1.blocktree_status = BTreeMap.clone();
                }
        
                _ => {
        
                }
            }

            if app_1.should_quit {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }
    });

    let app_ui_ref_2 = app_arc.clone();
    let handle_net_status_update = thread::spawn(move || {
        loop {
            let net_status_req = IPCMessageReqNakamoto::RequestNetStatus;
            let mut net_status_req_str = serde_json::to_string(&net_status_req).unwrap();
            net_status_req_str.push('\n');
            bin_nakamoto_stdin.write_all(chain_status_req_str.as_bytes()).unwrap();

            let mut chain_status_resp = String::new();
            bin_nakamoto_buf_reader.read_line(&mut chain_status_resp).unwrap();
            let ipc_net_status_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&net_status_resp);
            let mut app_2 = app_ui_ref_2.lock().unwrap();
            match ipc_net_status_msg_resp {
                IPCMessageRespNakamoto::NetStatus(BTreeMap) => {
                    app_2.network_status = BTreeMap.clone();
                }
        
                _ => {
        
                }
            }

            if app_2.should_quit {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }
    });

    let app_ui_ref_3 = app_arc.clone();
    let handle_miner_status_update = thread::spawn(move || {
        loop {
            let miner_status_req = IPCMessageReqNakamoto::RequestMinerStatus;
            let mut miner_status_req_str = serde_json::to_string(&miner_status_req).unwrap();
            miner_status_req_str.push('\n');
            bin_nakamoto_stdin.write_all(miner_status_req_str.as_bytes()).unwrap();

            let mut miner_status_resp = String::new();
            bin_nakamoto_buf_reader.read_line(&mut miner_status_resp).unwrap();
            let ipc_miner_status_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&miner_status_resp);
            let mut app_3 = app_ui_ref_3.lock().unwrap();
            match ipc_miner_status_msg_resp {
                IPCMessageRespNakamoto::MinerStatus(BTreeMap) => {
                    app_3.miner_status = BTreeMap.clone();
                }
        
                _ => {
        
                }
            }

            if app_3.should_quit {
                break;
            }
            
            thread::sleep(Duration::from_millis(500));
        }
    });

    let app_ui_ref_4 = app_arc.clone();
    let handle_tx_pool_status_update = thread::spawn(move || {
        loop {
            let tx_pool_status_req = IPCMessageReqNakamoto::RequestTxPoolStatus;
            let mut tx_pool_status_req_str = serde_json::to_string(&tx_pool_status_req).unwrap();
            tx_pool_status_req_str.push('\n');
            bin_nakamoto_stdin.write_all(tx_pool_status_req_str.as_bytes()).unwrap();

            let mut tx_pool_status_resp = String::new();
            bin_nakamoto_buf_reader.read_line(&mut tx_pool_status_resp).unwrap();
            let ipc_tx_pool_status_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&tx_pool_status_resp);
            let mut app_4 = app_ui_ref_4.lock().unwrap();
            match ipc_tx_pool_status_msg_resp {
                IPCMessageRespNakamoto::TxPoolStatus(BTreeMap) => {
                    app_4.miner_status = BTreeMap.clone();
                }
        
                _ => {
        
                }
            }

            if app_4.should_quit {
                break;
            }
            
            thread::sleep(Duration::from_millis(500));
        }
    });

    
    let app_ui_ref_5 = app_arc.clone();
    let handle_balance_status_update = thread::spawn(move || {
        loop {
            let balance_status_req = IPCMessageReqNakamoto::GetAddressBalance(user_id);
            let mut balance_status_req_str = serde_json::to_string(&balance_status_req).unwrap();
            balance_status_req_str.push('\n');
            bin_nakamoto_stdin.write_all(balance_status_req_str.as_bytes()).unwrap();

            let mut balance_status_resp = String::new();
            bin_nakamoto_buf_reader.read_line(&mut balance_status_resp).unwrap();
            let ipc_balance_status_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&balance_status_resp);
            let mut app_5 = app_ui_ref_5.lock().unwrap();
            match ipc_balance_status_msg_resp {
                IPCMessageRespNakamoto::AddressBalance(uid, balance) => {
                    app_5.user_balance = balance;
                }
        
                _ => {
        
                }
            }

            if app_5.should_quit {
                break;
            }
            
            thread::sleep(Duration::from_millis(500));
        }
    });
    
    // UI thread. Modify it to suit your needs. 
    let app_ui_ref = app_arc.clone();
    //let bin_wallet_stdin_p_cloned = bin_wallet_stdin_p.clone();
    //let nakamoto_stdin_p_cloned = nakamoto_stdin_p.clone();
    let handle_ui = thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        if NO_UI_DEBUG_NODE {
            // If app_ui.should_quit is set to true, the UI thread will exit.
            loop {
                if app_ui_ref.lock().unwrap().should_quit {
                    break;
                }
                // sleep for 500ms
                thread::sleep(Duration::from_millis(500));
            }
            return;
        }
        let ui_loop = || -> Result<(), io::Error> {
            // setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let mut last_tick = Instant::now();
            let mut counter = 1;
            loop {
                terminal.draw(|f| {
                    app_ui_ref.lock().unwrap().draw(f)
                })?;

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_millis(100));
                
                if crossterm::event::poll(timeout)? {
                    let input = event::read()?.into();
                    let mut app = app_ui_ref.lock().unwrap();
                    match input {
                        Input { key: Key::Esc, .. } => {app.on_quit();}
                        Input { key: Key::Down, .. } => {app.on_down()}
                        Input { key: Key::Up, .. } => {app.on_up()},
                        Input { key: Key::Enter, .. } => {
                            if !app.are_inputs_valid {
                                app.client_log("Invalid inputs! Cannot create Tx.".to_string());
                            } else {
                                let (sender, receiver, message) = app.on_enter();
                                let sign_req_str = create_sign_req(sender, receiver, message);
                                //bin_wallet_stdin_p_cloned.lock().unwrap().write_all(sign_req_str.as_bytes()).unwrap();
                                bin_wallet_stdin.write_all(sign_req_str.as_bytes()).unwrap();
                                
                                let mut wallet_resp = String::new();
                                bin_wallet_buf_reader.read_line(&mut wallet_resp);
                                let ipc_wallet_msg_resp : IPCMessageRespWallet = serde_json::from_str(&wallet_resp);
                                let pub_tx_req: IPCMessageReqNakamoto = match ipc_wallet_msg_resp {
                                    IPCMessageRespWallet::SignResponse(DataString, Signature) => {
                                        IPCMessageReqNakamoto::PublishTx(DataString, Signature)
                                    }

                                    _ =>{

                                    }
                                };
                                let mut pub_tx_req_str = serde_json::to_string(&pub_tx_req).unwrap();
                                pub_tx_req_str.push('\n');
                                bin_nakamoto_stdin.write_all(pub_tx_req_str.as_bytes()).unwrap();

                                let mut nakamoto_resp = String::new();
                                bin_nakamoto_buf_reader.read_line(&mut nakamoto_resp);
                                let ipc_nakamoto_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&nakamoto_resp);
                                match ipc_nakamoto_msg_resp {
                                    IPCMessageRespNakamoto::PublishTxDone => {
                                        app.notify_log.push(format!("[Tx_pool] Add trans to the pool"));
                                    }

                                    _ => {

                                    }
                                }
                            }
                        }
                        // on control + s, request Nakamoto to serialize its state
                        Input { key: Key::Char('s'), ctrl: true, .. } => {
                            let serialize_req = IPCMessageReqNakamoto::RequestStateSerialization;
                            //let mut nakamoto_stdin = nakamoto_stdin_p_cloned.lock().unwrap();
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            //nakamoto_stdin.write_all(to_send.as_bytes()).unwrap();
                            bin_nakamoto_stdin.write_all(to_send.as_bytes()).unwrap();

                            let mut nakamoto_resp = String::new();
                            bin_nakamoto_buf_reader.read_line(&mut nakamoto_resp);
                            let ipc_nakamoto_msg_resp : IPCMessageRespNakamoto = serde_json::from_str(&nakamoto_resp);
                            match ipc_nakamoto_msg_resp {
                                IPCMessageRespNakamoto::StateSerialization(blocktree_json_string, tx_pool_json_string) => {
                                    let mut save_path = String::new();
                                    save_path.push_str("./tests/nakamoto_config/");
                                    save_path.push_str(&user_name);
                                    save_path.push_str(&(counter.to_string()));
                                    fs::create_dir(save_path);
                                    let mut block_tree_file = save_path.clone();
                                    block_tree_file.push_str("/BlockTree.json");
                                    let mut file = File::create(block_tree_file).unwrap();
                                    file.write_all(blocktree_json_string);

                                    let mut tx_pool_file = save_path.clone();
                                    tx_pool_file.push_str("/TxPool.json");
                                    file = File::create(tx_pool_file).unwrap();
                                    file.write_all(tx_pool_json_string);
                                }

                                _ => {

                                }
                            }
                        }
                        input => {
                            app.on_textarea_input(input);
                        }
                    }
                }

                let mut app = app_ui_ref.lock().unwrap();
                if last_tick.elapsed() >= tick_rate {
                    app.on_tick();
                    last_tick = Instant::now();
                }
                if app.should_quit {
                    break;
                }

                counter += 1;
            }
            // restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            Ok(())
        };
        ui_loop().unwrap();
    }); 
    handle_ui.join().unwrap();
    
    eprintln!("--- Sending \"Quit\" command...");
    //nakamoto_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();
    bin_nakamoto_stdin.write_all("\"Quit\"\n".as_bytes()).unwrap();
    //bin_wallet_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();
    bin_wallet_stdin.write_all("\"Quit\"\n".as_bytes()).unwrap();

    // Please fill in the blank
    // Wait for the IPC threads to finish
    handle_chain_status_update.join().unwrap();
    handle_balance_status_update.join().unwrap();
    handle_miner_status_update.join().unwrap();
    handle_net_status_update.join().unwrap();
    handle_tx_pool_status_update.join().unwrap();

    //let ecode1 = nakamoto_process.wait().expect("failed to wait on child nakamoto");
    let ecode1 = bin_nakamoto.wait().expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    //let ecode2 = bin_wallet_process.wait().expect("failed to wait on child bin_wallet");
    let ecode2 = bin_wallet.wait().expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);

}


