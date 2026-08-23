use crate::event::{Event, EventState};
use crate::parser::parse_line;
use std::error::Error;
use std::io::{BufRead, BufReader, Lines, Read};

pub struct Consumer<T>
where
    T: Read,
{
    response: Lines<BufReader<T>>,
}

impl<T> Consumer<T>
where
    T: Read,
{
    pub fn new(response: T) -> Self {
        Consumer { response: BufReader::new(response).lines() }
    }
}

impl<T> Iterator for Consumer<T>
where
    T: Read,
{
    type Item = Result<Event, Box<dyn Error>>;

    fn next(&mut self) -> Option<Result<Event, Box<dyn Error>>> {
        let mut event = Event::new();
        let mut saw_line = false;

        for line_res in self.response.by_ref() {
            let line = match line_res {
                Ok(line) => line,
                Err(e) => return Some(Err(e.into())),
            };
            saw_line = true;
            match parse_line(&line, &mut event) {
                EventState::Pending => {}
                EventState::Ready => {
                    return Some(Ok(event));
                }
            }
        }

        // The underlying stream is exhausted: emit a final partial event if
        // any lines were consumed in this call, otherwise end the iterator.
        if saw_line {
            Some(Ok(event))
        } else {
            None
        }
    }
}
