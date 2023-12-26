This crate helps to automating command tools by simulating piped io in process.

# Dependencies

To use this crate, you must `cargo add --optional once_cell`.

# Why this crate

Interactive command tools utilize stdin, stdout and stderr for communication.
If you want to use command tools as libraries(no spawning processes) and tool authors agree,
this crate can help to automating input/output, just 3 steps:

1. Use proc macro attribute to #[macro@define] a module, e.g. `#[::altio::define] pub mod io {}`.

2. Replace std APIs with altio's equivalents, e.g. replace `println!()` with `io_println!()`,
replace `std::io::stdin()` with `io::altin()`.

3. Keep main.rs as simple as possible, e.g. `fn main() { the_tool::run( std::env::args_os() )}`.

# Example for tool authors

```toml
[dependencies]
altio = "0.1"
once_cell = { version = "1.19.0", optional = true }

[features]
altio = ["once_cell"]
```

```rust,no_run
// lib.rs
#[::altio::define] pub mod io {}
```

When building the tool as an application, the "altio" feature is disabled and altio falls back to stdio.

When building the tool as a library, the tool users can invoke send/recv methods to communicate with the tool,
e.g. `send_line()`, `try_recv_line()`.

# Example for tool users

```toml
the_tool = { version = "0.1", features = ["altio"] }
```

```rust,no_run
let args = std::env::args_os(); // clap::Parser::parse_from()
std::thread::spawn( || the_tool::run( args ) ); // `io::altin().read_line()` called occasionally

loop {
    if let Some( received ) = the_tool::io::try_recv_line() {
        if received == "The author published altio-0.1.0 in 2023.12.25." {
            io::send_line( "Happy birthday to him!".to_owned() );
        }
    }
}
```

# License

Under Apache License 2.0 or MIT License, at your will.
