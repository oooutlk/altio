This crate helps to automating command tools by simulating piped io in process.

# Dependencies

To use this crate, you must `cargo add --optional once_cell`.

# Why this crate

Interactive command tools utilize stdin, stdout and stderr for communication.

Sometimes automating is required, and we could use something like TCL Expect to makes this stuff trivial.

With the support from the authors of the tools, automating could be even more trivial. Just 4 steps:

1. Use this project to #[macro@define] a module, e.g. `#[::altio::define] pub mod io {}`.

2. Add prefix "alt_" to `print!()`, `println!()`, `eprint!()`, `eprintln!()`,
making them `alt_print!()`, `alt_println!()`, `alt_eprint!()`, `alt_eprintln!()`.

3. Put most of the code in the tool's lib.rs and sub `mod`s, keeping main.rs as simple as possible,
e.g. `fn main() { the_tool::run() }`.

4. Use `send()`, `send_line()`, `receive()`, `receive_err()`, `try_receive()` and `try_receive_err()`
to communicate to the tool, and use `read_to_string()`, `read_line()` in the tool to get the input of the tool users. 

# Example for tool users

```toml
the_tool = { version = "0.1", features = ["altio"] }
```

```rust,no_run

std::thread::spawn( || the_tool::run() ); // `read_to_string()`, `read_line()` called occasionally

use crate::io; // defined by `#[::altio::define] pub mod io {}`

loop {
    if let Some( received ) = io::try_receive() { /* omit */ }
}
```

# Example for tool authors

```toml
[dependencies]
altio = "0.1"
once_cell = { version = "1.19.0", optional = true }

[features]
altio = ["once_cell"]
```

in lib.rs:

```rust,no_run
#[::altio::define] pub mod io {}
```

When building main.rs, the "altio" feature is disabled and altio falls back to stdio.

# License

Under Apache License 2.0 or MIT License, at your will.
