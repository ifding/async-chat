//! Asynchronous chat server.

use std::sync::Arc;
use async_std::prelude::*;
use async_chat::utils::ChatResult;

use crate::connection::serve;

mod connection;
mod group;
mod group_table;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: server ADDRESS");
    // Represent the server's current list of chat groups, shared by all connections.
    let chat_group_table = Arc::new(group_table::GroupTable::new());
    
    async_std::task::block_on(async {
        use async_std::{net, task};
        // To handle incoming connections from clients, it creates a `TcpListener` sockets,
        // whose `incoming` method returns a stream of `std::io::Result<TcpStream>` values.
        let listener = net::TcpListener::bind(address).await?;

        let mut new_connections = listener.incoming();
        while let Some(socket_result) = new_connections.next().await {
            let socket = socket_result?;
            let groups = chat_group_table.clone();
            task::spawn(async {
                log_error(serve(socket, groups).await);
            });
        }
        Ok(())
    })
}

/// If `connection::serve` returns an error, log it to stderr and let the task exit.
/// Other connections continue to run as usual.
fn log_error(result: ChatResult<()>) {
    if let Err(error) = result {
        eprintln!("Error: {}", error);
    }
}