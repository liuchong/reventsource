use std::mem;
use std::str;
use std::time::Duration;

use crate::Event;

/// An incremental, transport-agnostic Server-Sent Events parser.
///
/// Feed arbitrary byte chunks with [`Parser::push`] and call
/// [`Parser::finish`] when the stream reaches EOF. UTF-8 code points and line
/// endings may be split across chunks.
#[derive(Clone, Debug)]
pub struct Parser {
    utf8_pending: Vec<u8>,
    line: String,
    skip_lf: bool,
    at_stream_start: bool,
    data_lines: Vec<String>,
    event_type: String,
    last_event_id: String,
    reconnection_time: Option<Duration>,
    finished: bool,
}

impl Parser {
    /// Creates a parser with empty event-stream state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            utf8_pending: Vec::new(),
            line: String::new(),
            skip_lf: false,
            at_stream_start: true,
            data_lines: Vec::new(),
            event_type: String::new(),
            last_event_id: String::new(),
            reconnection_time: None,
            finished: false,
        }
    }

    /// Parses a byte chunk and returns every event dispatched by it.
    ///
    /// Calls made after [`Parser::finish`] return no events.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<Event> {
        if self.finished {
            return Vec::new();
        }

        self.decode(chunk, false)
    }

    /// Finishes the stream and returns any events completed while decoding it.
    ///
    /// Per the EventSource processing model, EOF does not dispatch an event
    /// that lacks a terminating blank line.
    pub fn finish(&mut self) -> Vec<Event> {
        if self.finished {
            return Vec::new();
        }

        let events = self.decode(&[], true);
        if !self.line.is_empty() {
            self.process_line(&mut Vec::new());
        }
        self.data_lines.clear();
        self.event_type.clear();
        self.skip_lf = false;
        self.finished = true;
        events
    }

    /// Returns the current last-event ID, including an explicitly reset ID.
    #[must_use]
    pub fn last_event_id(&self) -> &str {
        &self.last_event_id
    }

    /// Returns the most recent valid reconnection delay from a `retry` field.
    #[must_use]
    pub fn reconnection_time(&self) -> Option<Duration> {
        self.reconnection_time
    }

    /// Returns whether [`Parser::finish`] has been called.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Starts parsing a new response while preserving reconnection state.
    ///
    /// Any incomplete event from the previous response is discarded. The
    /// current last-event ID and reconnection delay are retained so a
    /// transport can reconnect according to the EventSource processing model.
    pub fn restart(&mut self) {
        self.utf8_pending.clear();
        self.line.clear();
        self.skip_lf = false;
        self.at_stream_start = true;
        self.data_lines.clear();
        self.event_type.clear();
        self.finished = false;
    }

    fn decode(&mut self, chunk: &[u8], eof: bool) -> Vec<Event> {
        let mut bytes = mem::take(&mut self.utf8_pending);
        bytes.extend_from_slice(chunk);

        let decoded_end = if eof {
            bytes.len()
        } else {
            incomplete_utf8_start(&bytes).unwrap_or(bytes.len())
        };
        let decoded = String::from_utf8_lossy(&bytes[..decoded_end]);
        let mut events = Vec::new();
        self.process_decoded(&decoded, &mut events);
        self.utf8_pending.extend_from_slice(&bytes[decoded_end..]);
        events
    }

    fn process_decoded(&mut self, decoded: &str, events: &mut Vec<Event>) {
        for character in decoded.chars() {
            if self.at_stream_start {
                self.at_stream_start = false;
                if character == '\u{feff}' {
                    continue;
                }
            }

            if self.skip_lf {
                self.skip_lf = false;
                if character == '\n' {
                    continue;
                }
            }

            match character {
                '\r' => {
                    self.process_line(events);
                    self.skip_lf = true;
                }
                '\n' => self.process_line(events),
                _ => self.line.push(character),
            }
        }
    }

    fn process_line(&mut self, events: &mut Vec<Event>) {
        let line = mem::take(&mut self.line);
        if line.is_empty() {
            self.dispatch(events);
            return;
        }
        if line.starts_with(':') {
            return;
        }

        let (field, value) = line
            .split_once(':')
            .map_or((line.as_str(), ""), |(field, value)| {
                (field, value.strip_prefix(' ').unwrap_or(value))
            });

        match field {
            "data" => self.data_lines.push(value.to_owned()),
            "event" => self.event_type = value.to_owned(),
            "id" if !value.contains('\0') => self.last_event_id = value.to_owned(),
            "retry" if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
                if let Ok(milliseconds) = value.parse::<u64>() {
                    self.reconnection_time = Some(Duration::from_millis(milliseconds));
                }
            }
            _ => {}
        }
    }

    fn dispatch(&mut self, events: &mut Vec<Event>) {
        if self.data_lines.is_empty() {
            return;
        }

        let data = self.data_lines.join("\n");
        self.data_lines.clear();
        let event_type = if self.event_type.is_empty() {
            "message".to_owned()
        } else {
            mem::take(&mut self.event_type)
        };
        self.event_type.clear();

        events.push(Event {
            data,
            event_type,
            last_event_id: self.last_event_id.clone(),
        });
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

fn incomplete_utf8_start(bytes: &[u8]) -> Option<usize> {
    for start in bytes.len().saturating_sub(3)..bytes.len() {
        let suffix = &bytes[start..];
        if let Err(error) = str::from_utf8(suffix) {
            if error.valid_up_to() == 0 && error.error_len().is_none() {
                return Some(start);
            }
        }
    }
    None
}
