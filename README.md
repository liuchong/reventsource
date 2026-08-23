# reventsource

[![CI](https://github.com/liuchong/reventsource/actions/workflows/ci.yml/badge.svg)](https://github.com/liuchong/reventsource/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/reventsource.svg)](https://crates.io/crates/reventsource)
[![docs.rs](https://docs.rs/reventsource/badge.svg)](https://docs.rs/reventsource)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://www.rust-lang.org)
[![license](https://img.shields.io/crates/l/reventsource.svg)](LICENSE-MIT)

`reventsource` is a transport-agnostic
[Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html)
parser for Rust. It has no runtime or development dependencies.

## Usage

Use `Parser` when a transport supplies arbitrary byte chunks:

```rust
use reventsource::Parser;

let mut parser = Parser::new();
let mut events = parser.push(b"id: 42\ndata: hello");
events.extend(parser.push(b"\ndata: world\n\n"));
events.extend(parser.finish());

assert_eq!(events[0].event_type, "message");
assert_eq!(events[0].data, "hello\nworld");
assert_eq!(events[0].last_event_id, "42");
assert_eq!(parser.last_event_id(), "42");
```

For synchronous sources, `Consumer<T: Read>` is an iterator:

```rust
use std::io::{self, Cursor};

let source = Cursor::new(b"event: update\ndata: ready\n\n");
for event in reventsource::new(source) {
    let event = event?;
    println!("{}: {}", event.event_type, event.data);
}
# Ok::<(), io::Error>(())
```

The parser accepts UTF-8 split across chunks, all EventSource line endings,
comments, persistent last-event IDs, and valid decimal `retry` values. Invalid
UTF-8 is replaced according to Rust's lossy UTF-8 decoding behavior. At EOF,
an event without a terminating blank line is not dispatched.
Call `Parser::restart` before feeding a reconnected response; it discards any
incomplete event while preserving the last-event ID and reconnection delay.

See the dependency-free [`cursor` example](examples/cursor.rs) for a complete
program.

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT License](LICENSE-MIT), at your option.
