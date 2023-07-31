extern crate wikipedia;
use scraper::Html;

fn main() {
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

    let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
    let page = wiki.page_from_title(album_name.clone());
    let content = page
        .get_html_content()
        .expect(&("Couldn't find wikipedia page about \"".to_owned() + &album_name + "\"."));
    let document = Html::parse_document(&content);
    let mut track_listings = vec![];
    let mut author = None;
    let mut album_cover = None;

    for node in document.tree.nodes() {
        let value = node.value();
        if value.is_element()
            && value.as_element().unwrap().has_class(
                "contributor",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            )
            && author == None
        {
            author = Some(
                node.first_child()
                    .unwrap()
                    .value()
                    .as_element()
                    .unwrap()
                    .attr("title")
                    .unwrap()
                    .replace(" (band)", ""),
            );
        } else if value.is_element()
            && value.as_element().unwrap().has_class(
                "infobox-image",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            )
            && album_cover == None
        {
            for node in node.descendants() {
                let value = node.value();
                if value.is_element()
                    && value.as_element().unwrap().has_class(
                        "mw-file-description",
                        scraper::CaseSensitivity::AsciiCaseInsensitive,
                    )
                {
                    album_cover = Some(value.as_element().unwrap().attr("href").unwrap());
                }
            }
        } else if value.is_element()
            && value.as_element().unwrap().has_class(
                "track-listing",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            )
        {
            let mut tracks = vec![];
            'tr: for node in node.descendants() {
                let value = node.value();
                if value.is_element() && value.as_element().unwrap().name() == "tr" {
                    for node in node.descendants() {
                        let value = node.value();
                        if value.is_element() && value.as_element().unwrap().name() == "td" {
                            for node in node.descendants() {
                                if node.value().is_text() {
                                    let mut base =
                                        String::from(node.value().as_text().unwrap() as &str);
                                    for node in node.next_siblings() {
                                        base += match node.value() {
                                            scraper::node::Node::Element(elem) => {
                                                elem.attr("title").unwrap()
                                            }
                                            scraper::node::Node::Text(text) => text as &str,
                                            _ => "",
                                        };
                                    }
                                    tracks.push(base);
                                    continue 'tr;
                                }
                            }
                        }
                    }
                }
            }
            tracks.pop().unwrap();
            track_listings.push(tracks);
        }
    }

    println!(
        "Here are the tracks found for the album \"{}\", authored by \"{}\":\n[",
        page.get_title().unwrap(),
        author.clone().unwrap_or(String::from("Unknown"))
    );
    for tracks in track_listings.iter() {
        println!("  [");
        for track in tracks {
            println!("    {}", track);
        }
        println!("  ]");
    }
    if listings_to_use.len() == 0 && track_listings.len() > 1 {
        println!("]\nSince there is more than one track listing, an explicit specification of which listings should be included in the download is required. Please specify at least one item from the list to download. List the listing number of each listing to include, in order (zero indexed).");
        return;
    }

    let mut tracklist = vec![];
    for listing in listings_to_use {
        for track in track_listings[listing].iter() {
            let mut trackname = String::from(track.split("\"").nth(1).unwrap());
            let mut remove_patterns = vec![String::from(" (song)")];
            if let Some(author) = &author {
                remove_patterns.push(String::from(" (") + author + " song)");
                remove_patterns.push(String::from(" (") + author + " EP)");
            }
            for pattern in remove_patterns {
                trackname = trackname.replace(&pattern, "");
            }
            tracklist.push(trackname);
        }
    }

    let mut cover_url = None;
    if let Some(album_cover) = album_cover {
        let cover_name = album_cover.split("File:").nth(1).unwrap();
        for image in page.get_images().unwrap() {
            if image.url.contains(cover_name) {
                cover_url = Some(image.url);
            }
        }
    }
    let mut cover_image = None;
    if let Some(cover_url) = cover_url {
        println!("Here's the URL to the album cover: {}", cover_url);
        let cover = reqwest::blocking::get(cover_url).unwrap().bytes().unwrap();
        let cover = image::load_from_memory(&cover).unwrap();
        cover_image = Some(cover);
    }

    for track in tracklist {
        println!("{}", track);
    }
}
