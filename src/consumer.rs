use crate::event::{Event, EventState};
use crate::parser::parse_line;
use std::error::Error;
use std::io::{BufRead, BufReader, Lines, Read};
use std::iter::Enumerate;

pub struct Consumer<T>
where
    T: Read,
{
    response: Enumerate<Lines<BufReader<T>>>
}

impl<'a, T> Consumer<T>
where
    T: Read,
{
    pub fn new(response: T) -> Self {
        Consumer { response: BufReader::new(response).lines().enumerate() }
    }
}

impl<T> Iterator for Consumer<T>
where
    T: Read,
{
    type Item = Result<Event, Box<dyn Error>>;

    fn next(&mut self) -> Option<Result<Event, Box<dyn Error>>> {
        let mut event = Event::new();

        for (_num, line_res) in self.response.by_ref() {
            let line = line_res.unwrap();
            match parse_line(&line, &mut event) {
                EventState::Pending => {}
                EventState::Ready => {
                    return Some(Ok(event));
                }
            }
        }

        Some(Ok(event))
    }
}
