use std::error::Error;

use serde::{Serialize, de::DeserializeOwned};
use async_std::prelude::*;
use std::marker::Unpin;

/// `aysnc_std`, `serde` and `tokio` crates each define their own error types.
/// But `?` operator can automatically convert them all into `ChatError` type.
/// The `Send` and `Sync` bounds ensure that if a task spawned onto another 
/// task fails, it can safely report the error to the main thread.
pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;

/// This function builds the JSON representation of packet as a String,
/// adds a newline at the end, and then writes it all to outbound stream.
/// This function is flexible, it does not depend on the details of the stream
/// or packet type. 
/// The `Unpin` constraint on `S` is required to use the `write_all` method.
pub async fn send_as_json<S, P>(outbound: &mut S, packet: &P) -> ChatResult<()> 
where
    S: async_std::io::Write + Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(packet)?;
    json.push('\n');
    outbound.write_all(json.as_bytes()).await?;
    Ok(())
}

/// Like `send_as_json`, this function is generic in the input stream and packet type.
/// `BufRead` represents a buffed input byte stream.
/// A type that implements `DeserializeOwned` is always independent of the buffer
/// it was deserialized from.
/// The function return type indicates that we return _some_ type that produces a 
/// sequence of `ChatResult<P>` values asynchronously.
/// This function is not an async function, it is an ordinary function that returns
/// an async value, a stream.
pub fn receive_as_json<S, P>(inbound: S) -> impl Stream<Item = ChatResult<P>>
where
    S: async_std::io::BufRead + Unpin,
    P: DeserializeOwned,
{
    // `.line()` gives us a Stream of `std::io::Result<String>` values
    inbound.lines().map(|line_result| -> ChatResult<P> {
        let line = line_result?;  // handle errors
        // parse each line as the JSON form of a value of type `P`
        let parsed = serde_json::from_str::<P>(&line)?;
        Ok(parsed)
    })
}