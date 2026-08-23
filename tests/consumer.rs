use std::io::{self, Cursor, Read};
use std::time::Duration;

use reventsource::{Consumer, Event, Parser};

fn event(data: &str, event_type: &str, last_event_id: &str) -> Event {
    Event {
        data: data.to_owned(),
        event_type: event_type.to_owned(),
        last_event_id: last_event_id.to_owned(),
    }
}

fn parse_chunks(chunks: &[&[u8]]) -> (Vec<Event>, Parser) {
    let mut parser = Parser::new();
    let mut events = Vec::new();
    for chunk in chunks {
        events.extend(parser.push(chunk));
    }
    events.extend(parser.finish());
    (events, parser)
}

#[test]
fn parses_identically_at_every_single_chunk_boundary() {
    let input = b"\xef\xbb\xbfevent: greeting\r\nid: 42\r\nretry: 1500\r\ndata: hello \xe2\x98\x83\r\ndata: world\r\n\r\ndata: next\r\n\r\n";
    let expected = vec![
        event("hello \u{2603}\nworld", "greeting", "42"),
        event("next", "message", "42"),
    ];

    for split in 0..=input.len() {
        let (events, parser) = parse_chunks(&[&input[..split], &input[split..]]);
        assert_eq!(events, expected, "split at byte {split}");
        assert_eq!(parser.last_event_id(), "42");
        assert_eq!(
            parser.reconnection_time(),
            Some(Duration::from_millis(1500))
        );
    }

    let chunks: Vec<&[u8]> = input.chunks(1).collect();
    assert_eq!(parse_chunks(&chunks).0, expected);
}

#[test]
fn supports_lf_crlf_and_bare_cr_lines() {
    for ending in ["\n", "\r\n", "\r"] {
        let input = format!("event: update{ending}data: first{ending}data: second{ending}{ending}");
        assert_eq!(
            parse_chunks(&[input.as_bytes()]).0,
            vec![event("first\nsecond", "update", "")]
        );
    }
}

#[test]
fn handles_crlf_split_between_chunks() {
    let (events, _) = parse_chunks(&[b"data: one\r", b"\ndata: two\r", b"\n\r", b"\n"]);
    assert_eq!(events, vec![event("one\ntwo", "message", "")]);
}

#[test]
fn replaces_invalid_utf8_independently_of_chunks() {
    let input = b"data: before \xf0\x28\x8c\x28 after\n\n";
    let expected = vec![event("before \u{fffd}(\u{fffd}( after", "message", "")];

    for split in 0..=input.len() {
        assert_eq!(
            parse_chunks(&[&input[..split], &input[split..]]).0,
            expected,
            "split at byte {split}"
        );
    }
}

#[test]
fn strips_only_an_initial_bom() {
    let input = b"\xef\xbb\xbfdata: first\n\ndata: \xef\xbb\xbfsecond\n\n";
    assert_eq!(
        parse_chunks(&input.chunks(1).collect::<Vec<_>>()).0,
        vec![
            event("first", "message", ""),
            event("\u{feff}second", "message", ""),
        ]
    );
}

#[test]
fn handles_comments_unknown_fields_and_one_optional_space() {
    let input = b": comment\nunknown\nother: value\ndata:  one\ndata:\ttwo\n\n";
    assert_eq!(
        parse_chunks(&[input]).0,
        vec![event(" one\n\ttwo", "message", "")]
    );
}

#[test]
fn field_without_colon_has_an_empty_value() {
    let input = b"id: previous\nid\ndata\n\n";
    assert_eq!(parse_chunks(&[input]).0, vec![event("", "message", "")]);
}

#[test]
fn id_persists_resets_and_ignores_nul() {
    let input = b"id: one\ndata: a\n\nid: bad\0id\ndata: b\n\nid\ndata: c\n\ndata: d\n\n";
    let expected = vec![
        event("a", "message", "one"),
        event("b", "message", "one"),
        event("c", "message", ""),
        event("d", "message", ""),
    ];
    let (events, parser) = parse_chunks(&[input]);
    assert_eq!(events, expected);
    assert_eq!(parser.last_event_id(), "");
}

#[test]
fn event_type_defaults_and_resets_only_after_dispatch() {
    let input = b"event: retained\n\n:data absent\ndata: one\n\ndata: two\n\n";
    assert_eq!(
        parse_chunks(&[input]).0,
        vec![event("one", "retained", ""), event("two", "message", ""),]
    );
}

#[test]
fn retry_accepts_only_nonempty_ascii_decimal_u64() {
    let mut parser = Parser::new();
    parser.push(b"retry: 2500\n");
    assert_eq!(
        parser.reconnection_time(),
        Some(Duration::from_millis(2500))
    );

    for invalid in [
        b"retry:\n".as_slice(),
        b"retry: -1\n",
        b"retry: +1\n",
        b"retry: 1.5\n",
        "retry: １２\n".as_bytes(),
        b"retry: 18446744073709551616\n",
    ] {
        parser.push(invalid);
        assert_eq!(
            parser.reconnection_time(),
            Some(Duration::from_millis(2500))
        );
    }

    parser.push(b"retry: 0\n");
    assert_eq!(parser.reconnection_time(), Some(Duration::ZERO));
}

#[test]
fn blank_lines_dispatch_only_when_data_was_seen() {
    let input = b"\n\n: comment\n\nunknown: x\n\nretry: 5\n\nid: x\n\n\ndata: ok\n\n\n\n";
    let (events, parser) = parse_chunks(&[input]);
    assert_eq!(events, vec![event("ok", "message", "x")]);
    assert_eq!(parser.reconnection_time(), Some(Duration::from_millis(5)));
}

#[test]
fn eof_discards_an_unterminated_event_but_keeps_stream_state() {
    let mut parser = Parser::new();
    assert!(
        parser
            .push(b"id: final\nretry: 9\ndata: incomplete")
            .is_empty()
    );
    assert!(parser.finish().is_empty());
    assert!(parser.is_finished());
    assert_eq!(parser.last_event_id(), "final");
    assert_eq!(parser.reconnection_time(), Some(Duration::from_millis(9)));
    assert!(parser.push(b"\n\n").is_empty());
    assert!(parser.finish().is_empty());
}

#[test]
fn restart_preserves_reconnection_state_and_accepts_a_new_stream() {
    let mut parser = Parser::new();
    assert_eq!(
        parser.push(b"id: first\nretry: 25\ndata: one\n\n"),
        vec![event("one", "message", "first")]
    );
    assert!(parser.finish().is_empty());

    parser.restart();

    assert!(!parser.is_finished());
    assert_eq!(parser.last_event_id(), "first");
    assert_eq!(parser.reconnection_time(), Some(Duration::from_millis(25)));
    assert_eq!(
        parser.push(b"\xef\xbb\xbfdata: two\n\n"),
        vec![event("two", "message", "first")]
    );
}

#[test]
fn consumer_reads_incrementally_and_exposes_state() {
    let reader = ChunkedReader::new(b"id: 7\nretry: 12\ndata: hello\n\n", 2);
    let mut consumer = Consumer::new(reader);
    assert_eq!(
        consumer.next().transpose().unwrap(),
        Some(event("hello", "message", "7"))
    );
    assert_eq!(consumer.last_event_id(), "7");
    assert_eq!(
        consumer.reconnection_time(),
        Some(Duration::from_millis(12))
    );
    assert!(consumer.next().is_none());
}

#[test]
fn consumer_propagates_io_errors_once() {
    let mut consumer = Consumer::new(FailingReader { first_read: true });
    assert_eq!(
        consumer.next().transpose().unwrap(),
        Some(event("before error", "message", ""))
    );
    let error = consumer.next().expect("expected an I/O error").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert!(consumer.next().is_none());
}

#[test]
fn cursor_consumer_is_dependency_free() {
    let input = Cursor::new(b"data: cursor\n\n");
    let events = reventsource::new(input)
        .collect::<io::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(events, vec![event("cursor", "message", "")]);
}

struct ChunkedReader {
    inner: Cursor<&'static [u8]>,
    chunk_size: usize,
}

impl ChunkedReader {
    fn new(bytes: &'static [u8], chunk_size: usize) -> Self {
        Self {
            inner: Cursor::new(bytes),
            chunk_size,
        }
    }
}

impl Read for ChunkedReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let limit = buffer.len().min(self.chunk_size);
        self.inner.read(&mut buffer[..limit])
    }
}

struct FailingReader {
    first_read: bool,
}

impl Read for FailingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.first_read {
            self.first_read = false;
            let payload = b"data: before error\n\n";
            buffer[..payload.len()].copy_from_slice(payload);
            Ok(payload.len())
        } else {
            Err(io::Error::other("synthetic failure"))
        }
    }
}
