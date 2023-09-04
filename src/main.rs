extern crate clap;
extern crate dotenv;
extern crate reqwest;
extern crate tokio;

use clap::{App, Arg};
use dotenv::dotenv;
use reqwest::Client;
use reqwest::Body;
use std::fmt;
use std::process::exit;
//use tokio::main;

extern crate serde;
use serde::Serialize;
use serde::Deserialize;
use serde_json;
use serde_json::Error as SerdeJsonError; // Import SerdeJsonError

// TagData, useful holder for any_tags vs not_tags
#[derive(Serialize)]
struct TagsData<'a> {
    any: Vec<&'a str>,
    not: Vec<&'a str>,
}

// Implement the Debug trait for TagsData
impl<'a> fmt::Debug for TagsData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\n    any: {:?},\n    not: {:?}\n}}", self.any, self.not)
    }
}

impl<'a> TagsData<'a> {
    fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("[!!!] Error serializing TagsData to JSON: {}", err);
                std::process::exit(1); // Exit with a non-zero status code
            }
        }
    }
}

fn parse_tags_data<'a>(tags: &'a str, not_tags: &'a str) -> TagsData<'a> {
    TagsData {
        any: tags.split(',').map(|s| s.trim()).collect(),
        not: not_tags.split(',').map(|s| s.trim()).collect(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    let mut api_hostname = "http://default-api-hostname.com".to_string();
    dotenv().ok();

    // Access the JUKECTL_HOST environment variable
    if let Ok(hostname) = std::env::var("JUKECTL_HOST") {
        api_hostname = hostname;
        println!("[-] jukectl API base URL: {}", api_hostname);
    } else {
        eprintln!("Error: JUKECTL_HOST environment variable is not set.");
        exit(1);
    }

    let matches = App::new("myctl")
        .version("1.0")
        .author("Your Name")
        .about("Command-line controller for a JSON web service")
        .subcommand(App::new("tag")
            .about("Tag an item")
            .arg(Arg::with_name("TagName")
                .help("Name of the tag")
                .required(true)))
        .subcommand(App::new("untag")
            .about("Untag an item")
            .arg(Arg::with_name("TagName")
                .help("Name of the tag")
                .required(true)))
        .subcommand(App::new("skip")
            .about("Skip an item"))
        .subcommand(App::new("playback")
            .about("Playback with tags")
            .arg(Arg::with_name("tags")
                .help("Tags for playback")
                .required(true))
            .arg(Arg::with_name("not_tags")
                .help("Tags to exclude from playback")
                .required(false)))
        .get_matches();

    // Handle subcommands
    match matches.subcommand() {
        //("tag", Some(tag_matches)) => {
        //    let tag_name = tag_matches.value_of("TagName").unwrap();
        //    tag_item(tag_name);
        //}
        //("untag", Some(untag_matches)) => {
        //    let tag_name = untag_matches.value_of("TagName").unwrap();
        //    untag_item(tag_name);
        //}
        ("skip", Some(_)) => {
            match skip_item(&api_hostname).await {
                Ok(_) => println!("[-] skip() function completed successfully."),
                Err(err) => eprintln!("[!] Error: {}", err),
            }
        }
	("playback", Some(playback_matches)) => {
	    let tags = playback_matches.value_of("tags").unwrap_or("");
	    let not_tags = playback_matches.value_of("not_tags").unwrap_or("");

	    let tags_data = parse_tags_data(tags, not_tags);

            match playback(&api_hostname, &tags_data).await {
                Ok(_) => println!("[-] playback() function completed successfully."),
                Err(err) => eprintln!("[!] Error: {}", err),
            }
        }
        _ => println!("Invalid subcommand. Use 'myctl --help' for usage."),
    }

    Ok(())
}

async fn skip_item(api_hostname: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/skip", api_hostname);

    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_LENGTH, "0")
        .send()
        .await?;

    if response.status().is_success() {
        println!("[+] Item skipped successfully.");
    } else {
        eprintln!("[!] Error: Failed to skip item (HTTP {})", response.status());
    }

    Ok(())
}

async fn playback(
    api_hostname: &str,
    tags_data: &TagsData<'_>,
) -> Result<(), reqwest::Error> {
    println!("[-] TagsData: {:?}", tags_data);

    let client = reqwest::Client::new();
    let url = format!("{}/tags", api_hostname);

    // Serialize it to JSON
    let json_data = tags_data.to_json();

    // Build the request
    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_data)
        .send()
        .await;

    let response = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(tags_data.to_json())
        .send()
        .await?;

    if response.status().is_success() {
        println!("Tags updated successfully.");
    } else {
        eprintln!("Error: Failed to update tags (HTTP {})", response.status());
    }

    Ok(())
}
