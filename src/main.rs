use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use log::{info, error};
use env_logger;
use std::fs::File;
use std::process;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;

/// Program converts JSON Telegram's contacts to vCards
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to JSON file
    #[clap(short, long)]
    file_path: String,
}

#[derive(Serialize, Deserialize)]
struct Data {
    about: String,
    contacts: Contacts,
    frequent_contacts: FrequentContacts
}

#[derive(Serialize, Deserialize)]
struct FrequentContacts {
    about: String,
    list: Vec<FrequentContact>,
}

#[derive(Serialize, Deserialize)]
struct FrequentContact {
    id: u64,
    category: String,
    #[serde(alias="type")] type_: String,
    name: String,
    rating: f32,
}

#[derive(Serialize, Deserialize)]
struct Contacts {
    about: String,
    list: Vec<Contact>
}

#[derive(Serialize, Deserialize)]
struct Contact {
    first_name: String,
    last_name: String,
    phone_number: String,
    date: String,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    info!("Using file: {}", args.file_path);

    let file: File = match File::open(args.file_path) {
        Ok(file) => file,
        Err(msg) => {
            error!("{}", msg);
            process::exit(9);
        }
    };

    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    match buf_reader.read_to_string(&mut content) {
        Ok(_) => (),
        Err(msg) => {
            error!("{}", msg);
            process::exit(8);
        }
    };
    
    let data: Data = match serde_json::from_str(&content) {
        Ok(data) => data,
        Err(msg) => {
            error!("{}", msg);
            process::exit(7);
        }
    };

    let mut contacts: HashMap<String, Vec<String>> = HashMap::new();

    for contact in data.contacts.list {
        let key: String = String::from(format!("{} {}", contact.last_name, contact.first_name).trim());
        
        info!("Processing contact: {}", key);
        match contacts.get_mut(&key) {
            Some(numbers) => {
                numbers.push(contact.phone_number)
            },
            _ => {
                let mut number: Vec<String> = Vec::new();
                number.push(contact.phone_number);
                contacts.insert(key, number);
            }
        };
    }

    for (contact, numbers) in contacts.iter() {
        let filename = format!("{}.vcf", contact).replace("\"", "");
        let mut file = match File::create(filename.clone()) {
            Err(msg) => {
                error!("{}: {}", filename, msg);
                continue
            },
            Ok(file) => file
        };

        fn make_number(number: &String) -> String {
            let patterns : &[_] = &['+', '0'];
            let mut number = number.trim_start_matches(patterns).replace(' ', "");
            if number.starts_with("7") {
                number = format!("+{}", number);
            }
            format!("TEL;TYPE#work,voice;VALUE#uri:tel:{}", number)
        }

        info!("Contact \"{}\" has {} numbers", contact, numbers.len());
        let vcard = format!(
            "BEGIN:VCARD\n\
             VERSION:4.0\n\
             FN:{}\n\
             {}\n\
             END:VCARD", 
             contact, numbers.iter().map(make_number).collect::<Vec<String>>().join("\n")
        );

        info!("Writing contact's data to vCard");
        match file.write_all(&vcard.as_bytes()) {
            Err(msg) => error!("{}", msg),
            _ => {}
        }
    }
}