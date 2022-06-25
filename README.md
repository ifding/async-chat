
# Asynchronous chat client and server

> Chapter 20 of the book _Programming Rust (2nd)_, by Blandy, Orendorff, and Tindal

The entire stucture:

```
.
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
└── src
    ├── bin
    │   ├── client.rs
    │   └── server
    │       ├── connection.rs
    │       ├── group.rs
    │       ├── group_table.rs
    │       └── main.rs
    ├── lib.rs
    └── utils.rs
```

To run the server, type:
```
    $ cargo run --release --bin server -- localhost:8088
```
To run the client, type:
```
    $ cargo run --release --bin client -- localhost:8088
```
The client supports only two commands:

- `join group` - Join the group named `group`. If
    that group does not exist, it is created. The name of the group must not
    contain any spaces.

-  `post group message` - Post `message` to the chat group named `group`. The group name
    must not contain any spaces, but the message can.

There is no command to leave a group. There is no concept of a user name. To
exit the client, hit ctrl-D on Linux or macOS, or ctrl-Z on Windows.

An example client session:
```
    $ cargo run --release --bin client -- localhost:8088
        Finished release [optimized] target(s) in 0.04s
         Running `/home/chapters/asynchronous/target/release/client 'localhost:8088'`
    Commands:
    join GROUP
    post GROUP MESSAGE...
    Type Control-D (on Unix) or Control-Z (on Windows) to close the connection.
    join Rust
    post Rust I love Rust!
    message posted to Rust: I love Rust!
    message posted to Rust: LOL, I do too!
    message posted to Rust: Hello, rust lovers!
    post Rust Hi!
    message posted to Rust: Hi!
    ctrl-D
    $
```