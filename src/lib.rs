use anyhow::Result;
use async_io::udp::{recv_from_async, send_async};
use async_trait::async_trait;
use smarthome::Home;
use std::{io, thread, time::Duration};
use thiserror::Error;
use tokio::net::UdpSocket;

pub type HomeServerResult<T> = Result<T, HomeServerError>;

#[derive(Debug, Error)]
pub enum HomeServerError {
    #[error("Unexpected handshake: {0}")]
    BadHandshake(String),

    #[error("Can't connect to client")]
    Io(#[from] io::Error),
}

pub struct HomeConnection {
    udp: UdpSocket,
}

#[async_trait]
pub trait SmartUdpThermometer {
    async fn try_udp_handshake(socket: UdpSocket) -> HomeServerResult<UdpSocket>;
    async fn stream_themperature(&self, home: &mut Home) -> Result<()>;
}

impl HomeConnection {
    pub async fn new(udp: UdpSocket) -> HomeServerResult<Self> {
        let client = HomeConnection::try_udp_handshake(udp).await?;
        println!(
            "Clinet {} has been successfully connected",
            client.local_addr()?
        );
        Ok(Self { udp: client })
    }
}

#[async_trait]
impl SmartUdpThermometer for HomeConnection {
    async fn try_udp_handshake(socket: UdpSocket) -> HomeServerResult<UdpSocket> {
        let mut buf = [0; 5];
        println!("try handshake...");
        let addr = recv_from_async(&socket, &mut buf).await?;
        if &buf != b"smart" {
            let msg = format!("recieved string: {:?}", buf);
            return Err(HomeServerError::BadHandshake(msg));
        }
        println!("addr: {}", addr);
        socket.connect(addr).await?;
        send_async(&socket, b"home").await?;
        Ok(socket)
    }

    async fn stream_themperature(&self, home: &mut Home) -> Result<()> {
        loop {
            let device = home.get_room("first")?.get_device("t2").unwrap();
            let temp = device.return_state()?;
            println!("{}", temp);
            // let temp = i32::from_be_bytes(temp);
            let len = temp.as_bytes().len() as u32;
            send_async(&self.udp, len.to_be_bytes().as_slice()).await?;
            send_async(&self.udp, temp.as_bytes()).await?;
            thread::sleep(Duration::from_secs(1));
        }
    }
}
