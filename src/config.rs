use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{prelude::*, BufReader, Write};
use std::path::PathBuf;

use anyhow::Result;
use hyper::Uri;
use serde::{Deserialize, Serialize};

use crate::protocol::Server;

pub async fn latest_servers() -> anyhow::Result<Vec<Server>> {
    let uri = std::env::var("PROFILE_URI")?.parse::<Uri>()?;
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let resp = client.get(uri).await?;
    let body = resp.into_body();
    let body = hyper::body::to_bytes(body).await?;
    let s = String::from_utf8(body.to_vec())?;
    let s: String = base64::decode(s)?.into_iter().map(|u| u as char).collect();
    let servers = s.lines().map(|l| l.parse().unwrap()).collect();
    Ok(servers)
}

pub(crate) async fn stitch_latest() -> Result<String> {
    let static_config = std::fs::File::open(std::env::var("STATIC_BASE")?)?;
    println!("open static config base");
    let mut configs = std::fs::read_dir(std::env::var("CLASH_PROFILE_PATH")?)?
        .map(|f| f.unwrap())
        .collect::<Vec<_>>();
    configs.sort_by_key(|p| p.metadata().unwrap().created().unwrap());
    let f_server = configs
        .iter().rev()
        .find(|p| p.file_name().to_str().unwrap().contains("servers"))
        .unwrap()
        .path();
    let servers: Vec<Server> =
        serde_json::from_str(&std::fs::read_to_string(f_server.to_str().unwrap()).unwrap())?;
    let rules = std::fs::File::open(std::env::var("RULE_BASE")?)?;
    println!("open rules base");
    stitch(static_config, &servers, rules)
}

fn stitch(static_config: File, servers: &[Server], rules: File) -> Result<String> {
    let path = format!(
        "/home/iwazaki/.config/clash/config/profile_{}.yaml",
        chrono::Local::now().to_rfc3339()
    );
    let mut file = File::create(&path)?;
    println!("create file: {}", &path);
    let mut buf = vec![];
    // write static config to buf;
    let mut static_config = BufReader::new(static_config);
    static_config.read_to_end(&mut buf)?;
    // write servers to buf
    buf.extend_from_slice("\nproxies:\n".as_bytes());
    for s in servers.iter().map(|s| format!("- {}", s.to_string())) {
        buf.push(b'\n');
        buf.extend_from_slice("    ".as_bytes());
        buf.extend_from_slice(s.as_bytes());
    }
    buf.extend_from_slice("\n\n".as_bytes());
    // parse group from servers and write
    let groups = parse_group(servers)?;
    buf.extend_from_slice(groups.as_bytes());
    // write rules
    let mut rules = BufReader::new(rules);
    rules.read_to_end(&mut buf)?;
    file.write(&buf)?;
    file.flush()?;
    Ok(path)
}

lazy_static::lazy_static! {
    static ref RE_COUNTRIES: regex::Regex =
        regex::Regex::new(r"(?P<country>香港|美国|新加坡|台湾|日本)").unwrap();
}

fn parse_group(servers: &[Server]) -> Result<String> {
    let mut country_group = HashMap::new();
    for s in servers.into_iter() {
        let name = s.name();
        let country = RE_COUNTRIES
            .captures(&name)
            .map_or("others", |s| s.name("country").unwrap().as_str());
        country_group
            .entry(String::from(country))
            .or_insert_with(Vec::new)
            .push(s);
    }
    let mut buf = vec![];
    buf.extend_from_slice("\nproxy-groups:\n".as_bytes());
    buf.extend_from_slice("    - { name: 'Direct', type: select, proxies: [DIRECT] }\n".as_bytes());
    buf.extend_from_slice(
        "    - { name: 'Reject', type: select, proxies: [REJECT,DIRECT] }\n".as_bytes(),
    );
    buf.extend_from_slice(
        "    - { name: 'Unmatched', type: select, proxies: ['HongKong'] }\n".as_bytes(),
    );
    let groups: HashMap<String, String> = country_group
        .into_iter()
        .map(|(k, v)| {
            let country_en = match k.as_str() {
                "香港" => "HongKong",
                "美国" => "US",
                "新加坡" => "Singapore",
                "台湾" => "Taiwan",
                "日本" => "Japan",
                "others" => "others",
                _ => unimplemented!("countries unknown"),
            }
            .to_string();
            let proxies = v
                .into_iter()
                .map(|s| s.name())
                .collect::<Vec<String>>()
                .join(", ");
            (country_en, proxies)
        })
        .collect();
    for (country, proxies) in &groups {
        buf.extend_from_slice(
            format!(
                "    - {{ name: {}, type: select, proxies: [{}] }}\n",
                country, proxies
            )
            .as_bytes(),
        );
    }
    buf.extend_from_slice(
        "    - { name: 'Choice', type: select, proxies: ['HongKong'] }\n".as_bytes(),
    );
    buf.extend_from_slice("    - { name: 'telegram', type: select, proxies: ['Choice'] }\n".as_bytes());
    Ok(String::from_utf8(buf)?)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Group {
    name: String,
    g_type: GroupType,
    proxies: HashSet<Server>,
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum GroupType {
    Select,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum GroupTag {
    US,
    Hongkong,
    Taiwan,
    Japan,
    Others,
    Direct,
    Reject,
    Choice,
}

pub fn get_recent_config() -> Result<PathBuf> {
    let mut configs = std::fs::read_dir(std::env::var("CLASH_PROFILE_PATH")?)?
        .map(|f| f.unwrap())
        .collect::<Vec<_>>();
    // println!("looking for profiles");
    configs.sort_by_key(|p| p.metadata().unwrap().created().unwrap());
    let recent = configs
        .iter().rev()
        .find(|p| p.file_name().to_str().unwrap().contains("yaml"))
        .unwrap()
        .path();
    // println!("find recent: {}", recent.to_str().unwrap());
    Ok(recent)
}
