use std::{collections::HashMap, error::Error};
use mpd::Client;
use std::net::TcpStream;
use clap::Parser;
use std::error::Error;



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("starting mpd server....");
    let args = Args::parse();
    (())
}
