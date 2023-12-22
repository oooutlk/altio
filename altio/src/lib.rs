//! This crate helps to automating command tools by simulating piped io in process.
//!
//! # Dependencies
//!
//! To use this crate, you must `cargo add --optional once_cell`.
//!
//! # Why this crate
//!
//! Interactive command tools utilize stdin, stdout and stderr for communication.
//! If you want to use command tools as libraries(no spawning processes) and tool authors agree,
//! this crate can help to automating input/output, just 4 steps:
//!
//! 1. Use proc macro attribute to #[macro@define] a module, e.g. `#[::altio::define] pub mod io {}`.
//!
//! 2. Replace `print!()`, `println!()`, `eprint!()`, `eprintln!()` with `_print!()`, `_println!()`, `_eprint!()`, `_eprintln!()`.
//!
//! 3. Keep main.rs as simple as possible, e.g. `fn main() { the_tool::run( std::env::args_os() )}`.
//!
//! 4. Replace `std::io::read_to_string(std::io::stdin())`/`std::io::stdin().read_line()` with `io::read_to_string()`/`io::read_line()`.
//!
//! # Example for tool authors
//!
//! ```toml
//! [dependencies]
//! altio = "0.1"
//! once_cell = { version = "1.19.0", optional = true }
//! 
//! [features]
//! altio = ["once_cell"]
//! ```
//!
//! ```rust,no_run
//! // lib.rs
//! #[::altio::define] pub mod io {}
//! ```
//!
//! When building the tool as an application, the "altio" feature is disabled and altio falls back to stdio.
//!
//! When building the tool as a library, the tool users can use `send()`, `sendln()`, `recv()`, `recv_err()`, `try_recv()` and `try_recv_err()` to communicate to the tool,
//!
//! # Example for tool users
//!
//! ```toml
//! the_tool = { version = "0.1", features = ["altio"] }
//! ```
//!
//! ```rust,no_run
//! let args = std::env::args_os(); // clap::Parser::parse_from()
//! std::thread::spawn( || the_tool::run( args ) ); // `read_to_string()`/`read_line()` called occasionally
//!
//! loop {
//!     if let Some( received ) = the_tool::io::try_recv() { /* omit */ }
//! }
//! ```
//! 
//! # License
//! 
//! Under Apache License 2.0 or MIT License, at your will.

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemMod;

/// Defines a `mod`, providing APIs to simulate piped io in process:
///
/// 1. macros _print!(), _println!(), _eprint!(), _eprintln!(), and functions read_to_string(), read_line(),
/// to alternate between altio and stdio by enabling/disabling the "altio" feature.
///
/// 2. functions send(), sendln(), recv(), recv_err(), and try_recv(),
/// to simulate piped io in process.
///
/// 3. global variables ALT_IN, ALT_OUT and ALT_ERR, to store input/output data in memory.
///
/// # Example
///
/// ```rust,no_run
/// #[::altio::define] pub mod io {}
/// ```
#[proc_macro_attribute]
pub fn define( _args: TokenStream, input: TokenStream ) -> TokenStream {
    let ItemMod{ attrs, vis, unsafety, mod_token, ident, content, semi }
        = syn::parse::<ItemMod>( input ).expect("#[define] a `mod`.");

    let items = content.map( |content| content.1 ).unwrap_or_default();
    let _ = (mod_token, semi);

    quote! {
        #[macro_use] #(#attrs)* #vis #unsafety mod #ident {
            #[cfg(not(feature="altio"))] pub fn read_line( buf: &mut String ) -> std::io::Result<usize> { std::io::stdin().read_line( buf )}
            #[cfg(not(feature="altio"))] pub fn read_to_string() -> std::io::Result<String> { std::io::read_to_string( std::io::stdin() )}

            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! _print    {($($tt:tt)+) => {std::print!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! _println  {($($tt:tt)+) => {std::println!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! _eprint   {($($tt:tt)+) => {std::eprint!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! _eprintln {($($tt:tt)+) => {std::eprintln!($($tt)+)}}

            #[cfg( feature = "altio" )]
            pub static ALT_IN : once_cell::sync::Lazy<std::sync::Mutex<String>>
                = once_cell::sync::Lazy::new( || std::sync::Mutex::new( String::new() ));

            #[cfg( feature = "altio" )]
            pub static ALT_OUT: once_cell::sync::Lazy<std::sync::Mutex<String>>
                = once_cell::sync::Lazy::new( || std::sync::Mutex::new( String::new() ));

            #[cfg( feature = "altio" )]
            pub static ALT_ERR: once_cell::sync::Lazy<std::sync::Mutex<String>>
                = once_cell::sync::Lazy::new( || std::sync::Mutex::new( String::new() ));

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! _print {
                ($($tt:tt)+) => {{
                    let text = format!( $($tt)+ );
                    if !text.is_empty() {
                        if let Ok( mut buf ) = crate::#ident::ALT_OUT.lock() {
                            buf.push_str( &text );
                        }
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! _println {
                ($($tt:tt)+) => {{
                    let mut text = format!( $($tt)+ );
                    text.push( '\n' );
                    if let Ok( mut buf ) = crate::#ident::ALT_OUT.lock() {
                        buf.push_str( &text );
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! _eprint {
                ($($tt:tt)+) => {{
                    let text = format!( $($tt)+ );
                    if !text.is_empty() {
                        if let Ok( mut buf ) = crate::#ident::ALT_ERR.lock() {
                            buf.push_str( &text );
                        }
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! _eprintln {
                ($($tt:tt)+) => {{
                    let mut text = format!( $($tt)+ );
                    text.push( '\n' );
                    if let Ok( mut buf ) = crate::#ident::ALT_ERR.lock() {
                        buf.push_str( &text );
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            pub fn read_line( buf: &mut String ) -> std::io::Result<usize> {
                loop {
                    if let Ok( ref mut input ) = crate::#ident::ALT_IN.lock() {
                        if let Some( offset ) = input.find( '\n' ) {
                            buf.extend( input.drain( ..=offset ));
                            return Ok( buf.len() );
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn read_to_string() -> std::io::Result<String> {
                loop {
                    if let Ok( ref mut input ) = crate::#ident::ALT_IN.lock() {
                        if !input.is_empty() {
                            let mut contents = String::new();
                            std::mem::swap( &mut contents, input );
                            return Ok( contents );
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn send( text: &str ) {
                if !text.is_empty() {
                    loop {
                        if let Ok( mut buf ) = crate::#ident::ALT_IN.lock() {
                            buf.push_str( text );
                            break;
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn sendln( text: &str ) {
                loop {
                    if let Ok( mut buf ) = crate::#ident::ALT_IN.lock() {
                        buf.push_str( text );
                        buf.push( '\n' );
                        break;
                    }
                }

            }

            #[cfg( feature = "altio" )]
            pub fn recv() -> String {
                loop {
                    if let Ok( ref mut buf ) = crate::#ident::ALT_OUT.lock() {
                        if !buf.is_empty() {
                            let mut received = String::new();
                            std::mem::swap( &mut received, buf );
                            return received;
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn recv_err() -> String {
                loop {
                    if let Ok( ref mut buf ) = crate::#ident::ALT_ERR.lock() {
                        if !buf.is_empty() {
                            let mut received = String::new();
                            std::mem::swap( &mut received, buf );
                            return received;
                        }
                    }
                }
            }
            #[cfg( feature = "altio" )]
            pub fn try_recv() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALT_OUT.try_lock() {
                    return if buf.is_empty() {
                        None
                    } else {
                        let mut received = String::new();
                        std::mem::swap( &mut received, buf );
                        return Some( received );
                    };
                }
                None
            }

            #[cfg( feature = "altio" )]
            pub fn try_recv_err() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALT_ERR.try_lock() {
                    return if buf.is_empty() {
                        None
                    } else {
                        let mut received = String::new();
                        std::mem::swap( &mut received, buf );
                        return Some( received );
                    };
                }
                None
            }

            #(#items)*
        }
    }.into()
}
