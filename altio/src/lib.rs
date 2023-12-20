//! This crate helps to automating command tools by simulating piped io in process.
//!
//! # Dependencies
//!
//! To use this crate, you must `cargo add --optional once_cell`.
//!
//! # Why this crate
//!
//! Interactive command tools utilize stdin, stdout and stderr for communication.
//!
//! Sometimes automating is required, and we could use something like TCL Expect to makes this stuff trivial.
//!
//! With the support from the authors of the tools, automating could be even more trivial. Just 4 steps:
//!
//! 1. Use this project to #[macro@define] a module, e.g. `#[::altio::define] pub mod io {}`.
//!
//! 2. Add prefix "alt_" to `print!()`, `println!()`, `eprint!()`, `eprintln!()`,
//! making them `alt_print!()`, `alt_println!()`, `alt_eprint!()`, `alt_eprintln!()`.
//!
//! 3. Put most of the code in the tool's lib.rs and sub `mod`s, keeping main.rs as simple as possible,
//! e.g. `fn main() { the_tool::run() }`.
//!
//! 4. Use `send()`, `send_line()`, `receive()`, `receive_err()`, `try_receive()` and `try_receive_err()`
//! to communicate to the tool, and use `read_to_string()`, `read_line()` in the tool to get the input of the tool users. 
//!
//! # Example for tool users
//!
//! ```toml
//! the_tool = { version = "0.1", features = ["altio"] }
//! ```
//!
//! ```rust,no_run
//!
//! std::thread::spawn( || the_tool::run() ); // `read_to_string()`, `read_line()` called occasionally
//!
//! use crate::io; // defined by `#[::altio::define] pub mod io {}`
//!
//! loop {
//!     if let Some( received ) = io::try_receive() { /* omit */ }
//! }
//! ```
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
//! in lib.rs:
//!
//! ```rust,no_run
//! #[::altio::define] pub mod io {}
//! ```
//!
//! When building main.rs, the "altio" feature is disabled and altio falls back to stdio.
//! 
//! # License
//! 
//! Under Apache License 2.0 or MIT License, at your will.

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemMod;

/// Defines a `mod`, providing APIs to simulate piped io in process:
///
/// 1. macros alt_print!(), alt_println!(), alt_eprint!(), alt_eprintln!(), and functions read_to_string(), read_line(),
/// to alternate between altio and stdio by enabling/disabling the "altio" feature.
///
/// 2. functions send(), send_line(),receive(), receive_err(), and try_receive(),
/// to simulate piped io in process.
///
/// 3. global variables ALTIO_INPUT, ALTIO_PRINT and ALTIO_EPRINT,
/// to store input/output data in memory.
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
            #[cfg( not( feature = "altio" ))] pub fn read_to_string() -> std::io::Result<String> { std::io::read_to_string( std::io::stdin() )}
            #[cfg( not( feature = "altio" ))] pub fn read_line() -> std::io::Result<String> { let mut buf = String::new(); std::io::stdin().read_line( &mut buf )?; Ok( buf )}

            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! alt_print    {($($tt:tt)+) => {std::print!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! alt_println  {($($tt:tt)+) => {std::println!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! alt_eprint   {($($tt:tt)+) => {std::eprint!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! alt_eprintln {($($tt:tt)+) => {std::eprintln!($($tt)+)}}

            #[cfg( feature = "altio" )]
            pub static ALTIO_INPUT : once_cell::sync::Lazy<std::sync::Mutex<Option<String>>>
                = once_cell::sync::Lazy::new( || std::sync::Mutex::new( None ));

            #[cfg( feature = "altio" )]
            pub static ALTIO_PRINT : once_cell::sync::Lazy<std::sync::Mutex<Option<String>>>
                = once_cell::sync::Lazy::new( || std::sync::Mutex::new( None ));

            #[cfg( feature = "altio" )]
            pub static ALTIO_EPRINT: once_cell::sync::Lazy<std::sync::Mutex<Option<String>>>
                = once_cell::sync::Lazy::new( || std::sync::Mutex::new( None ));

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! alt_print {
                ($($tt:tt)+) => {{
                    let text = format!( $($tt)+ );
                    if !text.is_empty() {
                        if let Ok( mut buf ) = crate::#ident::ALTIO_PRINT.lock() {
                            if let Some( buf ) = buf.as_mut() {
                                buf.push_str( &text );
                            } else {
                                *buf = Some( text );
                            }
                        }
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! alt_println {
                ($($tt:tt)+) => {{
                    let mut text = format!( $($tt)+ );
                    text.push( '\n' );
                    if let Ok( mut buf ) = crate::#ident::ALTIO_PRINT.lock() {
                        if let Some( buf ) = buf.as_mut() {
                            buf.push_str( &text );
                        } else {
                            *buf = Some( text );
                        }
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! alt_eprint {
                ($($tt:tt)+) => {{
                    let text = format!( $($tt)+ );
                    if !text.is_empty() {
                        if let Ok( mut buf ) = crate::#ident::ALTIO_EPRINT.lock() {
                            if let Some( buf ) = buf.as_mut() {
                                buf.push_str( &text );
                            } else {
                                *buf = Some( text );
                            }
                        }
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! alt_eprintln {
                ($($tt:tt)+) => {{
                    let mut text = format!( $($tt)+ );
                    text.push( '\n' );
                    if let Ok( mut buf ) = crate::#ident::ALTIO_EPRINT.lock() {
                        if let Some( buf ) = buf.as_mut() {
                            buf.push_str( &text );
                        } else {
                            *buf = Some( text );
                        }
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            pub fn read_to_string() -> std::io::Result<String> {
                loop {
                    if let Ok( mut input ) = crate::#ident::ALTIO_INPUT.lock() {
                        if let Some( input ) = input.take() {
                            return Ok( input );
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn read_line() -> std::io::Result<String> {
                loop {
                    if let Ok( mut input ) = crate::#ident::ALTIO_INPUT.lock() {
                        if let Some( mut s ) = input.take() {
                            if let Some( offset ) = s.find( '\n' ) {
                                *input = Some( s.split_off( offset+1 ));
                                return Ok( s );
                            }
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn send( text: String ) {
                if !text.is_empty() {
                    loop {
                        if let Ok( mut buf ) = crate::#ident::ALTIO_INPUT.lock() {
                            if let Some( buf ) = buf.as_mut() {
                                buf.push_str( &text );
                            } else {
                                *buf = Some( text );
                            }
                            break;
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn send_line( mut text: String ) {
                text.push( '\n' );
                send( text );
            }

            #[cfg( feature = "altio" )]
            pub fn receive() -> String {
                loop {
                    if let Ok( ref mut buf ) = crate::#ident::ALTIO_PRINT.lock() {
                        if let Some( output ) = buf.take() {
                            return output;
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn receive_err() -> String {
                loop {
                    if let Ok( ref mut buf ) = crate::#ident::ALTIO_EPRINT.lock() {
                        if let Some( output ) = buf.take() {
                            return output;
                        }
                    }
                }
            }
            #[cfg( feature = "altio" )]
            pub fn try_receive() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALTIO_PRINT.try_lock() {
                    if let Some( output ) = buf.take() {
                        return Some( output );
                    }
                }
                None
            }

            #[cfg( feature = "altio" )]
            pub fn try_receive_err() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALTIO_EPRINT.try_lock() {
                    if let Some( output ) = buf.take() {
                        return Some( output );
                    }
                }
                None
            }

            #(#items)*
        }
    }.into()
}
