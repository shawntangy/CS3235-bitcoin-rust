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

    let nakamoto_init_req = IPCMessageReqNakamoto::Initialize(serde_json::to_string(&(blocktree_json, tx_pool_json, config_json)));
    let mut nakamoto_init_req_str = serde_json::to_string(&nakamoto_init_req).unwrap();
    nakamoto_init_req_str.push('\n');
    bin_nakamoto_stdin.write_all(nakamoto_init_req_str.as_bytes()).unwrap();

    // - Init request code for bin_wallet
    let wallet_config_path = std::env::args().nth(4).expect("Please specify wallet config path");
    let wallet_json = read_string_from_file(&wallet_config_path);
    let wallet_init_req = IPCMessageReqWallet::Initialize(serde_json::to_string(&wallet_json).unwrap());
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
        let bot_config_path = std::env::args().nth(7);

        
    }


    // Please fill in the blank
    // - Spawn threads to read/write from/to bin_nakamoto/bin_wallet. (Through their piped stdin and stdout)
    // - You should request for status update from bin_nakamoto periodically (every 500ms at least) to update the App (UI struct) accordingly.
    // - You can also create threads to read from stderr of bin_nakamoto/bin_wallet and add those lines to the UI (app.stderr_log) for easier debugging.


    // UI thread. Modify it to suit your needs. 
    let app_ui_ref = app_arc.clone();
    let bin_wallet_stdin_p_cloned = bin_wallet_stdin_p.clone();
    let nakamoto_stdin_p_cloned = nakamoto_stdin_p.clone();
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
                                bin_wallet_stdin_p_cloned.lock().unwrap().write_all(sign_req_str.as_bytes()).unwrap();
                            }
                        }
                        // on control + s, request Nakamoto to serialize its state
                        Input { key: Key::Char('s'), ctrl: true, .. } => {
                            let serialize_req = IPCMessageReqNakamoto::RequestStateSerialization;
                            let mut nakamoto_stdin = nakamoto_stdin_p_cloned.lock().unwrap();
                            let mut to_send = serde_json::to_string(&serialize_req).unwrap();
                            to_send.push_str("\n");
                            nakamoto_stdin.write_all(to_send.as_bytes()).unwrap();
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
    

    let ecode1 = nakamoto_process.wait().expect("failed to wait on child nakamoto");
    eprintln!("--- nakamoto ecode: {}", ecode1);

    let ecode2 = bin_wallet_process.wait().expect("failed to wait on child bin_wallet");
    eprintln!("--- bin_wallet ecode: {}", ecode2);

}


