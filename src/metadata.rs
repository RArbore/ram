extern crate wikipedia;

use id3::TagLike;
use scraper::Html;

pub struct AlbumInfo {
    name: String,
    artist: String,
    track_names: Vec<String>,
    cover: Option<image::DynamicImage>,
}

impl std::fmt::Display for AlbumInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Album Name: {}\n\n", self.name)?;
        write!(f, "Artist: {}\n\n", self.name)?;
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
    let mut artist = None;
    let mut album_cover = None;

    for node in document.tree.nodes() {
        let value = node.value();
        if value.is_element()
            && value.as_element().unwrap().has_class(
                "contributor",
                scraper::CaseSensitivity::AsciiCaseInsensitive,
            )
            && artist == None
        {
            artist = Some(
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
            if let Some(artist) = &artist {
                remove_patterns.push(String::from(" (") + artist + " song)");
                remove_patterns.push(String::from(" (") + artist + " EP)");
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
        artist: artist.unwrap_or(String::from("Unknown")),
        track_names: tracklist,
        cover: cover_image,
    })
}

pub fn update_album_metadata(
    songs: Vec<std::path::PathBuf>,
    metadata: AlbumInfo,
) -> Result<(), String> {
    for (idx, (song, track_name)) in std::iter::zip(songs, metadata.track_names).enumerate() {
        let mut tag = id3::Tag::new();
        tag.set_album(&metadata.name);
        tag.set_artist(&metadata.artist);
        tag.set_title(track_name);
        tag.set_track(1 + idx as u32);
        tag.add_frame(id3::frame::Comment {
            lang: "eng".to_string(),
            description: "tool".to_string(),
            text: "Created using Russel's Album Manager".to_string(),
        });
        if let Some(cover) = &metadata.cover {
            let mut cover_bytes = std::io::Cursor::new(Vec::new());
            cover
                .write_to(&mut cover_bytes, image::ImageOutputFormat::Jpeg(90))
                .unwrap();
            tag.add_frame(id3::frame::Picture {
                mime_type: String::from("image/jpeg"),
                picture_type: id3::frame::PictureType::CoverFront,
                description: String::new(),
                data: cover_bytes.into_inner(),
            });
        }
        tag.write_to_path(song, id3::Version::Id3v24)
            .ok()
            .ok_or("Couldn't set ID3 tag.")?;
    }
    Ok(())
}
