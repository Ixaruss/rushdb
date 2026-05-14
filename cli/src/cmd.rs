use crate::{client::query, libc::ReqType, shell};
use clap::{Parser, Subcommand};
use tokio::net::TcpStream;

#[derive(Parser)]
#[command(name = "bat", about = "A CLI tool to talk to mem store", version)]
pub struct Cmd {
    #[command(subcommand)]
    commands: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
    Exists { key: String },
    Total,
}

impl Cmd {
    pub async fn init() {
        let cli = Cmd::parse();
        Cmd::handle_command_internal(cli.commands, None).await;
    }
    pub async fn handle_command(command_str: &str, stream: Option<&mut TcpStream>) {
        let args = command_str.split_whitespace().collect::<Vec<_>>();

        let mut full_args = vec!["mem-store"];
        full_args.extend(args);

        match Cmd::try_parse_from(full_args) {
            Ok(cmd) => {
                Cmd::handle_command_internal(cmd.commands, stream).await;
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    async fn handle_command_internal(commands: Option<Commands>, stream: Option<&mut TcpStream>) {
        let mut owned;
        let mut strm: &mut TcpStream = match stream {
            Some(s) => s,
            None => {
                owned = TcpStream::connect("0.0.0.0:6080")
                    .await
                    .expect("Failed to connect to server");
                &mut owned
            }
        };
        match commands {
            Some(Commands::Get { key }) => {
                let res = query(Some(key), None, ReqType::GET, &mut strm).await;
                match res {
                    Some(val) => println!("{}", val),
                    None => println!("DONT EXISTS"),
                }
            }
            Some(Commands::Set { key, value }) => {
                let res = query(Some(key), Some(value), ReqType::SET, &mut strm).await;
                match res {
                    Some(val) => println!("{}", val),
                    None => println!("DONE"),
                }
            }
            Some(Commands::Del { key }) => {
                let res = query(Some(key), None, ReqType::DEL, &mut strm).await;
                match res {
                    Some(val) => println!("{}", val),
                    None => println!("DONE"),
                }
            }
            Some(Commands::Exists { key }) => {
                let res = query(Some(key), None, ReqType::EXISTS, &mut strm).await;
                match res {
                    Some(val) => println!("{}", val),
                    None => (),
                }
            }
            Some(Commands::Total) => {
                let res = query(None, None, ReqType::TOTAL, &mut strm).await;
                match res {
                    Some(val) => println!("{}", val),
                    None => (),
                }
            }
            None => {
                shell::start_interactive_shell().await;
            }
        }
    }
}
