mod metadata;

fn main() -> Result<(), &'static str> {
    let mut args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() > 1,
        "Must provide at least one argument; the album name."
    );
    args.remove(0);
    let album_name = args.remove(0);
    let listings_to_use: Vec<usize> = args
        .into_iter()
        .map(|s| {
            s.parse::<usize>()
                .expect("A track listing number argument couldn't be parsed as a usize.")
        })
        .collect();

    let metadata = metadata::scrape_wikipedia(&album_name, &listings_to_use)?;
    println!("{}", metadata);

    Ok(())
}
