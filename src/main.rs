extern crate wikipedia;
use scraper::Html;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        2,
        "Must provide exactly one argument; the album name."
    );
    let album_name = args.pop().unwrap();

    let wiki = wikipedia::Wikipedia::<wikipedia::http::default::Client>::default();
    let page = wiki.page_from_title(album_name.clone());
    let content = page
        .get_html_content()
        .expect(&("Couldn't find wikipedia page about \"".to_owned() + &album_name + "\"."));
    let document = Html::parse_document(&content);
    for node in document.tree.nodes() {
        let value = node.value();
        if value.is_element()
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
                                    tracks.push(node.value().as_text().unwrap());
                                    continue 'tr;
                                }
                            }
                        }
                    }
                }
            }
            tracks.pop().unwrap();
            println!("{:?}", tracks);
        }
    }
}
