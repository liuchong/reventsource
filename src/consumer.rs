use crate::event::{Event, EventState};
use crate::parser::parse_line;
use std::error::Error;
use std::io::{BufRead, BufReader, Lines, Read};
use std::iter::Enumerate;

pub struct Consumer<T>
where
    T: Read,
{
    response: T,
}

impl<'a, T> Consumer<T>
where
    T: Read,
{
    pub fn new(response: T) -> Self {
        Consumer { response }
    }

    pub fn enumerate(&mut self) -> Enumerate<Lines<BufReader<&mut T>>>
    where
        T: Read,
    {
        let response = self.response.by_ref();
        let reader = BufReader::new(response);

        reader.lines().enumerate()
    }
}

impl<T> Iterator for Consumer<T>
where
    T: Read,
{
    type Item = Result<Event, Box<Error>>;

    fn next(&mut self) -> Option<Result<Event, Box<Error>>> {
        let mut event = Event::new();

        for (_num, line) in self.enumerate() {
            match parse_line(&line.unwrap(), &mut event) {
                EventState::Pending => {}
                EventState::Ready => {
                    return Some(Ok(event));
                }
            }
        }

        Some(Ok(event))
    }
}
