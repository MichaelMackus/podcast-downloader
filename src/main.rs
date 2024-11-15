use std::io;
use std::fs::File;
use std::path::Path;

use roxmltree::Node;
use downloader::Downloader;
use downloader::Error;
use url::Url;
use podcast_downloader::progress::{SimpleReporter};

const THREADS: u16 = 8;
const RETRIES: u16 = 3;

fn node_url(node: Node) -> Option<Url> {
    if let Some(text) = node.text() {
        Url::parse(text).ok()
    } else if let Some(text) = node.attribute("url") {
        Url::parse(text).ok()
    } else {
        None
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage:\n\tcargo run -- INPUT_XML MP3_XML_TAG");
        std::process::exit(1);
    }

    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("output"))
        .parallel_requests(THREADS)
        .retries(RETRIES)
        .build()
        .unwrap();
    let mut dls = vec![];

    let text = std::fs::read_to_string(&args[1]).expect("Unable to read input XML file");
    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };
    let doc = roxmltree::Document::parse_with_options(&text, opt).expect("Unable to parse input XML file");
    for node in doc.descendants() {
        if node.is_element() && node.tag_name().name() == args[2] {
            let url = node_url(node).expect("Error parsing URL from podcast XML element");
            let filename = url.path_segments().expect("Error parsing URL").last().expect("Error parsing URL");

            if !Path::new("output").join(filename).exists() {
                dls.push(downloader::Download::new(url.as_str())
                    .file_name(Path::new(filename))
                    .progress(SimpleReporter::create()));
            }
        }
    }

    // TODO need to verify downloads are successful - the downloader "succeeds" if there's an error downloading in the middle of the file
    let result = downloader.download(&dls).unwrap();
    for r in result {
        match r {
            Err(e) => {
                eprintln!("Error: {}", e.to_string());

                // remove file from FS
                match e {
                    Error::File(summary) | Error::Download(summary) => {
                        if summary.file_name.exists() {
                            eprintln!("Removing file: {}", summary.file_name.display());
                            if let Err(_) = std::fs::remove_file(summary.file_name) {
                                eprintln!("Error removing file!");
                            }
                        }
                    },
                    _ => {},
                }
            },
            Ok(s) => println!("Success: {}", &s),
        };
    }
}
