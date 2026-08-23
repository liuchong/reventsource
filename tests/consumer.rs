use reventsource::Event;

#[test]
fn parses_events_and_terminates() {
    let input: &[u8] =
        b"event: greeting\nid: 1\nretry: 3000\ndata: hello\ndata: world\n\ndata: second\n\n";
    let mut c = reventsource::new(input);

    let e1: Event = c.next().unwrap().unwrap();
    assert_eq!(e1.event.as_deref(), Some("greeting"));
    assert_eq!(e1.id.as_deref(), Some("1"));
    assert_eq!(e1.retry.as_deref(), Some("3000"));
    assert_eq!(e1.data, "hello\nworld\n");

    let e2 = c.next().unwrap().unwrap();
    assert_eq!(e2.data, "second\n");
    assert_eq!(e2.event, None);

    assert!(c.next().is_none());
}

#[test]
fn emits_trailing_partial_event() {
    let input: &[u8] = b"data: unterminated\n";
    let mut c = reventsource::new(input);

    let e = c.next().unwrap().unwrap();
    assert_eq!(e.data, "unterminated\n");

    assert!(c.next().is_none());
}

#[test]
fn empty_stream_yields_nothing() {
    let input: &[u8] = b"";
    let mut c = reventsource::new(input);

    assert!(c.next().is_none());
}

#[test]
fn ignores_comment_like_and_unknown_lines() {
    let input: &[u8] = b": comment\nfoo: bar\nignored-line\ndata: ok\n\n";
    let mut c = reventsource::new(input);

    let e = c.next().unwrap().unwrap();
    assert_eq!(e.data, "ok\n");

    assert!(c.next().is_none());
}

#[test]
fn display_formats_event() {
    let input: &[u8] = b"event: e\nid: 7\ndata: x\n\n";
    let mut c = reventsource::new(input);

    let e = c.next().unwrap().unwrap();
    assert_eq!(e.to_string(), "event: e\nid: 7\ndata: x\n");
}
