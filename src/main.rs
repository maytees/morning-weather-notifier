extern crate dotenv;

use dotenv::dotenv;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{env, fs, fs::File, io::Read, path::Path};
use twrs_sms;

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    temp: f32,
    weather: Weather,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiRes {
    data: Vec<Data>,
}

struct Clothing {
    shorts: bool,
    sweater: bool,
    umbrella: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // let mut config_file: File;

    // if !Path::new("~/.config/weathernotifier").exists() {
    //     println!("by there");
    //     fs::create_dir("~/.config/weathernotifier")?;
    //     println!("not either");
    //     fs::File::create("~/.config/weathernotifier/config.toml")?;
    //     println!("Another/!?!?!");
    //     config_file = File::create("~/.config/weathernotifier/config.toml")?;
    // } else {
    //     println!("Hi there");
    //     config_file = fs::File::open("~/.config/weathernotifier/config.toml")?;
    // }

    let mut host = String::from("");
    let mut tok = String::from("");

    for (key, value) in env::vars() {
        match key.as_str() {
            "WEATHERBIT_KEY" => tok = value,
            "WEATHERBIT_HOST" => host = value,
            _ => (),
        }
    }

    let client = reqwest::Client::new();

    let res = client
        .get("https://weatherbit-v1-mashape.p.rapidapi.com/current?lon=-77.3545&lat=38.96019&units=imperial&lang=en")
        .header(
            "X-RapidAPI-Key",
            tok,
        )
        .header("X-RapidAPI-Host", host)
        .send()
        .await?
        .json::<ApiRes>()
        .await?;

    let wdata: &Data;

    match res.data.first() {
        None => panic!("Nope"),
        Some(dat) => wdata = dat,
    }

    // send_msg(create_wmsg(wdata), config_file);

    send_msg(create_wmsg(wdata));

    println!("{:#?}", res);
    Ok(())
}

fn create_wmsg(weather_data: &Data) -> String {
    // Generate a message based on the given data
    let mut message = String::from("");
    let temp = weather_data.temp;
    let mut clothing = Clothing {
        shorts: false,
        sweater: false,
        umbrella: false,
    };

    match weather_data.weather.description.as_str() {
        "Clear sky" => {
            clothing.umbrella = false;
            message.push_str("Today's sky is clear.");
        }
        _ => {
            let mut statement = String::from("I don't know what this weather description is: ");
            statement.push_str(weather_data.weather.description.as_str());
            message.push_str(statement.as_str());
            message.push_str(". You should add it!");
        }
    }

    if temp >= 70.0 {
        clothing.shorts = true;
        clothing.sweater = false;
    } else if temp >= 60.0 {
        clothing.shorts = true;
        clothing.sweater = true;
    } else {
        clothing.shorts = false;
        clothing.sweater = true;
    }

    let mut statement = String::from(" Since the temp is ");
    statement.push_str(temp.to_string().as_str());

    if clothing.shorts {
        statement.push_str("F outside, you should wear shorts, and ");
    } else {
        statement.push_str(", you should not wear shorts, and ");
    }

    if clothing.sweater {
        statement.push_str("you should also wear a sweater.");
    } else {
        statement.push_str("you should not wear a sweater.");
    }

    if clothing.umbrella {
        statement.push_str(
            " Since there will be percepitation today, you should bring an umbrella, just in case",
        );
    }

    message.push_str(statement.as_str());

    println!("{}", message);
    message
}

// fn send_msg(weather_msg: String, config_file: File) {
fn send_msg(weather_msg: String) {
    let mut to = String::new();
    let mut from = String::new();
    let mut sid = String::new();
    let mut tok = String::new();

    for (key, value) in env::vars() {
        match key.as_str() {
            "ACCOUNT_SID" => sid = value,
            "AUTH_TOK" => tok = value,
            "TWILIO_NUMBER" => from = value,
            "TO_NUMBER" => to = value,
            _ => (),
        }
    }

    let t: twrs_sms::TwilioSend = twrs_sms::TwilioSend {
        To: &to,
        From: &from,
        Body: &weather_msg.as_str(),
    };

    let encoded = t.encode().expect("Error encoding");

    let mut res =
        twrs_sms::send_message(&sid, &tok, encoded).expect("Something went wrong with the request");

    assert_eq!(StatusCode::from_u16(201).unwrap(), res.status());

    let delivered =
        twrs_sms::is_delivered(&mut res, &sid, &tok).expect("Error - SMS not delivered");

    assert_eq!(delivered, "delivered");
}
