/// A message dispatched by a Server-Sent Events stream.
///
/// `last_event_id` is the stream's current last-event ID when this event is
/// dispatched. It can therefore be inherited from an earlier `id` field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    /// The event payload, with consecutive `data` fields joined by `\n`.
    pub data: String,
    /// The event type, or `"message"` when no non-empty `event` field was set.
    pub event_type: String,
    /// The stream's last-event ID at dispatch time.
    pub last_event_id: String,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            data: String::new(),
            event_type: "message".to_owned(),
            last_event_id: String::new(),
        }
    }
}
