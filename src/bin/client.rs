use async_std::prelude::*;
use async_std::{io, net};
use async_chat::utils::{self, ChatResult};

/// The asynchronous `BufReader`'s `lines` method cannot return an iterator,
/// so calling `command_lines.next()` will block the thread until the next 
/// line is ready. Instead, `lines` returns a _stream_ of `Result<String>` 
/// values. It produces a sequence of values on demand, in an async-friendly
/// way.
async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    println!("Commands:\n\
              join GROUP\n\
              post GROUP MESSAGE...\n\
              Type Control-D (on Unix) or Control-Z (on Windows) \
              to close the connection.");
    let mut command_lines = io::BufReader::new(io::stdin()).lines();
    while let Some(command_line) = command_lines.next().await {
        let command = command_line?;
        let request = match parse_command(&command) {
            Some(request) => request,
            None => continue,
        };
        utils::send_as_json(&mut to_server, &request).await?;
        to_server.flush().await?;
    }
    Ok(())
}

use async_chat::FromClient;
use std::sync::Arc;

/// Parse a line (presumably read from the standard input) as a `Request`.
fn parse_command(line: &str) -> Option<FromClient> {
    let (command, rest) = get_next_token(line)?;
    if command == "post" {
        let (group, rest) = get_next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(FromClient::Post {
            group_name: Arc::new(group.to_string()), 
            message: Arc::new(message),
        });
    } else if command == "join" {
        let (group, rest) = get_next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(FromClient::Join {
            group_name: Arc::new(group.to_string()),
        });
    } else {
        eprintln!("Unrecognized command: {:?}", line);
        return None; 
    }

}

/// Given a string `input`, return `Some((token, rest))`, where `token` is the
/// first run of non-whitespace characters in `input`, and `rest` is the rest of
/// the string. If the string contains no non-whitespace characters, return
/// `None`.
fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();
    if input.is_empty() {
        return None;
    }
    
    match input.find(char::is_whitespace) {
        Some(i) => Some((&input[..i], &input[i..])),
        None => Some((input, "")),
    }
}

use async_chat::FromServer;

/// Receives a stream of `FromServer` values from the network and prints them out.
async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    // Take a socket receiving data from the server and wraps a `BufReader` around it.
    let buffered = io::BufReader::new(from_server);
    let mut reply_stream = utils::receive_as_json(buffered);

    while let Some(reply) = reply_stream.next().await {
        let reply = reply?;
        match reply {
            FromServer::Error(error) => {
                eprintln!("error from server: {}", error);
            }
            FromServer::Message {group_name, message } => {
                println!("message posted to {}: {}", group_name, message);
            }
        }
    }
    Ok(())
}

use async_std::task;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1)
        .expect("Usage: client ADDRESS:PORT");
    // wrap functions in an async block and pass the block's future to `task::block_on` to run.
    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;

        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);

        // Return a new future that polls both `to_server` and `from_server` and returns
        // `Poll::Ready(v)` as soon as either of them is ready. Both futures must have the 
        // same output type: the final value that of whichever future finished first. The 
        // uncompleted future is dropped.
        from_server.race(to_server).await?;

        Ok(())
    })
}