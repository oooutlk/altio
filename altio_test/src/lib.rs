#[::altio::define] pub mod io {}

/// Clear input/output buffers.
///
/// ```compile_fail
/// altio_test::io::ALT_IN.lock().unwrap().clear();
/// ```
///
/// ```compile_fail
/// altio_test::io::ALT_OUT.lock().unwrap().clear();
/// ```
///
/// ```compile_fail
/// altio_test::io::ALT_ERR.lock().unwrap().clear();
/// ```
///
/// ```compile_fail
/// let _ = altio_test::io::Altin(());
///
/// ```compile_fail
/// let _ = altio_test::io::Altout(());
///
/// ```compile_fail
/// let _ = altio_test::io::Alterr(());
/// ```
pub fn clear() {
    io::ALT_IN.lock() .expect("ALT_IN.lock()") .clear();
    io::ALT_OUT.lock().expect("ALT_OUT.lock()").clear();
    io::ALT_ERR.lock().expect("ALT_ERR.lock()").clear();
}

/*fn main() {
    io_print!("Good morning, class!");
    assert_eq!( io::recv(), "Good morning, class!" );
    io_println!("Good morning, teacher!");
    assert_eq!( io::recv(), "Good morning, teacher!\n" );

    std::thread::spawn( || {
        let mut line = String::new();
        loop {
            io::altin().read_line( &mut line ).unwrap();
            io_print!( "[read_line] {line}",  );
            line.clear();
        }
    });

    io::send_line("My name is Gao Hui.");
    assert_eq!( io::recv(), "[read_line] My name is Gao Hui.\n" );

    foo::bar::baz();
}

pub mod foo {
    pub mod bar {
        pub fn baz() {
            #[macro_use]
            use crate::io;

            io_eprint!("What's your name?");
            assert_eq!( io::recv_err(), "What's your name?" );
            io_eprintln!("Li Lei.");
            assert_eq!( io::recv_err(), "Li Lei.\n" );

            io::send_line("Listen, read and say.");
            assert_eq!( io::recv(), "[read_line] Listen, read and say.\n" );
        }
    }
}
*/
#[cfg( test )]
pub mod tests {
    use once_cell ::sync::Lazy;
    use std::{
        io::Result,
        sync::{Mutex, MutexGuard},
    };
    use super::{clear, io};

    const ALPHABET: &'static str = "abcdefg\nhijklmn\nopq rst\nuvw xyz";

    // ensure running one test at a time
    static LOCK : Lazy<Mutex<()>> = Lazy::new( || Mutex::new(()));

    struct Lock<'a>( MutexGuard<'a, ()> );

    impl<'a> Lock<'a> { fn new() -> Self { Lock( LOCK.lock().unwrap() )}}
    impl<'a> Drop for Lock<'a> { fn drop( &mut self ) { clear() }}

    #[test]
    fn altin_lock_read_line() -> Result<()> {
        let _lock = Lock::new();

        io::send_line( ALPHABET );

        let mut lock = io::altin().lock();
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
        let _lock = Lock::new();

        io::send( ALPHABET );

        let mut lock = io::altin().lock();
        let mut buf = String::new();

        lock.read_to_string( &mut buf )?;
        assert_eq!( buf, ALPHABET );

        Ok(())
    }

    #[test]
    fn lines() {
        let _lock = Lock::new();

        assert!( io::altin().lines().collect::<String>().is_empty() );

        io::send( ALPHABET );
        assert_eq!( io::altin().lines().collect::<Vec<String>>(),
            vec![ "abcdefg\n".to_owned(), "hijklmn\n".to_owned(), "opq rst\n".to_owned() ]);
    }

    #[test]
    fn altin_read_line() -> Result<()> {
        let _lock = Lock::new();

        io::send( ALPHABET );

        let mut buf = String::new();
        io::altin().read_line( &mut buf )?;
        assert_eq!( buf, "abcdefg\n" );

        Ok(())
    }

    #[test]
    fn altin_read_to_string() -> Result<()> {
        let _lock = Lock::new();

        io::send( ALPHABET );

        let mut buf = String::new();
        io::altin().read_to_string( &mut buf )?;
        assert_eq!( buf, ALPHABET );

        Ok(())
    }

    #[test]
    fn altout_lock_write_fmt() -> Result<()> {
        let _lock = Lock::new();

        {
            let mut lock = io::altout().lock();
            let contents = ALPHABET;
            for line in contents.lines() {
                writeln!( lock, "{}", line )?;
            }
        }

        assert_eq!( io::recv().trim(), ALPHABET );

        Ok(())
    }

    #[test]
    fn altout_write_fmt() -> Result<()> {
        let _lock = Lock::new();

        let contents = ALPHABET;
        for line in contents.lines() {
            writeln!( io::altout(), "{}", line )?;
        }

        assert_eq!( io::recv().trim(), ALPHABET );

        Ok(())
    }
    #[test]
    fn alterr_lock_write_fmt() -> Result<()> {
        let _lock = Lock::new();

        {
            let mut lock = io::alterr().lock();
            let contents = ALPHABET;
            for line in contents.lines() {
                writeln!( lock, "{}", line )?;
            }
        }

        assert_eq!( io::recv_err().trim(), ALPHABET );

        Ok(())
    }

    #[test]
    fn alterr_write_fmt() -> Result<()> {
        let _lock = Lock::new();

        let contents = ALPHABET;
        for line in contents.lines() {
            writeln!( io::alterr(), "{}", line )?;
        }

        assert_eq!( io::recv_err().trim(), ALPHABET );

        Ok(())
    }

    #[test]
    fn nothing_received() {
        let _lock = Lock::new();

        assert!( io::try_recv().is_none() );
        assert!( io::try_recv_line().is_none() );
        assert!( io::try_recv_err().is_none() );
        assert!( io::try_recv_err_line().is_none() );
    }

    #[test]
    fn io_print() {
        let _lock = Lock::new();

        clear(); io_print!( "" ); assert!( io::try_recv().is_none() );
        clear(); io_print!( "" ); assert!( io::try_recv_line().is_none() );
        clear(); io_print!( "" ); assert!( io::try_recv_err().is_none() );
        clear(); io_print!( "" ); assert!( io::try_recv_err_line().is_none() );

        clear(); io_print!( " " ); assert!( io::try_recv().is_some() );
        clear(); io_print!( " " ); assert!( io::try_recv_line().is_none() );
        clear(); io_print!( " " ); assert!( io::try_recv_err().is_none() );
        clear(); io_print!( " " ); assert!( io::try_recv_err_line().is_none() );

        clear(); io_print!( "\n" ); assert!( io::try_recv().is_some() );
        clear(); io_print!( "\n" ); assert!( io::try_recv_line().is_some() );
        clear(); io_print!( "\n" ); assert!( io::try_recv_err().is_none() );
        clear(); io_print!( "\n" ); assert!( io::try_recv_err_line().is_none() );
    }

    #[test]
    fn io_println() {
        let _lock = Lock::new();

        clear(); io_println!( "" ); assert!( io::try_recv().is_some() );
        clear(); io_println!( "" ); assert!( io::try_recv_line().is_some() );
        clear(); io_println!( "" ); assert!( io::try_recv_err().is_none() );
        clear(); io_println!( "" ); assert!( io::try_recv_err_line().is_none() );
    }

    #[test]
    fn io_eprint() {
        let _lock = Lock::new();

        clear(); io_eprint!( "" ); assert!( io::try_recv().is_none() );
        clear(); io_eprint!( "" ); assert!( io::try_recv_line().is_none() );
        clear(); io_eprint!( "" ); assert!( io::try_recv_err().is_none() );
        clear(); io_eprint!( "" ); assert!( io::try_recv_err_line().is_none() );

        clear(); io_eprint!( " " ); assert!( io::try_recv().is_none() );
        clear(); io_eprint!( " " ); assert!( io::try_recv_line().is_none() );
        clear(); io_eprint!( " " ); assert!( io::try_recv_err().is_some() );
        clear(); io_eprint!( " " ); assert!( io::try_recv_err_line().is_none() );

        clear(); io_eprint!( "\n" ); assert!( io::try_recv().is_none() );
        clear(); io_eprint!( "\n" ); assert!( io::try_recv_line().is_none() );
        clear(); io_eprint!( "\n" ); assert!( io::try_recv_err().is_some() );
        clear(); io_eprint!( "\n" ); assert!( io::try_recv_err_line().is_some() );
    }

    #[test]
    fn io_eprintln() {
        let _lock = Lock::new();

        clear(); io_eprintln!( "" ); assert!( io::try_recv().is_none() );
        clear(); io_eprintln!( "" ); assert!( io::try_recv_line().is_none() );
        clear(); io_eprintln!( "" ); assert!( io::try_recv_err().is_some() );
        clear(); io_eprintln!( "" ); assert!( io::try_recv_err_line().is_some() );
    }

    #[test]
    fn receive() {
        let _lock = Lock::new();

        io_print!("");
        assert!( io::try_recv().is_none() );

        io_print!(" ");
        assert!( io::try_recv_err().is_none() );
        assert_eq!( io::try_recv(), Some( " ".to_owned() ) );

        io_print!( "abcdefg\nhijklmn\nopq rst\nuvw xyz" );
        assert_eq!( io::try_recv_line(), Some( "abcdefg\n".to_owned() ) );
        assert_eq!( io::recv_line(), "hijklmn\n" );
        assert_eq!( io::recv(), "opq rst\nuvw xyz" );
    }

    #[test]
    fn receive_err() {
        let _lock = Lock::new();

        io_eprint!("");
        assert!( io::try_recv_err().is_none() );

        io_eprint!(" ");
        assert!( io::try_recv().is_none() );
        assert_eq!( io::try_recv_err(), Some( " ".to_owned() ) );

        io_eprint!( "abcdefg\nhijklmn\nopq rst\nuvw xyz" );
        assert_eq!( io::try_recv_err_line(), Some( "abcdefg\n".to_owned() ) );
        assert_eq!( io::recv_err_line(), "hijklmn\n" );
        assert_eq!( io::recv_err(), "opq rst\nuvw xyz" );
    }
}
