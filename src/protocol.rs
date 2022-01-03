use std::str::FromStr;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize, Serialize)]
pub enum Server {
    Direct,
    Reject,
    Vmess(Vmess),
    SS(ShadowSocks),
}

impl Server {
    pub fn name(&self) -> String {
        String::from(match self {
            Server::Vmess(v) => format!("'{}'", &v.name),
            Server::SS(s) => format!("'{}'", &s.name),
            Server::Direct => String::from("DIRECT"),
            Server::Reject => String::from("REJECT"),
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct Vmess {
    #[serde(rename(deserialize = "v",serialize = "v"))]
    version: String,
    #[serde(rename(deserialize = "ps", serialize = "ps"))]
    name: String,
    #[serde(rename(deserialize = "add",serialize = "add"))]
    host: String,
    port: String,
    #[serde(rename(deserialize = "id",serialize = "id"))]
    uuid: String,
    #[serde(rename(deserialize = "aid", serialize = "aid"))]
    alter_id: String,
}
// #[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
// pub struct Vmess {
//     #[serde(alias = "v")]
//     version: String,
//     #[serde(alias = "ps")]
//     name: String,
//     #[serde(alias = "add")]
//     host: String,
//     port: String,
//     #[serde(alias = "id")]
//     uuid: String,
//     #[serde(alias = "aid")]
//     alter_id: String,
// }
#[derive(PartialEq, Eq, Hash, Debug, Deserialize, Serialize, Clone)]
pub struct ShadowSocks {
    name: String,
    host: String,
    port: i32,
    cipher: String,
    password: String,
    udp: bool,
}

lazy_static! {
    static ref RE_PROTO: regex::Regex =
        regex::Regex::new(r"^(?P<p>ss|vmess)://(?P<body>.*)").unwrap();
    static ref RE_SS: regex::Regex =
        regex::Regex::new(r"(?P<cipher>.*)@(?P<server>.*)#(?P<name>.*)").unwrap();
    static ref RE_VMESS: regex::Regex = regex::Regex::new(r"").unwrap();
}

impl FromStr for Server {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cap = RE_PROTO.captures(s).unwrap();
        let proto = cap.name("p").unwrap().as_str();
        let body = cap.name("body").unwrap().as_str();
        match proto {
            "ss" => {
                let caps = RE_SS.captures(body).unwrap();
                let cipher = caps.name("cipher").unwrap().as_str();
                let cipher = String::from_utf8(base64::decode(cipher).unwrap()).unwrap();
                let (cipher, password) = cipher.split_once(':').unwrap();
                let server = caps.name("server").unwrap().as_str();
                let (host, port) = server.split_once(':').unwrap();
                let name = caps.name("name").unwrap().as_str();
                Ok(Server::SS(ShadowSocks {
                    name: urlencoding::decode(name).unwrap().to_string(),
                    host: String::from(host),
                    port: port.parse().unwrap(),
                    cipher: String::from(cipher),
                    password: String::from(password),
                    udp: true,
                }))
            }
            "vmess" => {
                let body = String::from_utf8(base64::decode(body).unwrap()).unwrap();
                Ok(Server::Vmess(serde_json::from_str(&body).unwrap()))
            }
            _ => Err("unexpected proto".into()),
        }
    }
}

impl ToString for Server {
    fn to_string(&self) -> String {
        match self {
            Server::Vmess(v) => v.to_string(),
            Server::SS(ss) => ss.to_string(),
            Server::Direct => "DIRECT".to_owned(),
            Server::Reject => "REJECT".to_owned(),
        }
    }
}

impl ToString for ShadowSocks {
    fn to_string(&self) -> String {
        format!(
            "{{ name: '{}', type: ss, server: {}, port: {}, cipher: {}, password: {}, udp: {} }}",
            self.name, self.host, self.port, self.cipher, self.password, self.udp
        )
    }
}
impl ToString for Vmess {
    fn to_string(&self) -> String {
        format!("{{ name: '{}', type: vmess, server: {}, port: {}, uuid: {}, alterId: {}, cipher: auto, udp: true }}",
        self.name, self.host, self.port, self.uuid, self.alter_id
    )
    }
}
