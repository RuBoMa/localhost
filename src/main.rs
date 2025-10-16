mod config;
mod core;
mod server;

use config::Config;
use server::Server;
use core::ClientConnection;

fn main() {
    let config = Config::from_file("config/config.toml")
        .expect("Failed to load config");

    println!("{:#?}", config);

    let mut server = Server::from_config(&config)
        .expect("Failed to initialize server");

    println!("{:#?}", server);
    println!("[*] Server initialized");
    server.run();
}