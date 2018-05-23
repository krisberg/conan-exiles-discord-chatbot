extern crate discord;
extern crate regex;
extern crate rev_lines;

use discord::Discord;
use discord::model::Event;
use std::env;
use regex::Regex;
use rev_lines::RevLines;
use std::io::BufReader;
use std::fs::File;
use std::process::Command;

fn handle_message(discord: &Discord, message: &discord::model::Message) {
    println!("{} says: {}", message.author.name, message.content);

    let response: String;

    match message.content.as_ref() {
        "!help" => {
            response = String::from(
                "Available commands:\n \
                !help - shows this text\n \
                !listplayers - shows currently online players\n \
                !status - shows server status"
            );
        }
        "!listplayers" => {
            response = list_players();
        }
        "!status" => {
            response = get_server_status();
        }
        _ => { return }
    }

    match discord.send_message(message.channel_id, &response, "", false) {
        Ok(_) => {}
        Err(err) => {println!("Receive error: {:?}", err)}
    }
}

fn rcon(command: &str) -> String {
    let output = Command::new("mcrcon")
        .arg("-c")
        .arg("-H")
        .arg("127.0.0.1")
        .arg("-P")
        .arg("25575")
        .arg("-p")
        .arg(&env::var("RCON_PASSWORD").expect("Expected RCON_PASSWORD environment variable"))
        .arg(command)
        .output()
        .expect("failed to execute process");

    return String::from_utf8_lossy(&output.stdout).to_string();
}

fn list_players() -> String {
    return rcon("listplayers");
}

fn read_log(contains: &str) -> String {
    let file = File::open(
        format!(
            "{}{}",
            &env::var("CONAN_DIR").expect("Expected CONAN_DIR environment variable"),
            "/ConanSandbox/Saved/Logs/ConanSandbox.log"
        )).unwrap();
    let rev_lines = RevLines::new(BufReader::new(file)).unwrap();

    for line in rev_lines {
        if line.contains(contains) {
            return line;
        }
    }
    return "".to_string();
}

fn get_server_report() -> String {
    return read_log("LogServerStats: Sending report: exiles-stats?");
}

fn get_server_status() -> String {
    let report = get_server_report();

    let re = Regex::new(r"players=([0-9]*)").unwrap();
    let caps = re.captures(&report).unwrap();
    let players = caps[1].to_owned();

    let re = Regex::new(r"uptime=([0-9]*)").unwrap();
    let caps = re.captures(&report).unwrap();
    let uptime = caps[1].to_owned();

    let re = Regex::new(r"cpu_time=([0-9]*.[0-9]*)").unwrap();
    let caps = re.captures(&report).unwrap();
    let cpu = caps[1].to_owned();

    return String::from("Server Report:\n Players: ") + &players +
        &String::from("\n Uptime: ") + &seconds_to_string(uptime) +
        &String::from("\n CPU Usage: ") + &cpu + &String::from("%");
}

fn seconds_to_string(seconds_string: String) -> String {
    let seconds: i32 = seconds_string.parse().unwrap();
    let minutes = (seconds / 60) % 60;
    let hours = (seconds / 60 / 60) % 24;
    let days = (seconds / 60 / 60) / 24;
    let seconds = seconds % 60;

    return String::from("Days: ") + &days.to_string() + &String::from(" Hours: ") + &hours.to_string() + &String::from(" Minutes: ") + &minutes.to_string() + &String::from(" Seconds: ") + &seconds.to_string();
}

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN environment variable"),
    ).expect("login failed");

    // Establish and use a websocket connection
    let (mut connection, _) = discord.connect().expect("connect failed");
    println!("Ready.");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                handle_message(&discord, &message);
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
                break
            }
            Err(err) => println!("Receive error: {:?}", err)
        }
    }
}
