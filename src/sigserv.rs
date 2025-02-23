use serde::Deserialize;
use thiserror::Error;


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Peer {
	pub ip: String,
	pub port: u16
}

impl Peer {
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.ip.to_string(), self.port)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Chan {
	pub id: String,
	pub peers: Vec<Peer>
}

#[derive(Error, Debug)]
pub enum SignalServerError {
    #[error("Reqwest error: {0}")]
    Io(#[from] reqwest::Error),
	#[error("Bad request")]
	BadRequest,
	#[error("Serde json error: {0}")]
    SerDeError(#[from] serde_json::Error),
}

pub async fn create_chan(ip: &std::net::Ipv4Addr, port: &u16) -> Result<String, SignalServerError> {
	let url = format!("http://185.204.2.206:8099/chan?ip={}&port={}", ip.to_string(), *port);
	let res = reqwest::get(url).await?;
	if res.status() != 200 {
		return Err(SignalServerError::BadRequest);
	}
	let body = res.text().await?;
	let chan = serde_json::from_str::<Chan>(&body)?;
	Ok(chan.id)
}

pub async fn get_chan(id: &str) -> Result<Chan, SignalServerError> {
	let url = format!("http://185.204.2.206:8099/chan/{}", id);
	let res = reqwest::get(url).await?;
	if res.status() != 200 {
		return Err(SignalServerError::BadRequest);
	}
	let body = res.text().await?;
	let chan = serde_json::from_str::<Chan>(&body)?;
	Ok(chan)
}

pub async fn chan_join(id: &str, ip: &std::net::Ipv4Addr, port: &u16) -> Result<Chan, SignalServerError> {
	let url = format!("http://185.204.2.206:8099/chan/{}?ip={}&port={}", id, ip.to_string(), *port);
	let res = reqwest::get(url).await?;
	if res.status() != 200 {
		return Err(SignalServerError::BadRequest);
	}
	let body = res.text().await?;
	let chan = serde_json::from_str::<Chan>(&body)?;
	Ok(chan)
}

pub async fn del_chan(id: &str) -> Result<String, SignalServerError> {
	let url = format!("http://185.204.2.206:8099/chan/del/{}", id);
	let res = reqwest::get(url).await?;
	if res.status() != 200 {
		return Err(SignalServerError::BadRequest);
	}
	let body = res.text().await?;
	Ok(body)
}
