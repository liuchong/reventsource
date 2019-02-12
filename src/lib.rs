use std::io::Read;

mod consumer;
mod event;
mod parser;

pub use consumer::Consumer;

pub fn new<T>(response: T) -> Consumer<T>
where
    T: Read,
{
    Consumer::new(response)
}
