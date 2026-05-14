use crate::cmd::Cmd;
use crate::util::ShellHelper;
use rustyline::error::ReadlineError;
use tokio::net::TcpStream;

pub async fn start_interactive_shell() {
    let mut stream = TcpStream::connect("0.0.0.0:6080")
        .await
        .expect("Failed to connect to server");
    let mut rl = rustyline::Editor::<ShellHelper, rustyline::history::DefaultHistory>::with_config(
        rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build(),
    )
    .expect("Failed to initialize shell");

    rl.set_helper(Some(ShellHelper));

    // Load history from a file (optional)
    // if rl.load_history("history.txt").is_err() {
    //     println!("No previous history found.");
    // }

    println!("Type 'help' for commands or 'exit' to quit.");
    println!("version: 1.1");
    loop {
        let readline = rl.readline("cli> ");

        match readline {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }
                if input == "exit" {
                    break;
                }
                let _ = rl.add_history_entry(input);
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(Cmd::handle_command(input, Some(&mut stream)));
                });
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history back to the file on exit
    // let _ = rl.save_history("history.txt");
}
