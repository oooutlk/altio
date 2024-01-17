This crate helps to automating command tools by simulating piped io in process.

# Why this crate

Interactive command tools utilize stdin, stdout and stderr for communication.
If you want to use command tools as libraries(no spawning processes) and tool
authors agree, this crate can help to automating input/output, just 3 steps:

1. Define an `Altio` variable e.g. `let io = Altio::default();`.

2. Replace std APIs with altio's equivalents, e.g. replace `println!(...)` with
`writeln!( io.out(), ... )`, replace `std::io::stdin()` with `io.input()`.

3. Keep main.rs as simple as possible, e.g.
`fn main() { the_tool::run( std::env::args_os() )}`.

# Example for tool authors

```toml
[dependencies]
altio = { version = "0.2", no_default_features = true }

[features]
altio = ["altio/altio"]
```

```rust,no_run
// lib.rs
pub struct TheTool {
    // fields omitted
    pub io: Altio,
}

impl_altio_output!( TheTool );
```

When building the tool as an application, the "altio" feature is disabled and
altio falls back to stdio.

When building the tool as a library, the tool users can invoke send/recv methods
to communicate with the tool, e.g. `send_line()`, `try_recv_line()`.

# Example for tool users

```toml
the_tool = { version = "1.0", features = ["altio"] }
```

```rust,no_run
let args = std::env::args_os(); // clap::Parser::parse_from()
let tool = the_tool::new();
let tool_io = tool.io.clone();

// `io.input().read_line()` called occasionally
std::thread::spawn( || tool.run( args ));

loop {
    if let Some( received ) = tool_io.try_recv_line() {
        if received == "Lorum" {
            tool_io.send_line( "Ipsum" );
        }
    }
}
```

# License

Under Apache License 2.0 or MIT License, at your will.
