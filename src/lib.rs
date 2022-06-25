use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod utils;

/// Using a referene-counted Arc<String> instead of a plain String helps
/// the server avoid making copies of strings as it manages groups and 
/// distributes messages.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FromClient {
    Join { group_name: Arc<String> },
    Post { 
        group_name: Arc<String>, 
        message: Arc<String> 
    },
}

/// `Serialize` and `Deserialize` let us call `serde_json::to_string` to convert
/// them into JSON values, send them across the network, and `serde_json::from_str` 
/// to convert them back into their Rust forms.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum FromServer {
    Message { 
        group_name: Arc<String>, 
        message: Arc<String> 
    },
    Error(String),
}

#[test]
fn test_fromclient_json() {
    use std::sync::Arc;

    let from_client = FromClient::Post { 
        group_name: Arc::new("Dogs".to_string()), 
        message: Arc::new("Samoyeds rock!".to_string()), 
    };

    let json = serde_json::to_string(&from_client).unwrap();
    assert_eq!(json, 
        r#"{"Post":{"group_name":"Dogs","message":"Samoyeds rock!"}}"#);

    assert_eq!(serde_json::from_str::<FromClient>(&json).unwrap(),
        from_client
    );
}