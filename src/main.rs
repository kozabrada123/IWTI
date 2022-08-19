use std::{fs, env, process::exit, collections::HashMap, time::Duration, time::Instant};
use reqwest::ResponseBuilderExt;
use serde_json;

mod parser;

fn main() {
    // Needed variables
    let mut url = "".to_string(); // The item url
    let mut opath = "".to_string(); // The output config path
    let mut profile = "".to_string(); // The profile name with a custom link
    let mut sid = "".to_string(); // The steam id 64
    let mut customlink = true; // Whether or not the user has a custom link
    let mut retry = false; // Whether or not to retry failed items

    let mut limit: usize = 50;

    // Start tracking time
    let start = Instant::now();

    let args =  env::args();

    // Get the command line parameters
    let mut prev_arg = "".to_string();

    for arg in args.into_iter() {
        
        match arg.as_str() {
            "-h" | "--help" | "--h" => {
                println!(
r#"Parameters:
        
        -i => Define a url inspect link of an item
        -p => Define a profile id / name
        -sid => Define a profile steam id 64
        
        For profiles both -p and -sid are required
        
        -l => Define a limit for the profile items. Defaults to 50
        -r => Retry rejects, likely wont do anything

        -o => Define a config file path

"#);
            },
            // We have a -r
            "-r" => {retry = true;},
            _ => {

                match prev_arg.as_str() {
                    // Previous parameter was -i
                    "-i" => {
                        url = arg.to_string();
                    },

                    // Previous parameter was -p
                    "-p" => {
                        profile = arg.to_string();
                    },

                    // Previous parameter was -sid
                    "-sid" => {
                        sid = arg.to_string();
                    },

                    // Previous parameter was -l
                    "-l" => {
                        limit = arg.to_string().parse::<usize>().unwrap();
                    },

                    // Previous parameter was -o
                    "-o" => {
                        opath = arg.to_string();
                    },
                    _ => {
                    }
                }
            }
        }

        prev_arg = arg;
    }

    // Check that we are good
    if url == "" && profile == "" && sid == "" {
        println!("No url or profile set! Use -i or -p");
        exit(0);
    }

    if (profile != "" && sid == "") {
        println!("Steam Id not provided. Use -sid");
        exit(0);
    }

    if (profile == "" && sid != "") {
        println!("Custom link not provided. Assuming /profiles/sid");
        customlink = false;
    }

    
    // Make a backup of the config
    println!("Making config backup..");
    fs::copy(&opath, format!("{}.old", &opath)).unwrap();

    // Setup what we'll write
    let mut writedata = "".to_string();

    // If we got a profile, load the whole profile
    if &url == "" && ((profile == "" && sid != "") || (profile != "" && sid != "")) {
        writedata = parser::get_inv(&profile, &sid, limit, retry, customlink);
    }

    // If we got just one item, load just that item
    if &profile == "" && sid == "" {
        writedata = parser::get_item(&url, None);
    }

    // Open the config in memory
    let mut confdata = fs::read_to_string(&opath).unwrap();
    // add a \n
    confdata.push_str("\n");

    // Find where we have the items and put our shit there
    confdata.insert_str(confdata.find(r#""Items": ["#).unwrap() + 10, &writedata);

    // Finally, write it
    println!("Saving config..");

    fs::write(&opath, confdata).unwrap();
    // We're done!
    println!("Done! (Everything took {:?})", start.elapsed());
}
