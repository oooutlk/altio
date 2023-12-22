#[::altio::define] pub mod io {}

fn main() {
    _print!("Good morning, class!");
    assert_eq!( io::recv(), "Good morning, class!" );
    _println!("Good morning, teacher!");
    assert_eq!( io::recv(), "Good morning, teacher!\n" );

    std::thread::spawn( || {
        let mut line = String::new();
        loop {
            io::read_line( &mut line ).unwrap();
            _print!( "[read_line] {line}",  );
            line.clear();
        }
    });

    io::sendln("My name is Gao Hui.");
    assert_eq!( io::recv(), "[read_line] My name is Gao Hui.\n" );

    foo::bar::baz();
}

pub mod foo {
    pub mod bar {
        pub fn baz() {
            use crate::io;

            _eprint!("What's your name?");
            assert_eq!( io::recv_err(), "What's your name?" );
            _eprintln!("Li Lei.");
            assert_eq!( io::recv_err(), "Li Lei.\n" );

            io::sendln("Listen, read and say.");
            assert_eq!( io::recv(), "[read_line] Listen, read and say.\n" );
        }
    }
}
