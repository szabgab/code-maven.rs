use std::fs::File;
use std::io::{BufRead, BufReader};

use clap::Parser;
use sendgrid::SGClient;
use sendgrid::{Destination, Mail};

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[arg(long)]
    tofile: String,
    // #[arg(long)]
    // mail: String,
}

fn main() {
    let args = Cli::parse();
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let addresses = read_tofile(&args.tofile);

    let sendgrid_api_key = get_key();
    let to_name = "Gabor Szabo".to_string();
    let subject = "Test mail".to_string();

    for (ix, to_address) in addresses.iter().enumerate() {
        log::info!("Sending {}/{}", ix + 1, addresses.len());
        sendgrid(&sendgrid_api_key, &to_name, to_address, &subject);
    }
}

fn read_tofile(path: &str) -> Vec<String> {
    log::info!("Read addresses from '{}'", path);

    let mut addresses: Vec<String> = vec![];
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
                log::info!("line '{}'", line);
                addresses.push(line);
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

fn sendgrid(api_key: &str, to_name: &str, to_address: &str, subject: &str) {
    let sg = SGClient::new(api_key);

    let mut x_smtpapi = String::new();
    x_smtpapi.push_str(r#"{"unique_args":{"test":7}}"#);

    let mail_info = Mail::new()
        .add_to(Destination {
            address: to_address,
            name: to_name,
        })
        .add_from("gabor@szabgab.com")
        .add_from_name("Original Sender")
        .add_subject(subject)
        .add_html("<h1>Hello from SendGrid!</h1>")
        .add_header("x-cool".to_string(), "indeed")
        .add_x_smtpapi(&x_smtpapi);

    match sg.send(mail_info) {
        Err(err) => println!("Error: {}", err),
        Ok(body) => println!("Response: {:?}", body),
    };
}
