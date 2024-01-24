use std::fs::File;
use std::io::{BufRead, BufReader};

use regex::Regex;
use sendgrid::v3::{
    ClickTrackingSetting, Content, Email, Message, OpenTrackingSetting, Personalization, Sender,
    SubscriptionTrackingSetting, TrackingSettings,
};

use crate::{read_config, read_md_file};

#[derive(Debug)]
struct EmailAddress {
    name: String,
    email: String,
}

pub fn cm_sendgrid(root: &str, mail: &str, tofile: &str) {
    let config = read_config(root);

    let (page, _paths) = match read_md_file(&config, root, mail) {
        Ok(page) => page,
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    };

    let from = config.from.unwrap_or_else(|| {
        log::error!("The 'from' field is missing from the config file");
        std::process::exit(1);
    });

    let from = EmailAddress {
        name: from.name,
        email: from.email,
    };

    let addresses = read_tofile(tofile);

    let sendgrid_api_key = get_key();

    for (ix, to_address) in addresses.iter().enumerate() {
        log::info!(
            "Sending {}/{}  {} <{}>",
            ix + 1,
            addresses.len(),
            to_address.name,
            to_address.email
        );
        sendgrid(
            &sendgrid_api_key,
            &from,
            to_address,
            &page.title,
            &page.content,
        );
    }
}

fn read_tofile(path: &str) -> Vec<EmailAddress> {
    log::info!("Read addresses from '{}'", path);
    let re_full = Regex::new(r"(.+?)\s*<(.+)>").unwrap();

    let mut addresses: Vec<EmailAddress> = vec![];
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                if line.starts_with('#') {
                    continue;
                }
                if !line.contains('@') {
                    continue;
                }
                let parts = line.split(',').collect::<Vec<&str>>();

                log::info!("line '{}'", parts[1]);
                let address = match re_full.captures(parts[1]) {
                    Some(value) => EmailAddress {
                        name: value[1].to_owned(),
                        email: value[2].to_owned(),
                    },
                    None => EmailAddress {
                        name: "".to_string(),
                        email: parts[1].to_string(),
                    },
                };
                println!("{address:?}");

                addresses.push(address);
            }
        }
        Err(error) => {
            println!("Error opening file {path}: {error}");
        }
    }

    addresses
}

fn get_key() -> String {
    let filename = "config.txt";
    match File::open(filename) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let parts = line.split('=');
                let parts: Vec<&str> = parts.collect();
                if parts[0] == "SENDGRID_API_KEY" {
                    return parts[1].to_string();
                }
            }
            panic!("Could not find line");
        }
        Err(error) => {
            panic!("Error opening file {filename}: {error}");
        }
    }
}

fn sendgrid(api_key: &str, from: &EmailAddress, to: &EmailAddress, subject: &str, html: &str) {
    let person = Personalization::new(Email::new(&to.email).set_name(&to.name));

    let message = Message::new(Email::new(&from.email).set_name(&from.name))
        .set_subject(subject)
        .add_content(Content::new().set_content_type("text/html").set_value(html))
        .set_tracking_settings(TrackingSettings {
            click_tracking: Some(ClickTrackingSetting {
                enable: Some(false),
                enable_text: None,
            }),
            subscription_tracking: Some(SubscriptionTrackingSetting {
                enable: Some(false),
            }),
            open_tracking: Some(OpenTrackingSetting {
                enable: Some(false),
                substitution_tag: None,
            }),
        })
        .add_personalization(person);

    let sender = Sender::new(api_key.to_owned());
    match sender.blocking_send(&message) {
        Ok(res) => println!("sent {}", res.status()),
        Err(err) => eprintln!("err: {err}",),
    }
}
