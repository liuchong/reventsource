use crate::event::{Event, EventState};

pub fn parse_line(line: &str, event: &mut Event) -> EventState {
    // data ends with `\n\n`
    if line == "" {
        return EventState::Ready;
    }

    let kv: Vec<&str> = line.splitn(2, ':').collect();
    if kv.len() < 2 {
        return EventState::Pending;
    }
    let k = kv[0];
    let mut v = kv[1];

    // clean up one optional space
    if v.starts_with(' ') {
        v = &v[1..]
    };

    match k {
        "data" => {
            event.data.push_str(v);
            event.data.push('\n');
        }
        "event" => {
            event.event = Some(v.to_string());
        }

        "id" => {
            event.id = Some(v.to_string());
        }
        "retry" => {
            event.retry = Some(v.to_string());
        }
        _ => {}
    }

    EventState::Pending
}
