extern crate reqwest;
extern crate reventsource;

fn main() -> Result<(), Box<std::error::Error>> {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https("http://127.0.0.1:7777")?)
        .timeout(None)
        .build()?;

    let resp = client
        .get("https://hacker-news.firebaseio.com/v0/updates.json?print=pretty")
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .send()?;

    println!("{:#?}", resp);

    let c = reventsource::new(resp);
    for result in c {
        let event = result?;
        println!("Event:\n{}", event);
    }

    Ok(())
}
