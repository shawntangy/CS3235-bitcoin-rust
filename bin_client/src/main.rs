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

use core::panic;
use std::fs::{File, read, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufRead};
use std::process::{Command, Stdio};
use std::collections::{BTreeMap, HashMap, btree_map};
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
    let nakamoto_seccomp_path = std::env::args().nth(3).expect("Please specify nakamoto seccomp path");
    let mut bin_nakamoto_process = Command::new("./target/debug/bin_nakamoto").arg(nakamoto_seccomp_path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    // - Create bin_wallet process:  Command::new("./target/debug/bin_wallet")...
    let wallet_seccomp_path = std::env::args().nth(5).expect("Please specify wallet seccomp path");
    let mut bin_wallet_process = Command::new("./target/debug/bin_wallet").arg(wallet_seccomp_path).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    
    // - Get stdin and stdout of those processes
    // - Get nakamoto stdin and stdout
    let nakamoto_stdin = bin_nakamoto_process.stdin.take().unwrap();
    let nakamoto_stdout = bin_nakamoto_process.stdout.take().unwrap();
    let nakamoto_stderr = bin_nakamoto_process.stderr.take().unwrap();

    let nakamoto_stdin_p = Arc::new(Mutex::new(nakamoto_stdin));

    // - Get wallet stdin and stdout
    let bin_wallet_stdin = bin_wallet_process.stdin.take().unwrap();
    let bin_wallet_stdout = bin_wallet_process.stdout.take().unwrap();

    let bin_wallet_stdin_p = Arc::new(Mutex::new(bin_wallet_stdin));

    // - Create buffer readers if necessary
    let nakamoto_stdout_p = Arc::new(Mutex::new(BufReader::new(nakamoto_stdout)));
    let nakamoto_stderr_p = Arc::new(Mutex::new(BufReader::new(nakamoto_stderr)));
    let bin_wallet_stdout_p = Arc::new(Mutex::new(BufReader::new(bin_wallet_stdout)));

    // - Send initialization requests to bin_nakamoto and bin_wallet
    let nakamoto_stdin_p_cloned_a = nakamoto_stdin_p.clone();
    let bin_wallet_stdin_p_cloned_a = bin_wallet_stdin_p.clone();
    let bin_wallet_stdout_p_cloned_a = bin_wallet_stdout_p.clone();
    // - Init request code for bin_nakamoto
    let nakamoto_config_path = std::env::args().nth(2).expect("PLease specify nakamoto config path");
    
    let mut blocktree_path = nakamoto_config_path.clone();
    blocktree_path.push_str("/BlockTree.json");
    let blocktree_json = read_string_from_file(&blocktree_path);
    
    let mut tx_pool_path = nakamoto_config_path.clone();
    tx_pool_path.push_str("/TxPool.json");
    let tx_pool_json = read_string_from_file(&tx_pool_path);

    let mut config_path = nakamoto_config_path.clone();
    config_path.push_str("/Config.json");
    let config_json = read_string_from_file(&config_path);

    let nakamoto_init_req = IPCMessageReqNakamoto::Initialize(blocktree_json, tx_pool_json, config_json);
    let mut nakamoto_init_req_str = serde_json::to_string(&nakamoto_init_req).unwrap();
    nakamoto_init_req_str.push('\n');
    nakamoto_stdin_p_cloned_a.lock().unwrap().write_all(nakamoto_init_req_str.as_bytes()).unwrap();

    // - Init request code for bin_wallet
    let wallet_config_path = std::env::args().nth(4).expect("Please specify wallet config path");
    let wallet_json = read_string_from_file(&wallet_config_path);
    
    let wallet_init_req = IPCMessageReqWallet::Initialize(wallet_json);
    let mut wallet_init_req_str = serde_json::to_string(&wallet_init_req).unwrap();
    wallet_init_req_str.push('\n');
    bin_wallet_stdin_p_cloned_a.lock().unwrap().write_all(wallet_init_req_str.as_bytes()).unwrap();

    let mut wallet_init_resp = String::new();
    bin_wallet_stdout_p_cloned_a.lock().unwrap().read_line(&mut wallet_init_resp).unwrap();

    let client_seccomp_path = std::env::args().nth(1).expect("Please specify client seccomp path");
    // Please fill in the blank
    // sandboxing the bin_client (For part B). Leave it blank for part A.
    let policy_path = client_seccomp_path; 
    // If the first param is provided, read the seccomp config and apply it
    let policy = read_string_from_file(&policy_path);
    let filter_map: BpfMap = seccompiler::compile_from_json(policy.as_bytes(), std::env::consts::ARCH.try_into().unwrap()).unwrap();
    let filter = filter_map.get("main_thread").unwrap();
    seccompiler::apply_filter(&filter).unwrap();
    

    // Please fill in the blank
    // Read the user info from wallet
    let mut user_name = String::new();
    let mut user_id: String = String::new();

    let user_info_req = IPCMessageReqWallet::GetUserInfo;
    let mut user_info_req_str = serde_json::to_string(&user_info_req).unwrap();
    user_info_req_str.push('\n');
    bin_wallet_stdin_p_cloned_a.lock().unwrap().write_all(user_info_req_str.as_bytes()).unwrap();
    
    let mut wallet_resp = String::new();
    bin_wallet_stdout_p_cloned_a.lock().unwrap().read_line(&mut wallet_resp).unwrap();
    let ipc_wallet_resp : IPCMessageRespWallet = serde_json::from_str(&wallet_resp).unwrap();
    match ipc_wallet_resp {
        IPCMessageRespWallet::UserInfo(username, uid) => {
            user_name.push_str(&username);
            user_id.push_str(&uid);
        }

        _ => panic!(),
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
        let app_ui_ref_a = app_arc.clone();
        let user_id_a = user_id.clone();
        let bin_wallet_stdin_p_cloned_b = bin_wallet_stdin_p.clone();
        let nakamoto_stdin_p_cloned_b = nakamoto_stdin_p.clone();
        let bin_wallet_stdout_p_cloned_b = bin_wallet_stdout_p.clone();
        let nakamoto_stdout_p_cloned_b = nakamoto_stdout_p.clone();

        let bot_config_path = std::env::args().nth(6).unwrap();

        thread::spawn(move || {
            let file = File::open(bot_config_path).unwrap();
            let mut reader = BufReader::new(file);

            loop {
                if app_ui_ref_a.lock().unwrap().should_quit {
                    break;
                }

                let mut read = String::new();
                reader.read_line(&mut read).unwrap();
                if read.len() == 2 {
                    continue;
                }

                if read.len() !=0 {
                    let bot_command : BotCommand = serde_json::from_str(&read).unwrap();
                    match bot_command {
                        BotCommand::Send(receiver_user_id, transaction_message) => {
                            let sign_req_str = create_sign_req(user_id_a.clone(), receiver_user_id, transaction_message);
                            bin_wallet_stdin_p_cloned_b.lock().unwrap().write_all(sign_req_str.as_bytes()).unwrap();
                        }
    
                        BotCommand::SleepMs(milliseconds) => {
                            thread::sleep(Duration::from_millis(milliseconds));
                        }
                    }                    
                }
            }
        });
    }

    // Please fill in the blank
    // - Spawn threads to read/write from/to bin_nakamoto/bin_wallet. (Through their piped stdin and stdout)
    // - You should request for status update from bin_nakamoto periodically (every 500ms at least) to update the App (UI struct) accordingly.
    // - You can also create threads to read from stderr of bin_nakamoto/bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.
    let app_ui_ref_b = app_arc.clone();
    let user_id_b = user_id.clone();
    let nakamoto_stdin_p_cloned_c = nakamoto_stdin_p.clone();
    let handle_nakamoto_req_update = thread::spawn(move || {
        loop {
            if app_ui_ref_b.lock().unwrap().should_quit {
                break;
            }

            // - Request Chain Status Update
            let chain_status_req = IPCMessageReqNakamoto::RequestChainStatus;
            let mut chain_status_req_str = serde_json::to_string(&chain_status_req).unwrap();
            chain_status_req_str.push('\n');
            nakamoto_stdin_p_cloned_c.lock().unwrap().write_all(&chain_status_req_str.as_bytes()).unwrap();

            // - Request Net Status Update
            let net_status_req = IPCMessageReqNakamoto::RequestNetStatus;
            let mut net_status_req_str = serde_json::to_string(&net_status_req).unwrap();
            net_status_req_str.push('\n');
            nakamoto_stdin_p_cloned_c.lock().unwrap().write_all(&net_status_req_str.as_bytes()).unwrap();

            // - Request Miner Status Update
            let miner_status_req = IPCMessageReqNakamoto::RequestMinerStatus;
            let mut miner_status_req_str = serde_json::to_string(&miner_status_req).unwrap();
            miner_status_req_str.push('\n');
            nakamoto_stdin_p_cloned_c.lock().unwrap().write_all(&miner_status_req_str.as_bytes()).unwrap();
            
            // - Request Tx Pool Status Update
            let tx_pool_status_req = IPCMessageReqNakamoto::RequestTxPoolStatus;
            let mut tx_pool_status_req_str = serde_json::to_string(&tx_pool_status_req).unwrap();
            tx_pool_status_req_str.push('\n');
            nakamoto_stdin_p_cloned_c.lock().unwrap().write_all(&tx_pool_status_req_str.as_bytes()).unwrap();
            
            // - Request Address Balance Status Update
            let balance_status_req = IPCMessageReqNakamoto::GetAddressBalance(user_id_b.clone());
            let mut balance_status_req_str = serde_json::to_string(&balance_status_req).unwrap();
            balance_status_req_str.push('\n');
            nakamoto_stdin_p_cloned_c.lock().unwrap().write_all(&balance_status_req_str.as_bytes()).unwrap();

            thread::sleep(Duration::from_millis(200));
        }
    });

    let app_ui_ref_c = app_arc.clone();
    let nakamoto_stdout_p_cloned_c = nakamoto_stdout_p.clone();
    let nakamoto_config_path_cloned = nakamoto_config_path.clone();
    //let mut counter = 1;
    let handle_nakamoto_resp = thread::spawn(move || {
        loop {
            let mut nakamoto_resp = String::new();
            nakamoto_stdout_p_cloned_c.lock().unwrap().read_line(&mut nakamoto_resp).unwrap();
            // eprintln!("nakamoto_resp: {}", nakamoto_resp);
            let ipc_nakamoto_resp : IPCMessageRespNakamoto = serde_json::from_str(&nakamoto_resp).unwrap();
            let mut app_c = app_ui_ref_c.lock().unwrap();
            match ipc_nakamoto_resp {
                IPCMessageRespNakamoto::Initialized => {
                    app_c.notify_log.push(format!("[Main] Nakamoto Initialized"));
                }

                IPCMessageRespNakamoto::PublishTxDone => {
                    app_c.notify_log.push(format!("[Tx_pool] Add trans to the pool"));
                }

                IPCMessageRespNakamoto::AddressBalance(user_id, balance) => {
                    app_c.user_balance = balance;
                }

                IPCMessageRespNakamoto::BlockData(block_data) => {}

                IPCMessageRespNakamoto::NetStatus(net_btree_map) => {
                    app_c.network_status = net_btree_map.clone();
                }

                IPCMessageRespNakamoto::ChainStatus(chain_btree_map) => {
                    app_c.blocktree_status = chain_btree_map.clone();
                }

                IPCMessageRespNakamoto::MinerStatus(miner_btree_map) => {
                    app_c.miner_status = miner_btree_map.clone();
                }

                IPCMessageRespNakamoto::TxPoolStatus(txpool_btree_map) => {
                    app_c.txpool_status = txpool_btree_map.clone();
                }

                IPCMessageRespNakamoto::StateSerialization(blocktree_json_string, tx_pool_json_string) => {
                    //let mut save_path = String::new();
                    let mut save_path = nakamoto_config_path_cloned.clone();
                    //save_path.push_str("tests/nakamoto_config_");
                    //save_path.push_str(&user_name);
                    //save_path.push_str(&(counter.to_string()));
                    save_path.push_str("/");
                    save_path.push_str(&(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string()));                    

                    let mut block_tree_file = save_path.clone();
                    let mut tx_pool_file = save_path.clone();

                    //fs::create_dir_all(save_path).unwrap();
                    
                    block_tree_file.push_str("-BlockTree.json");
                    let mut file = OpenOptions::new().create_new(true).write(true).open(block_tree_file).unwrap();
                    file.write_all(blocktree_json_string.as_bytes()).unwrap();

                    tx_pool_file.push_str("-TxPool.json");
                    file = OpenOptions::new().create_new(true).write(true).open(tx_pool_file).unwrap();
                    file.write_all(tx_pool_json_string.as_bytes()).unwrap();

                    //counter += 1;
                }

                IPCMessageRespNakamoto::Quitting => {
                    break;
                }

                IPCMessageRespNakamoto::Notify(msg) => {
                    app_c.notify_log.push(msg);
                }
            }

            //thread::sleep(Duration::from_millis(50));
        }
    });

    let bin_wallet_stdout_p_cloned_c = bin_wallet_stdout_p.clone();
    let nakamoto_stdin_p_cloned_d = nakamoto_stdin_p.clone();
    let handle_wallet_resp = thread::spawn(move || {
        loop {
            let mut wallet_resp = String::new();
            bin_wallet_stdout_p_cloned_c.lock().unwrap().read_line(&mut wallet_resp).unwrap();
            let ipc_wallet_resp : IPCMessageRespWallet = serde_json::from_str(&wallet_resp).unwrap();
            match ipc_wallet_resp {
                IPCMessageRespWallet::Initialized => {}

                IPCMessageRespWallet::Quitting => {
                    break;
                }

                IPCMessageRespWallet::SignResponse(data_string, signature) => {
                    let pub_tx_req = IPCMessageReqNakamoto::PublishTx(data_string, signature);
                    let mut pub_tx_req_str = serde_json::to_string(&pub_tx_req).unwrap();
                    // eprintln!("req str: {}",pub_tx_req_str);
                    pub_tx_req_str.push('\n');
                    nakamoto_stdin_p_cloned_d.lock().unwrap().write_all(pub_tx_req_str.as_bytes()).unwrap();
                }

                IPCMessageRespWallet::VerifyResponse(isSuccess, data_string) => {}

                IPCMessageRespWallet::UserInfo(username, uid) => {}
            }

            //thread::sleep(Duration::from_millis(50));
        }

    });

    let app_ui_ref_e = app_arc.clone();
    let nakamoto_stderr_p_cloned_a = nakamoto_stderr_p.clone();
    let handle_nakamoto_debug_resp = thread::spawn(move || {
        loop {
            if app_ui_ref_e.lock().unwrap().should_quit {
                break;
            }

            let mut nakamoto_debug_resp = String::new();
            nakamoto_stderr_p_cloned_a.lock().unwrap().read_line(&mut nakamoto_debug_resp);
            app_ui_ref_e.lock().unwrap().stderr_log.push(nakamoto_debug_resp);
        }
    });

    // UI thread. Modify it to suit your needs. 
    let app_ui_ref_d = app_arc.clone();
    let bin_wallet_stdin_p_cloned_c = bin_wallet_stdin_p.clone();
    let nakamoto_stdin_p_cloned_e = nakamoto_stdin_p.clone();
    let handle_ui = thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        if NO_UI_DEBUG_NODE {
            // If app_ui.should_quit is set to true, the UI thread will exit.
            loop {
                if app_ui_ref_d.lock().unwrap().should_quit {
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
            loop {
                terminal.draw(|f| {
                    app_ui_ref_d.lock().unwrap().draw(f)
                })?;

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_millis(100));
                
                if crossterm::event::poll(timeout)? {
                    let input = event::read()?.into();
                    let mut app = app_ui_ref_d.lock().unwrap();
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
                                bin_wallet_stdin_p_cloned_c.lock().unwrap().write_all(sign_req_str.as_bytes()).unwrap();
                            }
                        }
                        // on control + s, request Nakamoto to serialize its state
                        Input { key: Key::Char('s'), ctrl: true, .. } => {
                            let serialize_req = IPCMessageReqNakamoto::RequestStateSerialization;
                            let mut nakamoto_stdin = nakamoto_stdin_p_cloned_e.lock().unwrap();
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            nakamoto_stdin.write_all(to_send.as_bytes()).unwrap();
                        }
                        input => {
                            app.on_textarea_input(input);
                        }
                    }
                }

                let mut app = app_ui_ref_d.lock().unwrap();
                if last_tick.elapsed() >= tick_rate {
                    app.on_tick();
                    last_tick = Instant::now();
                }
                if app.should_quit {
                    break;
                }
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
    nakamoto_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();
    bin_wallet_stdin_p.lock().unwrap().write_all("\"Quit\"\n".as_bytes()).unwrap();

    // Please fill in the blank
    // Wait for the IPC threads to finish
    handle_nakamoto_req_update.join().unwrap();
    handle_nakamoto_resp.join().unwrap();
    handle_wallet_resp.join().unwrap();
    handle_nakamoto_debug_resp.join().unwrap();

    //let ecode1 = nakamoto_process.wait().expect("failed to wait on child nakamoto");
    let ecode1 = bin_nakamoto_process.wait().expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    //let ecode2 = bin_wallet_process.wait().expect("failed to wait on child bin_wallet");
    let ecode2 = bin_wallet_process.wait().expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);

}
