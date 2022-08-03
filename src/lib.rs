use anyhow::Result;
use smarthome::Home;
use std::{io, net::UdpSocket, thread, time::Duration};
use thiserror::Error;

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

pub trait SmartUdpThermometer {
    fn try_udp_handshake(socket: UdpSocket) -> HomeServerResult<UdpSocket>;
    fn stream_themperature(&self, home: &mut Home) -> Result<()>;
}

impl HomeConnection {
    pub fn new(udp: UdpSocket) -> HomeServerResult<Self> {
        let client = HomeConnection::try_udp_handshake(udp)?;
        println!(
            "Clinet {} has been successfully connected",
            client.local_addr()?
        );
        Ok(Self { udp: client })
    }
}

impl SmartUdpThermometer for HomeConnection {
    fn try_udp_handshake(socket: UdpSocket) -> HomeServerResult<UdpSocket> {
        let mut buf = [0; 5];
        println!("try handshake...");
        let (_, addr) = socket.recv_from(&mut buf)?;
        if &buf != b"smart" {
            let msg = format!("recieved string: {:?}", buf);
            return Err(HomeServerError::BadHandshake(msg));
        }
        socket.connect(addr)?;
        socket.send(b"home")?;
        Ok(socket)
    }

    fn stream_themperature(&self, home: &mut Home) -> Result<()> {
        loop {
            let device = home.get_room("first")?.get_device("t2").unwrap();
            let temp = device.return_state()?;
            // let temp = i32::from_be_bytes(temp);
            let len = temp.as_bytes().len() as u32;
            self.udp.send(len.to_be_bytes().as_slice())?;
            self.udp.send(temp.as_bytes())?;
            thread::sleep(Duration::from_secs(1));
        }
    }
}
