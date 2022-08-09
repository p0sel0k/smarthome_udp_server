use anyhow::Result;
use smarthome::SmartSocket;
use smarthome::*;
use smarthome_udp_server::{HomeConnection, SmartUdpThermometer};
use tokio::{self, net::UdpSocket};

#[tokio::main]
async fn main() -> Result<()> {
    let mut home = create_home()?;

    let connection = loop {
        let udp = UdpSocket::bind("127.0.0.1:8095").await?;
        println!("Server started on {}", udp.local_addr()?);
        match HomeConnection::new(udp).await {
            Ok(server) => {
                println!("connected!");
                break server;
            }
            Err(err) => {
                println!("{:?}", err);
                continue;
            }
        }
    };

    connection.stream_themperature(&mut home).await?;

    Ok(())
}

fn create_home() -> Result<Home> {
    let mut home = Home::new("home".into());

    let first_room_name = String::from("first");
    let second_room_name = String::from("second");

    let first_room = Room::new(first_room_name.clone());
    let second_room = Room::new(second_room_name.clone());

    println!("{}: {}", &first_room_name, home.add_room(first_room));
    println!("{}: {}", &second_room_name, home.add_room(second_room));

    let mut socket1 = SmartSocket::new("socket1".to_string());
    let termometer2 = SmartThermometer::new("t2".to_string());

    socket1.switch_on();

    home.add_device(&first_room_name, Box::new(socket1))?;
    home.add_device(&first_room_name, Box::new(termometer2))?;

    Ok(home)
}
