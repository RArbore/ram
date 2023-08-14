mod metadata;

fn main() -> Result<(), String> {
    let mut args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() > 2,
        "Must provide at least two arguments; the album name, and a playlist URL."
    );
    args.remove(0);
    let album_name = args.remove(0);
    let mut playlist_urls: Vec<String> = vec![];
    let mut listings_to_use: Vec<usize> = vec![];
    if args.len() == 1 {
        playlist_urls.push(args.remove(0));
        listings_to_use.push(0);
    } else {
        for arg in args {
            if let Ok(idx) = arg.parse::<usize>() {
                listings_to_use.push(idx);
            } else {
                playlist_urls.push(arg);
            }
        }
    }

    let metadata = metadata::scrape_wikipedia(&album_name, &listings_to_use)?;
    println!("{}", metadata);

    std::fs::create_dir_all(&album_name)
        .ok()
        .ok_or(format!("Couldn't create directory \"{}\".", &album_name))?;

    for playlist_url in playlist_urls {
        let num_downloaded = std::fs::read_dir(&album_name)
            .ok()
            .ok_or("Couldn't list files in download directory.")?
            .count();
        let playlist_specifier = format!(
            "%(playlist_index+{})02d - %(title)s.%(ext)s",
            num_downloaded
        );
        let ytdlp_args = vec![
            "--extract-audio",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "5",
            "--force-ipv4",
            "-o",
            &playlist_specifier,
            &playlist_url,
        ];
        let ytdlp_code = std::process::Command::new("yt-dlp")
            .args(ytdlp_args)
            .current_dir(&album_name)
            .status()
            .expect("Couldn't spawn yt-dlp process.");
        assert!(ytdlp_code.success(), "yt-dlp returned bad error code.");
    }

    let mut songs = std::fs::read_dir(&album_name)
        .ok()
        .ok_or("Couldn't list files in download directory.")?
        .map(|x| x.unwrap().path())
        .collect::<Vec<std::path::PathBuf>>();
    songs.sort();

    metadata::update_album_metadata(songs, metadata)?;

    Ok(())
}
