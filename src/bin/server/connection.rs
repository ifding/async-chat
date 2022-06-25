/// Handle a single client's connection.
 
use async_chat::{FromClient, FromServer};
use async_chat::utils::{self, ChatResult};
use async_std::prelude::*;
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::sync::Arc;

use crate::group_table::GroupTable;

/// The loop handles an incoming stream of `FromClient` values, built from a buffered TCP stream
/// with `receive_as_json`. If an error occurs, we generate a `FromServer::Error` packet to convey
/// the error to the client. 
pub async fn serve(socket: TcpStream, groups: Arc<GroupTable>) 
    -> ChatResult<()> 
{
    // Clients would like to receive message from the chat group they're joined. Wraps each 
    // `Outbound` in an `Arc` reference-counted pointer so that all the groups the client 
    // joins can point to the same shared `Outbound` instance.
    // Give everyone a clone of the TcpStream.
    let outbound = Arc::new(Outbound::new(socket.clone()));

    let buffered = BufReader::new(socket);
    let mut from_client = utils::receive_as_json(buffered);
    while let Some(request_result) = from_client.next().await {
        let request = request_result?;
        let result = match request {
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join(outbound.clone());
                Ok(())
            }
            FromClient::Post { group_name, message } => {
                match groups.get(&group_name) {
                    Some(group) => {
                        group.post(message);
                        Ok(())
                    }
                    None =>  {
                        Err(format!("Group '{}' does not exist", group_name))
                    }
                }
            }
        };
        if let Err(error) = result {
            let report = FromServer::Error(error);
            outbound.send(report).await?;
        }
    }
    Ok(())
}

use async_std::sync::Mutex;

pub struct Outbound(Mutex<TcpStream>);

/// Handle multipe sources trying to write a packet to the socket at the same time.
impl Outbound {
    // When created, an `Outbound` value takes ownership of a `TcpStream` and wraps 
    // in a `Mutex` to ensure that only one task can use it at a time.
    pub fn new(to_client: TcpStream) -> Self {
        Outbound(Mutex::new(to_client))
    }

    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        // First locks the mutex, returning a guard value that derederences to `TcpStream` value.
        let mut guard = self.0.lock().await;
        // Use `write_as_json` to transmit `packet`.
        // `&mut *guard`: Rust doesn't apply deref coercions to meet trait bounds. Instead, we
        // explicitly dereference the mutex guard and then borrow a mutable reference to the
        // `TcpStream` it protects, producing a `&mut TcpStream` that `send_as_json` requires.
        utils::send_as_json(&mut *guard, &packet).await?;
        // `guard.flush()` to ensure it won't languish half-transmitted in some buffer somewhere.
        guard.flush().await?;
        Ok(())
    }
}