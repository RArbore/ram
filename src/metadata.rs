extern crate wikipedia;
use scraper::Html;

pub struct AlbumInfo {
    name: String,
    track_names: Vec<String>,
    cover: Option<image::DynamicImage>,
}

impl std::fmt::Display for AlbumInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Album Name: {}\n\n", self.name)?;
        write!(f, "Track Listing:\n")?;
        for (idx, name) in self.track_names.iter().enumerate() {
            write!(f, "{}: {}\n", idx, name)?;
        }
        if let Some(cover) = &self.cover {
            write!(
                f,
                "\nCover Dimensions: {} x {}\n",
                cover.width(),
                cover.height()
            )?;
        }
        Ok(())
    }
}

pub fn scrape_wikipedia(album_name: &str, track_list_nums: &[usize]) -> Result<AlbumInfo, String> {
    let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
    let page = wiki.page_from_title(album_name.to_string());
    let content = page
        .get_html_content()
        .expect(&("Couldn't find wikipedia page about \"".to_owned() + &album_name + "\"."));
    let document = Html::parse_document(&content);
    let mut track_listings: Vec<Vec<String>> = vec![];
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

    let mut tracklist = vec![];
    for listing in track_list_nums {
        for track in track_listings[*listing].iter() {
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
        let cover = reqwest::blocking::get(cover_url.clone())
            .unwrap()
            .bytes()
            .unwrap();
        let cover = image::load_from_memory(&cover)
            .ok()
            .ok_or(format!("Couldn't load image from cover at: {}", cover_url))?;
        cover_image = Some(cover);
    }

    Ok(AlbumInfo {
        name: album_name.to_string(),
        track_names: tracklist,
        cover: cover_image,
    })
}
