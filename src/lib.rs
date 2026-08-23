//! A transport-agnostic parser for
//! [Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html).
//!
//! Use [`Parser`] when bytes arrive asynchronously or in arbitrary chunks.
//! [`Consumer`] adapts any synchronous [`std::io::Read`] implementation into
//! an iterator of parsed [`Event`] values.

use std::io::Read;

mod consumer;
mod event;
mod parser;

pub use consumer::Consumer;
pub use event::Event;
pub use parser::Parser;

/// Creates a synchronous EventSource consumer over `reader`.
#[must_use]
pub fn new<T>(response: T) -> Consumer<T>
where
    T: Read,
{
    Consumer::new(response)
}
