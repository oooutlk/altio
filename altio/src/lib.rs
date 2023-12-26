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
//! this crate can help to automating input/output, just 3 steps:
//!
//! 1. Use proc macro attribute to #[macro@define] a module, e.g. `#[::altio::define] pub mod io {}`.
//!
//! 2. Replace std APIs with altio's equivalents, e.g. replace `println!()` with `io_println!()`,
//! replace `std::io::stdin()` with `io::altin()`.
//!
//! 3. Keep main.rs as simple as possible, e.g. `fn main() { the_tool::run( std::env::args_os() )}`.
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
//! When building the tool as a library, the tool users can invoke send/recv methods to communicate with the tool,
//! e.g. `send_line()`, `try_recv_line()`.
//!
//! # Example for tool users
//!
//! ```toml
//! the_tool = { version = "0.1", features = ["altio"] }
//! ```
//!
//! ```rust,no_run
//! let args = std::env::args_os(); // clap::Parser::parse_from()
//! std::thread::spawn( || the_tool::run( args ) ); // `io::altin().read_line()` called occasionally
//!
//! loop {
//!     if let Some( received ) = the_tool::io::try_recv_line() {
//!         if received == "The author published altio-0.1.0 in 2023.12.25." {
//!             io::send_line( "Happy birthday to him!".to_owned() );
//!         }
//!     }
//! }
//! ```
//!
//! # License
//!
//! Under Apache License 2.0 or MIT License, at your will.

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemMod};

/// Defines a `mod`, providing APIs to simulate piped io in process.
///
/// 1. macros io_print!(), io_println!(), io_eprint!(), io_eprintln!(),
/// and functions io::altin(), io::altout(), io::alterr()
/// to alternate between altio and stdio by enabling/disabling the "altio" feature.
///
/// 2. various transmission APIs:
/// send(), send_line(),
/// recv(), recv_err(), try_recv(), try_recv_err(),
/// recv_line(), recv_err_line(), try_recv_line(), try_recv_err_line()
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
    let ItemMod{ attrs, vis, unsafety, mod_token:_, ident, content, semi:_ }
        = syn::parse::<ItemMod>( input ).expect("#[define] a `mod`.");

    let items = content.map( |content| content.1 ).unwrap_or_default();

    fn make_ident( sym: &str ) -> Ident { Ident::new( sym, Span::call_site().into() )}

    let  print   = make_ident( &format!( "{ident}_print"    ));
    let  println = make_ident( &format!( "{ident}_println"  ));
    let eprint   = make_ident( &format!( "{ident}_eprint"   ));
    let eprintln = make_ident( &format!( "{ident}_eprintln" ));

    quote! {
        #[macro_use] #(#attrs)* #vis #unsafety mod #ident {
            #[cfg( feature = "reexport-stdio" )]
            pub use std::io::*;

            use once_cell ::sync::Lazy;
            use std::{
                fmt::Arguments,
                io::{Read, Result, Stderr, Stdin, Stdout},
                ops::{Deref, DerefMut},
                sync::{Mutex, MutexGuard},
            };

            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! #print    {($($tt:tt)+) => {print!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! #println  {($($tt:tt)+) => {println!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! #eprint   {($($tt:tt)+) => {eprint!($($tt)+)}}
            #[cfg(not(feature="altio"))] #[macro_export] macro_rules! #eprintln {($($tt:tt)+) => {eprintln!($($tt)+)}}

            #[cfg(not(feature="altio"))] pub fn altin()  -> Stdin  { std::io::stdin()  }
            #[cfg(not(feature="altio"))] pub fn altout() -> Stdout { std::io::stdout() }
            #[cfg(not(feature="altio"))] pub fn alterr() -> Stderr { std::io::stderr() }

            #[cfg( feature = "altio" )]
            pub(crate)  static ALT_IN : Lazy<Mutex<String>> = Lazy::new( || Mutex::new( String::new() ));

            #[cfg( feature = "altio" )]
            pub(crate)  static ALT_OUT: Lazy<Mutex<String>> = Lazy::new( || Mutex::new( String::new() ));

            #[cfg( feature = "altio" )]
            pub(crate)  static ALT_ERR: Lazy<Mutex<String>> = Lazy::new( || Mutex::new( String::new() ));

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! #print {
                ($($tt:tt)+) => {{
                    let text = format!( $($tt)+ );
                    if !text.is_empty() {
                        crate::#ident::altout().lock().push_str( &text );
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! #println {
                ($($tt:tt)+) => {{
                    let mut text = format!( $($tt)+ );
                    text.push( '\n' );
                    crate::#ident::altout().lock().push_str( &text );
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! #eprint {
                ($($tt:tt)+) => {{
                    let text = format!( $($tt)+ );
                    if !text.is_empty() {
                        crate::#ident::alterr().lock().push_str( &text );
                    }
                }};
            }

            #[cfg( feature = "altio" )]
            #[macro_export]
            macro_rules! #eprintln {
                ($($tt:tt)+) => {{
                    let mut text = format!( $($tt)+ );
                    text.push( '\n' );
                    crate::#ident::alterr().lock().push_str( &text );
                }};
            }

            #[cfg( feature = "altio" )]
            #[derive( Debug )]
            pub struct Altin(());

            #[cfg( feature = "altio" )]
            pub struct AltinLock<'a> {
                inner: MutexGuard<'a, String>,
            }

            #[cfg( feature = "altio" )]
            impl<'a> AltinLock<'a> {
                pub fn read_line( &mut self, buf: &mut String ) -> Result<usize> {
                    if let Some( offset ) = self.inner.find( '\n' ) {
                        buf.extend( self.inner.drain( ..=offset ));
                        Ok( buf.len() )
                    } else {
                        Ok( 0 )
                    }
                }
                pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
                    if !self.inner.is_empty() {
                        let len = self.inner.len();
                        buf.extend( self.inner.drain(..) );
                        Ok( len )
                    } else {
                        Ok(0)
                    }
                }
                pub fn is_terminal( &self ) -> bool { false }
            }

            pub struct Lines<'a> {
                inner: MutexGuard<'a, String>,
            }

            impl<'a> Iterator for Lines<'a> {
                type Item = String;
                fn next( &mut self ) -> Option<String> {
                    self.inner
                        .find( '\n' )
                        .map( |offset| String::from_iter( self.inner.drain( ..=offset )))
                }
            }

            #[cfg( feature = "altio" )]
            impl Altin {
                pub fn lock( &self ) -> AltinLock<'static> {
                    loop {
                        if let Ok( lock ) = ALT_IN.lock() {
                            break AltinLock{ inner: lock };
                        }
                    }
                }
                pub fn lines( self ) -> Lines<'static> {
                    loop {
                        if let Ok( lock ) = ALT_IN.lock() {
                            break Lines{ inner: lock };
                        }
                    }
                }
                pub fn read_line( &self, buf: &mut String ) -> Result<usize> {
                    loop {
                        if let Ok( ref mut input ) = crate::#ident::ALT_IN.lock() {
                            if let Some( offset ) = input.find( '\n' ) {
                                buf.extend( input.drain( ..=offset ));
                                return Ok( buf.len() );
                            }
                        }
                    }
                }
                pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
                    loop {
                        if let Ok( ref mut input ) = crate::#ident::ALT_IN.lock() {
                            if !input.is_empty() {
                                let len = input.len();
                                buf.extend( input.drain(..) );
                                return Ok( len );
                            }
                        }
                    }
                }
                pub fn is_terminal( &self ) -> bool { false }
            }

            #[cfg( feature = "altio" )]
            #[derive( Debug )]
            pub struct Altout(());

            #[cfg( feature = "altio" )]
            pub struct AltoutLock<'a> {
                inner: MutexGuard<'a, String>,
            }

            #[cfg( feature = "altio" )]
            impl<'a> AltoutLock<'a> {
                pub fn write_fmt( &mut self, args: Arguments<'_> ) -> Result<()> {
                    use std::fmt::Write;
                    self.inner.write_fmt( args ).map_err( |e| unreachable!() )
                }
            }

            #[cfg( feature = "altio" )]
            impl<'a> Deref for AltoutLock<'a> {
                type Target = String;
                fn deref( &self ) -> &String {
                    self.inner.deref()
                }
            }

            #[cfg( feature = "altio" )]
            impl<'a> DerefMut for AltoutLock<'a> {
                fn deref_mut( &mut self ) -> &mut String {
                    self.inner.deref_mut()
                }
            }

            #[cfg( feature = "altio" )]
            impl Altout {
                pub fn lock( &self ) -> AltoutLock<'static> {
                    loop {
                        if let Ok( lock ) = ALT_OUT.lock() {
                            return AltoutLock{ inner: lock };
                        }
                    }
                }
                pub fn write_fmt( &mut self, args: Arguments<'_> ) -> Result<()> {
                    use std::fmt::Write;
                    self.lock().inner.write_fmt( args ).map_err( |e| unreachable!() )
                }
                pub fn flush( &mut self ) -> Result<()> {
                    Ok(())
                }
                pub fn is_terminal( &self ) -> bool { false }
            }

            #[cfg( feature = "altio" )]
            #[derive( Debug )]
            pub struct Alterr(());

            #[cfg( feature = "altio" )]
            pub struct AlterrLock<'a> {
                inner: MutexGuard<'a, String>,
            }

            #[cfg( feature = "altio" )]
            impl<'a> AlterrLock<'a> {
                pub fn write_fmt( &mut self, args: Arguments<'_> ) -> Result<()> {
                    use std::fmt::Write;
                    self.inner.write_fmt( args ).map_err( |e| unreachable!() )
                }
            }

            #[cfg( feature = "altio" )]
            impl<'a> Deref for AlterrLock<'a> {
                type Target = String;
                fn deref( &self ) -> &String {
                    self.inner.deref()
                }
            }

            #[cfg( feature = "altio" )]
            impl<'a> DerefMut for AlterrLock<'a> {
                fn deref_mut( &mut self ) -> &mut String {
                    self.inner.deref_mut()
                }
            }

            #[cfg( feature = "altio" )]
            impl Alterr {
                pub fn lock( &self ) -> AlterrLock<'static> {
                    loop {
                        if let Ok( lock ) = ALT_ERR.lock() {
                            return AlterrLock{ inner: lock };
                        }
                    }
                }
                pub fn write_fmt( &mut self, args: Arguments<'_> ) -> Result<()> {
                    use std::fmt::Write;
                    self.lock().inner.write_fmt( args ).map_err( |e| unreachable!() )
                }
                pub fn flush( &mut self ) -> Result<()> {
                    Ok(())
                }
                pub fn is_terminal( &self ) -> bool { false }
            }

            #[cfg( feature = "altio" )] pub fn altin()  -> Altin  { Altin(())  }
            #[cfg( feature = "altio" )] pub fn altout() -> Altout { Altout(()) }
            #[cfg( feature = "altio" )] pub fn alterr() -> Alterr { Alterr(()) }

            #[cfg( feature = "altio" )]
            pub fn send( text: &str ) {
                if !text.is_empty() {
                    loop {
                        if let Ok( mut buf ) = crate::#ident::ALT_IN.lock() {
                            buf.push_str( text );
                            return;
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn send_line( text: &str ) {
                loop {
                    if let Ok( mut buf ) = crate::#ident::ALT_IN.lock() {
                        buf.push_str( text );
                        buf.push( '\n' );
                        return;
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
                    if !buf.is_empty() {
                        let mut received = String::new();
                        std::mem::swap( &mut received, buf );
                        return Some( received );
                    }
                }
                None
            }

            #[cfg( feature = "altio" )]
            pub fn try_recv_err() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALT_ERR.try_lock() {
                    if !buf.is_empty() {
                        let mut received = String::new();
                        std::mem::swap( &mut received, buf );
                        return Some( received );
                    }
                }
                None
            }

            #[cfg( feature = "altio" )]
            pub fn recv_line() -> String {
                loop {
                    if let Ok( ref mut buf ) = crate::#ident::ALT_OUT.lock() {
                        if let Some( offset ) = buf.find( '\n' ) {
                            return String::from_iter( buf.drain( ..=offset ));
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn recv_err_line() -> String {
                loop {
                    if let Ok( ref mut buf ) = crate::#ident::ALT_ERR.lock() {
                        if let Some( offset ) = buf.find( '\n' ) {
                            return String::from_iter( buf.drain( ..=offset ));
                        }
                    }
                }
            }

            #[cfg( feature = "altio" )]
            pub fn try_recv_line() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALT_OUT.try_lock() {
                    if let Some( offset ) = buf.find( '\n' ) {
                        return Some( String::from_iter( buf.drain( ..=offset )));
                    }
                }
                None
            }

            #[cfg( feature = "altio" )]
            pub fn try_recv_err_line() -> Option<String> {
                if let Ok( ref mut buf ) = crate::#ident::ALT_ERR.try_lock() {
                    if let Some( offset ) = buf.find( '\n' ) {
                        return Some( String::from_iter( buf.drain( ..=offset )));
                    }
                }
                None
            }

            #(#items)*
        }
    }.into()
}
