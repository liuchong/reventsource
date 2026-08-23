fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
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
