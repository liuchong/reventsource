use std::default::Default;
use std::fmt::{Display, Formatter, Result};

pub enum EventState {
    Pending,
    Ready,
}

#[derive(Default)]
pub struct Event {
    pub data: String,
    pub event: Option<String>,
    pub id: Option<String>,
    pub retry: Option<String>,
}

impl Event {
    pub fn new() -> Event {
        Default::default()
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if let Some(event) = &self.event {
            writeln!(f, "event: {}", event)?;
        }
        if let Some(id) = &self.id {
            writeln!(f, "id: {}", id)?;
        }
        if let Some(retry) = &self.retry {
            writeln!(f, "retry: {}", retry)?;
        }
        write!(f, "data: {}", self.data)
    }
}
