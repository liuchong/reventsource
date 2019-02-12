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
        if self.event.is_some() {
            writeln!(f, "event: {}", self.event.as_ref().unwrap())?;
        }
        if self.id.is_some() {
            writeln!(f, "id: {}", self.id.as_ref().unwrap())?;
        }
        if self.retry.is_some() {
            writeln!(f, "retry: {}", self.retry.as_ref().unwrap())?;
        }
        write!(f, "data: {}", &self.data)
    }
}
