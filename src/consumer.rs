use std::collections::VecDeque;
use std::io::{self, Read};
use std::time::Duration;

use crate::{Event, Parser};

/// A synchronous [`Read`] adapter for an EventSource byte stream.
#[derive(Debug)]
pub struct Consumer<T>
where
    T: Read,
{
    reader: T,
    parser: Parser,
    pending: VecDeque<Event>,
    buffer: [u8; 8 * 1024],
    finished: bool,
}

impl<T> Consumer<T>
where
    T: Read,
{
    /// Creates a consumer that reads and parses `reader` on demand.
    pub fn new(response: T) -> Self {
        Self {
            reader: response,
            parser: Parser::new(),
            pending: VecDeque::new(),
            buffer: [0; 8 * 1024],
            finished: false,
        }
    }

    /// Returns the current last-event ID.
    #[must_use]
    pub fn last_event_id(&self) -> &str {
        self.parser.last_event_id()
    }

    /// Returns the most recent valid reconnection delay.
    #[must_use]
    pub fn reconnection_time(&self) -> Option<Duration> {
        self.parser.reconnection_time()
    }

    /// Returns the underlying incremental parser.
    #[must_use]
    pub fn parser(&self) -> &Parser {
        &self.parser
    }

    /// Returns a shared reference to the underlying reader.
    #[must_use]
    pub fn get_ref(&self) -> &T {
        &self.reader
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.reader
    }

    /// Consumes this adapter and returns the underlying reader.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.reader
    }
}

impl<T> Iterator for Consumer<T>
where
    T: Read,
{
    type Item = io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pending.pop_front() {
            return Some(Ok(event));
        }
        if self.finished {
            return None;
        }

        loop {
            match self.reader.read(&mut self.buffer) {
                Ok(0) => {
                    self.pending.extend(self.parser.finish());
                    self.finished = true;
                    return self.pending.pop_front().map(Ok);
                }
                Ok(read) => {
                    self.pending.extend(self.parser.push(&self.buffer[..read]));
                    if let Some(event) = self.pending.pop_front() {
                        return Some(Ok(event));
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    self.finished = true;
                    return Some(Err(error));
                }
            }
        }
    }
}
