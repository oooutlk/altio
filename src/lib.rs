//! This crate helps to automating command tools by simulating piped io in process.
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! altio = { version = "0.2", default-features = false }
//!
//! [features]
//! altio = ["altio/altio"]
//! ```
//!
//! # Why this crate
//!
//! Interactive command tools utilize stdin, stdout and stderr for communication.
//! If you want to use command tools as libraries(no spawning processes) and tool authors agree,
//! this crate can help to automating input/output, just 3 steps:
//!
//! 1. Define an `Altio` variable e.g. `let io = Altio::default();`.
//!
//! 2. Replace std APIs with altio's equivalents, e.g. replace `println!(...)` with
//! `writeln!( io.out(), ... )`, replace `std::io::stdin()` with `io.input()`.
//!
//! 3. Keep main.rs as simple as possible, e.g. `fn main() { the_tool::run( std::env::args_os() )}`.
//!
//! # License
//!
//! Under Apache License 2.0 or MIT License, at your will.

use std::{
    fmt::Arguments,
    io::Result,
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard},
};

/// This macro `write`s formatted data into a buffer, or panic on failures.
///
/// In the form of `echo!( -n, ... )`, the data will be written as is, otherwise an
/// additional new line will be appended.
#[macro_export]
macro_rules! echo {
    ( -n, $dst:expr, $($tt:tt)+) => {{
        #[cfg( all( feature="altio", debug_assertions ))]
        eprint!( $($tt)+ );

        write!( $dst, $($tt)+).unwrap()
    }};
    ( $dst:expr, $($tt:tt)+) => {{
        #[cfg( all( feature="altio", debug_assertions ))]
        eprintln!( $($tt)+ );

        writeln!( $dst, $($tt)+).unwrap()
    }};
}

/// Corresponding to std::io::StdinLock
pub struct AltinLock<'a> {
    inner: MutexGuard<'a, String>,
}

impl<'a> AltinLock<'a> {
    /// Reads a line of input, appending it to the specified buffer.
    pub fn read_line( &mut self, buf: &mut String ) -> Result<usize> {
        if let Some( offset ) = self.inner.find( '\n' ) {
            buf.extend( self.inner.drain( ..=offset ));
            Ok( buf.len() )
        } else {
            Ok( 0 )
        }
    }

    /// Reads all contents in this source, appending them to buf.
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        if !self.inner.is_empty() {
            let len = self.inner.len();
            buf.extend( self.inner.drain(..) );
            Ok( len )
        } else {
            Ok(0)
        }
    }

    /// Returns false to indicate it isn't a terminal/tty.
    pub fn is_terminal( &self ) -> bool { false }
}

/// Corresponding to `std::io::Lines`
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

/// Corresponding to std::io::Stdin
#[derive( Debug, Default )]
pub struct Altin( Mutex<String> );

impl Altin {
    /// Locks this handle to the altio input stream, returning a readable guard.
    ///
    /// The lock is released when the returned lock goes out of scope.
    /// The returned guard also provides read_line(), read_to_string(), is_terminal()
    /// for accessing the underlying data.
    pub fn lock( &self ) -> AltinLock<'_> {
        loop {
            if let Ok( lock ) = self.0.lock() {
                break AltinLock{ inner: lock };
            }
        }
    }

    /// Consumes this handle and returns an iterator over input lines.
    pub fn lines( &self ) -> Lines<'_> {
        loop {
            if let Ok( lock ) = self.0.lock() {
                break Lines{ inner: lock };
            }
        }
    }

    /// Locks this handle and reads a line of input, appending it to the specified buffer.
    pub fn read_line( &self, buf: &mut String ) -> Result<usize> {
        loop {
            if let Ok( ref mut input ) = self.0.lock() {
                if let Some( offset ) = input.find( '\n' ) {
                    buf.extend( input.drain( ..=offset ));
                    return Ok( buf.len() );
                }
            }
        }
    }

    /// Read all contents in this source, appending them to buf.
    pub fn read_to_string(&self, buf: &mut String) -> Result<usize> {
        loop {
            if let Ok( ref mut input ) = self.0.lock() {
                if !input.is_empty() {
                    let len = input.len();
                    buf.extend( input.drain(..) );
                    return Ok( len );
                }
            }
        }
    }

    /// Returns false to indicate it isn't a terminal/tty.
    pub fn is_terminal( &self ) -> bool { false }
}

/// Corresponding to std::io::StdoutLock
pub struct AltoutLock<'a> {
    inner: MutexGuard<'a, String>,
}

impl<'a> AltoutLock<'a> {
    /// Writes a formatted string into Altout, won't returning any error.
    pub fn write_fmt( &mut self, args: Arguments<'_> ) -> Result<()> {
        use std::fmt::Write;
        self.inner.write_fmt( args ).map_err( |_| unreachable!() )
    }
}

impl<'a> Deref for AltoutLock<'a> {
    type Target = String;
    fn deref( &self ) -> &String {
        self.inner.deref()
    }
}

impl<'a> DerefMut for AltoutLock<'a> {
    fn deref_mut( &mut self ) -> &mut String {
        self.inner.deref_mut()
    }
}

/// Corresponding to std::io::Stdout
#[derive( Debug, Default )]
pub struct Altout( Mutex<String> );

impl Altout {
    /// Locks this handle to the altio output stream, returning a writable guard.
    ///
    /// The lock is released when the returned lock goes out of scope. The returned
    /// guard also provide write_fmt() for writing data.
    pub fn lock( &self ) -> AltoutLock<'_> {
        loop {
            if let Ok( lock ) = self.0.lock() {
                return AltoutLock{ inner: lock };
            }
        }
    }
    /// Writes a formatted string into Altout, won't returning any error.
    pub fn write_fmt( &mut self, args: Arguments<'_> ) -> Result<()> {
        use std::fmt::Write;
        self.lock().inner.write_fmt( args ).map_err( |_| unreachable!() )
    }
    /// No-op.
    pub fn flush( &mut self ) -> Result<()> {
        Ok(())
    }

    /// Returns false to indicate it isn't a terminal/tty.
    pub fn is_terminal( &self ) -> bool { false }
}

#[inline]
fn get_lines<'a>( buf: &mut MutexGuard<'a,String>, mut cnt: usize, peek_only: bool ) -> Option<String> {
    let mut offset = 0;
    while let Some( mut off ) = buf[offset..].find( '\n' ) {
        off += 1;
        offset += off;
        cnt -= 1;
        if cnt == 0 {
            break;
        }
    }
    if cnt != 0 {
        None
    } else if peek_only {
        Some( buf[ ..offset ].to_owned() )
    } else {
        Some( String::from_iter( buf.drain( ..offset )))
    }
}

impl Altin {
    /// Sends text to altio input stream, without additional newline.
    pub fn send( &self, text: &str ) {
        if !text.is_empty() {
            loop {
                if let Ok( mut buf ) = self.0.lock() {
                    buf.push_str( text );
                    return;
                }
            }
        }
    }

    /// Sends text to altio input stream, with an additional newline.
    pub fn send_line( &self, text: &str ) {
        loop {
            if let Ok( mut buf ) = self.0.lock() {
                buf.push_str( text );
                buf.push( '\n' );
                return;
            }
        }

    }
}

impl Altout {
    /// Receives text from altio output stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv( &self ) -> String {
        loop {
            if let Ok( ref mut buf ) = self.0.lock() {
                if !buf.is_empty() {
                    let mut received = String::new();
                    std::mem::swap( &mut received, buf );
                    return received;
                }
            }
        }
    }

    /// Tries to receive text from altio output stream, without blocking.
    pub fn try_recv( &self ) -> Option<String> {
        if let Ok( ref mut buf ) = self.0.try_lock() {
            if !buf.is_empty() {
                let mut received = String::new();
                std::mem::swap( &mut received, buf );
                return Some( received );
            }
        }
        None
    }

    /// Receives one line of text from altio output stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_line( &self ) -> String {
        loop {
            if let Ok( ref mut buf ) = self.0.lock() {
                if let Some( offset ) = buf.find( '\n' ) {
                    return String::from_iter( buf.drain( ..=offset ));
                }
            }
        }
    }

    /// Tries to receive one line of text from altio output stream, without blocking.
    pub fn try_recv_line( &self ) -> Option<String> {
        if let Ok( ref mut buf ) = self.0.try_lock() {
            if let Some( offset ) = buf.find( '\n' ) {
                return Some( String::from_iter( buf.drain( ..=offset )));
            }
        }
        None
    }

    /// Receives certain amount lines of text from altio output stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_lines( &self, cnt: usize ) -> String {
        if cnt == 0 {
            String::new()
        } else {
            loop {
                if let Some( received ) = self.try_recv_lines( cnt ) {
                    break received;
                }
            }
        }
    }

    /// Tries to receive certain amount lines of text from altio output stream.
    pub fn try_recv_lines( &self, cnt: usize ) -> Option<String> {
        if cnt != 0 {
            if let Ok( ref mut buf ) = self.0.try_lock() {
                return get_lines( buf, cnt, false );
            }
        }
        None
    }

    /// Read one line of text in altio output stream, leaving it in the stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn peek_line( &self ) -> Option<String> {
        if let Ok( ref mut buf ) = self.0.try_lock() {
            if let Some( offset ) = buf.find( '\n' ) {
                return Some( buf[ ..=offset ].to_owned() );
            }
        }
        None
    }

    /// Tries to receive certain amount lines of text in altio output stream,
    /// leaving it in the stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn peek_lines( &self, cnt: usize ) -> Option<String> {
        if cnt != 0 {
            if let Ok( ref mut buf ) = self.0.try_lock() {
                return get_lines( buf, cnt, true );
            }
        }
        None
    }
}

#[cfg( not( feature = "altio" ))]
#[derive( Debug, Default )]
/// Placeholder for simulating a program's Stdin,Stdout,Stderr.
pub struct Altio;

#[cfg( not( feature = "altio" ))]
impl Altio {
    /// Returns `Stdin`.
    pub fn input( &self ) -> std::io::Stdin { std::io::stdin() }
    /// Returns `Stdout`.
    pub fn out( &self ) -> std::io::Stdout { std::io::stdout() }
    /// Returns `Stderr`.
    pub fn err( &self ) -> std::io::Stderr { std::io::stderr() }
}

#[cfg( feature = "altio" )]
#[derive( Clone, Debug, Default )]
/// Simulates a program's Stdin,Stdout,Stderr.
pub struct Altio( std::sync::Arc<(Altin, Altout, Altout)> );

#[cfg( feature = "altio" )]
impl Altio {
    /// Corresponding to Stdin.
    pub fn input( &self ) -> &Altin { &self.0.0 }

    /// Corresponding to Stdout.
    pub fn out( &self ) -> AltoutLock { self.0.1.lock() }

    /// Corresponding to Stderr.
    pub fn err( &self ) -> AltoutLock { self.0.2.lock() }

    /// Sends text to altio input stream, without additional newline.
    pub fn send( &self, text: &str ) { self.0.0.send( text )}

    /// Sends text to altio input stream, with an additional newline.
    pub fn send_line( &self, text: &str ) { self.0.0.send_line( text )}

    /// Receives text from altio output stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv( &self ) -> String { self.0.1.recv() }

    /// Tries to receive text from altio output stream, without blocking.
    pub fn try_recv( &self ) -> Option<String> { self.0.1.try_recv() }

    /// Receives one line of text from altio output stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_line( &self ) -> String { self.0.1.recv_line() }

    /// Tries to receive one line of text from altio output stream, without blocking.
    pub fn try_recv_line( &self ) -> Option<String> { self.0.1.try_recv_line() }

    /// Receives certain amount lines of text from altio output stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_lines( &self, cnt: usize ) -> String { self.0.1.recv_lines(cnt) }

    /// Tries to receive certain amount lines of text from altio output stream.
    pub fn try_recv_lines( &self, cnt: usize ) -> Option<String> { self.0.1.try_recv_lines(cnt) }

    /// Reads one line of text in altio output stream, leaving it in the
    /// stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn peek_line( &self ) -> Option<String> { self.0.1.peek_line() }

    /// Reads certain amount lines of text in altio output stream, leaving it in the
    /// stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn peek_lines( &self, cnt: usize ) -> Option<String> { self.0.1.peek_lines(cnt) }

    /// Receives text from altio error stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_err( &self ) -> String { self.0.2.recv() }

    /// Tries to receive text from altio error stream, without blocking.
    pub fn try_recv_err( &self ) -> Option<String> { self.0.2.try_recv() }

    /// Receives one line of text from altio error stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_err_line( &self ) -> String { self.0.2.recv_line() }

    /// Tries to receive one line of text from altio error stream, without blocking.
    pub fn try_recv_err_line( &self ) -> Option<String> { self.0.2.try_recv_line() }

    /// Receives certain amount lines of text from altio error stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn recv_err_lines( &self, cnt: usize ) -> String { self.0.2.recv_lines(cnt) }

    /// Tries to receive certain amount lines of text from altio error stream, without
    /// blocking.
    pub fn try_recv_err_lines( &self, cnt: usize ) -> Option<String> { self.0.2.try_recv_lines(cnt) }

    /// Reads one line of text in altio error stream, leaving it in the stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn peek_err_line( &self ) -> Option<String> { self.0.2.peek_line() }

    /// Reads certain amount line of text in altio error stream, leaving it in the
    /// stream.
    ///
    /// This function will always block the current thread if there is no data
    /// available.
    pub fn peek_err_lines( &self, cnt: usize ) -> Option<String> { self.0.2.peek_lines(cnt) }
}

/// Provides delegated `out()`/`err()` methods for the type which contains a field
/// named `altio`.
#[macro_export]
macro_rules! impl_altio_output {
    ($ty:ty) => {
        #[cfg( feature = "altio" )]
        impl $ty {
            pub fn out( &self ) -> altio::AltoutLock { self.altio.out() }
            pub fn err( &self ) -> altio::AltoutLock { self.altio.err() }
        }

        #[cfg( not( feature = "altio" ))]
        impl $ty {
            pub fn out( &self ) -> std::io::Stdout { std::io::stdout() }
            pub fn err( &self ) -> std::io::Stderr { std::io::stderr() }
        }
    };
}

#[cfg( all( test, feature="altio" ))]
pub mod tests {
    use super::{Altio, echo};

    use std::io::Result;

    const ALPHABET: &'static str = "abcdefg\nhijklmn\nopq rst\nuvw xyz";

    #[test]
    fn altin_lock_read_line() -> Result<()> {
        let io = Altio::default();

        io.send_line( ALPHABET );

        let mut lock = io.input().lock();
        let mut buf = String::new();

        lock.read_line( &mut buf )?;
        assert_eq!( buf, "abcdefg\n" );

        lock.read_line( &mut buf )?;
        assert_eq!( buf, "abcdefg\nhijklmn\n" );

        lock.read_line( &mut buf )?;
        assert_eq!( buf, "abcdefg\nhijklmn\nopq rst\n" );

        lock.read_line( &mut buf )?;
        assert_eq!( buf, "abcdefg\nhijklmn\nopq rst\nuvw xyz\n" );

        Ok(())
    }

    #[test]
    fn altin_lock_read_to_string() -> Result<()> {
        let io = Altio::default();

        io.send( ALPHABET );

        let mut lock = io.input().lock();
        let mut buf = String::new();

        lock.read_to_string( &mut buf )?;
        assert_eq!( buf, ALPHABET );

        Ok(())
    }

    #[test]
    fn lines() {
        let io = Altio::default();

        assert!( io.input().lines().collect::<String>().is_empty() );

        io.send( ALPHABET );
        assert_eq!( io.input().lines().collect::<Vec<String>>(),
            vec![ "abcdefg\n".to_owned(), "hijklmn\n".to_owned(), "opq rst\n".to_owned() ]);
    }

    #[test]
    fn altin_read_line() -> Result<()> {
        let io = Altio::default();

        io.send( ALPHABET );

        let mut buf = String::new();
        io.input().read_line( &mut buf )?;
        assert_eq!( buf, "abcdefg\n" );

        Ok(())
    }

    #[test]
    fn altin_read_to_string() -> Result<()> {
        let io = Altio::default();

        io.send( ALPHABET );

        let mut buf = String::new();
        io.input().read_to_string( &mut buf )?;
        assert_eq!( buf, ALPHABET );

        Ok(())
    }

    #[test]
    fn altout_lock_write_fmt() -> Result<()> {
        let io = Altio::default();

        {
            let mut lock = io.out();
            let contents = ALPHABET;
            for line in contents.lines() {
                writeln!( lock, "{}", line )?;
            }
        }

        assert_eq!( io.recv().trim(), ALPHABET );

        Ok(())
    }

    #[test]
    fn altout_write_fmt() -> Result<()> {
        let io = Altio::default();

        let contents = ALPHABET;
        for line in contents.lines() {
            writeln!( io.out(), "{}", line )?;
        }

        assert_eq!( io.recv().trim(), ALPHABET );

        Ok(())
    }
    #[test]
    fn alterr_lock_write_fmt() -> Result<()> {
        let io = Altio::default();

        {
            let mut lock = io.err();
            let contents = ALPHABET;
            for line in contents.lines() {
                writeln!( lock, "{}", line )?;
            }
        }

        assert_eq!( io.recv_err().trim(), ALPHABET );

        Ok(())
    }

    #[test]
    fn alterr_write_fmt() -> Result<()> {
        let io = Altio::default();

        let contents = ALPHABET;
        for line in contents.lines() {
            writeln!( io.err(), "{}", line )?;
        }

        assert_eq!( io.recv_err().trim(), ALPHABET );

        Ok(())
    }

    #[test]
    fn nothing_received() {
        let io = Altio::default();

        assert!( io.try_recv().is_none() );
        assert!( io.try_recv_line().is_none() );
        assert!( io.try_recv_err().is_none() );
        assert!( io.try_recv_err_line().is_none() );
    }

    #[test]
    fn io_print() {
        { let io = Altio::default(); echo!( -n, io.out(), "" ); assert!( io.try_recv().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.out(), "" ); assert!( io.try_recv_line().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.out(), "" ); assert!( io.try_recv_err().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.out(), "" ); assert!( io.try_recv_err_line().is_none() ); }

        { let io = Altio::default(); echo!( -n, io.out(), " " ); assert!( io.try_recv().is_some() ); }
        { let io = Altio::default(); echo!( -n, io.out(), " " ); assert!( io.try_recv_line().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.out(), " " ); assert!( io.try_recv_err().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.out(), " " ); assert!( io.try_recv_err_line().is_none() ); }

        { let io = Altio::default(); echo!( -n, io.out(), "\n" ); assert!( io.try_recv().is_some() ); }
        { let io = Altio::default(); echo!( -n, io.out(), "\n" ); assert!( io.try_recv_line().is_some() ); }
        { let io = Altio::default(); echo!( -n, io.out(), "\n" ); assert!( io.try_recv_err().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.out(), "\n" ); assert!( io.try_recv_err_line().is_none() ); }
    }

    #[test]
    fn io_println() {
        { let io = Altio::default(); echo!( io.out(), "" ); assert!( io.try_recv().is_some() ); }
        { let io = Altio::default(); echo!( io.out(), "" ); assert!( io.try_recv_line().is_some() ); }
        { let io = Altio::default(); echo!( io.out(), "" ); assert!( io.try_recv_err().is_none() ); }
        { let io = Altio::default(); echo!( io.out(), "" ); assert!( io.try_recv_err_line().is_none() ); }
    }

    #[test]
    fn io_eprint() {
        { let io = Altio::default(); echo!( -n, io.err(), "" ); assert!( io.try_recv().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), "" ); assert!( io.try_recv_line().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), "" ); assert!( io.try_recv_err().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), "" ); assert!( io.try_recv_err_line().is_none() ); }

        { let io = Altio::default(); echo!( -n, io.err(), " " ); assert!( io.try_recv().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), " " ); assert!( io.try_recv_line().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), " " ); assert!( io.try_recv_err().is_some() ); }
        { let io = Altio::default(); echo!( -n, io.err(), " " ); assert!( io.try_recv_err_line().is_none() ); }

        { let io = Altio::default(); echo!( -n, io.err(), "\n" ); assert!( io.try_recv().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), "\n" ); assert!( io.try_recv_line().is_none() ); }
        { let io = Altio::default(); echo!( -n, io.err(), "\n" ); assert!( io.try_recv_err().is_some() ); }
        { let io = Altio::default(); echo!( -n, io.err(), "\n" ); assert!( io.try_recv_err_line().is_some() ); }
    }

    #[test]
    fn io_eprintln() {
        { let io = Altio::default(); echo!( io.err(), "" ); assert!( io.try_recv().is_none() ); }
        { let io = Altio::default(); echo!( io.err(), "" ); assert!( io.try_recv_line().is_none() ); }
        { let io = Altio::default(); echo!( io.err(), "" ); assert!( io.try_recv_err().is_some() ); }
        { let io = Altio::default(); echo!( io.err(), "" ); assert!( io.try_recv_err_line().is_some() ); }
    }

    #[test]
    fn receive_out() {
        let io = Altio::default();

        echo!( -n, io.out(), "" );
        assert!( io.try_recv().is_none() );

        echo!( -n, io.out(), " " );
        assert!( io.try_recv_err().is_none() );
        assert_eq!( io.try_recv(), Some( " ".to_owned() ));

        echo!( -n, io.out(), "abcdefg\nhijklmn\nopq rst\nuvw xyz" );
        assert_eq!( io.try_recv_line(), Some( "abcdefg\n".to_owned() ));
        assert_eq!( io.recv_line(), "hijklmn\n" );
        assert_eq!( io.recv(), "opq rst\nuvw xyz" );
    }

    #[test]
    fn receive_err() {
        let io = Altio::default();

        echo!( -n, io.err(), "" );
        assert!( io.try_recv_err().is_none() );

        echo!( -n, io.err(), " " );
        assert!( io.try_recv().is_none() );
        assert_eq!( io.try_recv_err(), Some( " ".to_owned() ));

        echo!( -n, io.err(), "abcdefg\nhijklmn\nopq rst\nuvw xyz" );
        assert_eq!( io.try_recv_err_line(), Some( "abcdefg\n".to_owned() ));
        assert_eq!( io.recv_err_line(), "hijklmn\n" );
        assert_eq!( io.recv_err(), "opq rst\nuvw xyz" );
    }

    #[test]
    fn receive_lines() {
        let io = Altio::default();

        echo!( -n, io.out(), "abcd\nefg\nhijk\nlmn\nopq\nrst\nuvw\nxyz" );
        assert_eq!( io.try_recv_lines(1), Some( "abcd\n".to_owned() ) );
        assert_eq!( io.try_recv_lines(2), Some( "efg\nhijk\n".to_owned() ));
        assert_eq!( io.try_recv_lines(3), Some( "lmn\nopq\nrst\n".to_owned() ));
        assert_eq!( io.try_recv_lines(2), None );
    }

    #[test]
    fn receive_err_lines() {
        let io = Altio::default();

        echo!( -n, io.err(), "abcd\nefg\nhijk\nlmn\nopq\nrst\nuvw\nxyz" );
        assert_eq!( io.try_recv_err_lines(1), Some( "abcd\n".to_owned() ) );
        assert_eq!( io.try_recv_err_lines(2), Some( "efg\nhijk\n".to_owned() ));
        assert_eq!( io.try_recv_err_lines(3), Some( "lmn\nopq\nrst\n".to_owned() ));
        assert_eq!( io.try_recv_err_lines(2), None );
    }

    #[test]
    fn peek_line() {
        let io = Altio::default();

        echo!( -n, io.out(), "abcd\nefg\nhijk\nlmn\nopq\nrst\nuvw\nxyz" );
        assert_eq!( io.peek_line(), Some( "abcd\n".to_owned() ));
        assert_eq!( io.peek_line(), Some( "abcd\n".to_owned() ));
        assert_eq!( io.recv_line(),       "abcd\n".to_owned()  );
        assert_eq!( io.recv_line(),        "efg\n".to_owned()  );
    }

    #[test]
    fn peek_err_line() {
        let io = Altio::default();

        echo!( -n, io.err(), "abcd\nefg\nhijk\nlmn\nopq\nrst\nuvw\nxyz" );
        assert_eq!( io.peek_err_line(), Some( "abcd\n".to_owned() ));
        assert_eq!( io.peek_err_line(), Some( "abcd\n".to_owned() ));
        assert_eq!( io.recv_err_line(),       "abcd\n".to_owned()  );
        assert_eq!( io.recv_err_line(),        "efg\n".to_owned()  );
    }
}
