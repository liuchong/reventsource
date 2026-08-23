use std::io::{self, Cursor};

fn main() -> io::Result<()> {
    let stream = Cursor::new(
        b"id: 7\n\
          event: greeting\n\
          data: hello\n\
          data: world\n\
          \n"
        .as_slice(),
    );

    for event in reventsource::new(stream) {
        println!("{:#?}", event?);
    }

    Ok(())
}
