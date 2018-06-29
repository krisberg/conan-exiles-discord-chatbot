extern crate discord;
extern crate regex;
extern crate rev_lines;
extern crate chrono;
extern crate prettytable;
extern crate rcon;

use discord::Discord;
use discord::model::Event;
use std::env;
use regex::Regex;
use rev_lines::RevLines;
use std::io::BufReader;
use std::fs::File;
use chrono::prelude::DateTime;
use chrono::{Utc};
use std::time::{UNIX_EPOCH, Duration};
use prettytable::Table;

struct Player {
    name: String,
    level: String,
    clan: String,
    last_seen: String,
}

fn handle_message(discord: &Discord, message: &discord::model::Message) {
    let mut responses: Vec<String> = Vec::new();

    match message.content.as_ref() {
        "!help" => {
            responses.push("```\n\
                Available commands:\n \
                !help - shows this text\n \
                !listplayers - shows currently online players\n \
                !listlastplayers - shows detailed info about the last active players\n \
                !status - shows server status\
                \n```".to_string());
        }
        "!listplayers" => {
            responses.push(format!("```\n{}\n```", list_online_players()));
        }
        "!listlastplayers" => {
            let players = parse_player_list_sql_result(get_player_list_from_db(10));
            match Table::from_csv_string(&list_players_as_csv(players)) {
                Ok(table) => {
                    responses.push(format!("```\n{}\n```", table));
                }
                Err(_) => { responses.push("Error".to_string()) }
            }
        }
        "!status" => {
            responses.push(format!("```\n{}\n```", get_server_status()));
        }
        _ => { return }
    }

    for response in responses {
        match discord.send_message(message.channel_id, &response, "", false) {
            Ok(_) => {}
            Err(err) => {println!("Receive error: {:?}", err)}
        }
    }
}

fn rcon(command: &str) -> String {
    let mut conn = match rcon::Connection::connect("localhost:25575",
         &env::var("RCON_PASSWORD").expect("Expected RCON_PASSWORD environment variable")
    ) {
        Err(e) => {
            println!("Got error: {}", e);
            return "".to_string();
        },
        Ok(conn) => conn,
    };

    let result = match conn.cmd(command) {
        Err(e) => {
            println!("Got error: {}", e);
            return "".to_string();
        },
        Ok(result) => result,
    };

    return result;
}

fn list_online_players() -> String {
    let rcon_result = rcon("listplayers");
    if rcon_result == "" { return "RCON Error".to_string() }

    let mut players_string: String = "Currently online players:".to_string();

    let re = Regex::new(r" \s*\d* \| ([a-zA-Z\s]*) \| ([a-zA-Z\s]*) \| ([\d]*)").unwrap();

    for cap in re.captures_iter(&rcon("listplayers")) {
        players_string += &format!("\n{}", &cap[1]);
    }

    return players_string;
}

fn get_player_list_from_db(limit: u32) -> String {
    let rcon_result = rcon(&format!(
        "sql SELECT char_name, level, guilds.name, lastTimeOnline \
        FROM characters \
        INNER JOIN guilds ON characters.guild = guilds.guildId \
        ORDER BY lastTimeOnline DESC \
        LIMIT {}", limit.to_string())
    );

    if rcon_result == "" { return "RCON Error".to_string() }

    return rcon_result;
}

fn list_players_as_csv(players: Vec<Player>) -> String {
    let mut csv: String = "name,level,clan,last_online".to_string();

    for player in players {
        csv += &format!("\n{},{},{},{}", player.name, player.level, player.clan, player.last_seen);
    }

    return csv;
}

fn parse_player_list_sql_result(sql_result: String) -> Vec<Player> {
    let mut players: Vec<Player> = Vec::new();

    let re = Regex::new(
        r"#\d*\s*([A-Za-z\s\d?_]*) \| \s*([\d]*) \| \s*([A-Za-z\s\d?_]*) \|\s*([\d]*)"
    ).unwrap();

    for cap in re.captures_iter(&sql_result) {
        let d = UNIX_EPOCH + Duration::from_secs(cap[4].parse::<u64>().unwrap());
        let datetime = DateTime::<Utc>::from(d);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        let player = Player {
            name: cap[1].to_owned(),
            level: cap[2].to_owned(),
            clan: cap[3].to_owned(),
            last_seen: timestamp_str,
        };
        players.push(player);
    }

    return players;
}

fn read_log(contains: &str) -> String {
    let logfile = format!("{}/Logs/ConanSandbox.log",
              &env::var("SAVED_DIR").expect("Expected SAVED_DIR environment variable"));

    let file = match File::open(logfile) {
        Err(e) => {
            use std::io::ErrorKind::*;
            println!("Got error: {}", e);
            match e.kind() {
                NotFound => {
                    println!("File not found");
                }
                k => {
                    println!("Error: {:?}", k);
                }
            }
            return "".to_string();
        },
        Ok(file) => file,
    };

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

    if report == "" { return "Error getting server status".to_string() };

    let re = Regex::new(
        r"players=([0-9]*)...*uptime=([0-9]*)...*cpu_time=([0-9]*.[0-9]*)"
    ).unwrap();
    let caps = re.captures(&report).unwrap();

    return format!("Server Report:\n Players: {} \n Uptime: {} \n CPU Usage: {}%",
                   &caps[1].to_owned(),
                   &seconds_to_string(caps[2].to_owned()),
                   &caps[3].to_owned()
    );
}

fn seconds_to_string(seconds_string: String) -> String {
    let seconds: i32 = seconds_string.parse().unwrap();
    let minutes = (seconds / 60) % 60;
    let hours = (seconds / 60 / 60) % 24;
    let days = (seconds / 60 / 60) / 24;
    let seconds = seconds % 60;

    let mut date_string: String = "".to_string();
    if days > 0 {
        date_string += &format!("{} days ", &days);
    }
    if hours > 0 {
        date_string += &format!("{} hours ", &hours);
    }
    if minutes > 0 {
        date_string += &format!("{} minutes ", &minutes);
    }
    if seconds > 0 {
        date_string += &format!("{} seconds ", &seconds);
    }

    return date_string;
}

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN")
            .expect("Expected DISCORD_TOKEN environment variable"),
    ).expect("login failed");

    let _rcon_password = &env::var("RCON_PASSWORD")
        .expect("Expected RCON_PASSWORD environment variable");
    let _saved_dir = &env::var("SAVED_DIR")
        .expect("Expected SAVED_DIR environment variable");

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
