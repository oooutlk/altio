#[::altio::define] pub mod io {}

pub mod foo {
    pub mod bar {
        pub fn baz() {
            use crate::io;

            alt_print!("What's your name?");
            assert_eq!( io::receive(), "What's your name?" );
            alt_println!("Li Lei.");
            assert_eq!( io::receive(), "Li Lei.\n" );

            io::send_line("Listen, read and say.".into());
            assert_eq!( io::receive(), "[read_line] Listen, read and say.\n" );
        }
    }
}

fn main() {
    alt_print!("Good morning, class!");
    assert_eq!( io::receive(), "Good morning, class!" );
    alt_println!("Good morning, teacher!");
    assert_eq!( io::receive(), "Good morning, teacher!\n" );

    std::thread::spawn( || loop {
        alt_print!( "[read_line] {}", io::read_line().unwrap() );
    });

    io::send_line("My name is Gao Hui.".into());
    assert_eq!( io::receive(), "[read_line] My name is Gao Hui.\n" );

    foo::bar::baz();
}
