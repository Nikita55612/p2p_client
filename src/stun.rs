use std::sync::Arc;

use tokio::{
	net::{ToSocketAddrs, UdpSocket},
	time::{timeout, Duration}
};
use thiserror::Error;
use rand::Rng;


const MAGIC_COOKIE: [u8; 4] = [33, 18, 164, 66];

#[derive(Debug)]
pub struct ConnAddr {
    pub ip: std::net::Ipv4Addr,
    pub port: u16
}

impl ConnAddr {
    pub fn new(ip: std::net::Ipv4Addr, port: u16) -> Self {
        Self{ ip, port }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.ip.to_string(), self.port)
    }
}

#[derive(Error, Debug)]
pub enum StunError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid response received from STUN server")]
    InvalidResponse,
    #[error("Received response is too short")]
    ResponseTooShort,
    #[error("Unknown or unsupported message attribute")]
    UnknownAttribute,
	#[error("Response timeout expire")]
	TimeoutExpire (#[from] tokio::time::error::Elapsed),
}

pub async fn send_binding_request(
	socket_addr: impl ToSocketAddrs,
	stun_addr: impl ToSocketAddrs
) -> Result<ConnAddr, StunError> {
    let socket = UdpSocket::bind(socket_addr).await?;
    let request = new_binding_request();
    socket.send_to(&request, stun_addr).await?;
    let mut buf = [0u8; 512];
	let len = timeout(
		Duration::from_secs(5),
		socket.recv(&mut buf)
	).await??;
    let received_data = buf[..len].to_vec();
	if received_data.len() < 20 {
		return Err(StunError::ResponseTooShort);
	}
    if request[4..] != received_data[4..20] {
        return Err(StunError::InvalidResponse);
    }
    parse_response(&received_data)
}

fn new_binding_request() -> [u8; 20] {
    let mut buf = [0u8; 20];
    let mut rng = rand::rng();
    buf[8..].iter_mut().for_each(|v| *v = rng.random());
    buf[4..8].copy_from_slice(&MAGIC_COOKIE);
    buf[1] = 1;
    buf
}

fn parse_response(r: &[u8]) -> Result<ConnAddr, StunError> {
    if r.len() < 32 {
        return Err(StunError::ResponseTooShort);
    }
    match u16::from_be_bytes([r[20], r[21]]) {
        0x0020 => {
            let port = u16::from_be_bytes([r[26], r[27]])
                ^ ((u32::from_be_bytes(MAGIC_COOKIE) >> 16) as u16);
            let ip = std::net::Ipv4Addr::new(
                r[28] ^ MAGIC_COOKIE[0],
                r[29] ^ MAGIC_COOKIE[1],
                r[30] ^ MAGIC_COOKIE[2],
                r[31] ^ MAGIC_COOKIE[3],
            );
            Ok(ConnAddr::new(ip, port))
        }
        0x0001 => {
            let port = u16::from_be_bytes([r[26], r[27]]);
            let ip = std::net::Ipv4Addr::new(
				r[28], r[29], r[30], r[31]
			);
            Ok(ConnAddr::new(ip, port))
        }
        _ => Err(StunError::UnknownAttribute),
    }
}
