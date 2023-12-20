use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Parser;
use regex::Regex;
use sendgrid::SGClient;
use sendgrid::{Destination, Mail};

use code_maven::{read_config, read_md_file};

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long, default_value = ".")]
    root: String,

    #[arg(long)]
    tofile: String,

    #[arg(long)]
    mail: String,
}

#[derive(Debug)]
struct EmailAddress {
    name: String,
    email: String,
}

fn main() {
    let args = Cli::parse();
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let config = read_config(&args.root);

    // outdir would be needed if there were images to be copied
    let page = match read_md_file(&config, &args.root, &args.mail, "") {
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

    let addresses = read_tofile(&args.tofile);

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
                println!("{:?}", address);

                addresses.push(address);
            }
        }
        Err(error) => {
            println!("Error opening file {}: {}", path, error);
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
            panic!("Error opening file {}: {}", filename, error);
        }
    }
}

fn sendgrid(api_key: &str, from: &EmailAddress, to: &EmailAddress, subject: &str, html: &str) {
    let sg = SGClient::new(api_key);

    let mut x_smtpapi = String::new();
    x_smtpapi.push_str(r#"{"unique_args":{"test":7}}"#);

    let mail_info = Mail::new()
        .add_to(Destination {
            address: &to.email,
            name: &to.name,
        })
        .add_from(&from.email)
        .add_from_name(&from.name)
        .add_subject(subject)
        .add_html(html)
        .add_header("x-cool".to_string(), "indeed")
        .add_x_smtpapi(&x_smtpapi);

    match sg.send(mail_info) {
        Err(err) => println!("Error: {}", err),
        Ok(_body) => (), //println!("Response: {:?}", body),
    };
}
