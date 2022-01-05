use std::{collections::HashMap, io::Write};

use anyhow::{Ok, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessExt, SystemExt};

use crate::config::{self, get_recent_config};

pub fn stop() {
    let mut system = sysinfo::System::default();
    system.refresh_processes();
    let ps = system.process_by_name("clash");
    if ps.len() == 0 {
        println!("no clash process to stop");
    }
    for p in ps.iter().filter(|p| p.name() == "clash") {
        p.kill();
    }
}

pub fn start(config: impl AsRef<str>) -> anyhow::Result<()> {
    stop();
    let time = chrono::Local::now();
    let log_path = format!("/home/iwazaki/.config/clash/logs/{}.log", time.to_rfc3339());
    let output = std::fs::File::create(log_path)?;
    std::process::Command::new("clash")
        .args(["-f", config.as_ref()])
        .stdout(output)
        .spawn()?;
    let service_data = ServiceData {
        current_profile: String::from(config.as_ref()),
    };
    let mut f_service = std::fs::File::create(std::env::var("CLASH_SERVICE_DATA")?)?;
    f_service.write_all(serde_json::to_string_pretty(&service_data)?.as_bytes())?;
    f_service.flush()?;
    println!("clash started with {}", config.as_ref());
    Ok(())
}

pub async fn update() -> anyhow::Result<String> {
    let servies = crate::config::latest_servers().await?;
    let fname = format!(
        "{}/servers_{}",
        std::env::var("CLASH_PROFILE_PATH")?,
        chrono::Local::now().to_rfc3339()
    );
    let mut file = std::fs::File::create(&fname)?;
    use std::io::prelude::*;
    file.write_all(serde_json::to_string(&servies)?.as_bytes())?;
    file.flush()?;
    let latest = config::stitch_latest().await?;
    start(&latest)?;
    Ok(latest)
}

pub fn switch(group: &str, to: &str) -> anyhow::Result<String> {
    assert!(matches!(group, "Choice" | "telegram"));
    assert!(matches!(
        to,
        "HongKong" | "US" | "Singapore" | "Taiwan" | "Japan" | "others"
    ));
    let re_str = format!(".*name: '{}'.*proxies: \\[.*\\]", group);
    let re = Regex::new(&re_str)?;
    let recent = get_recent_config()?;
    let content = std::fs::read_to_string(recent)?;
    let mut matches = false;
    let new_content: Vec<String> = content
        .lines()
        .map(|l| {
            if re.is_match(l) {
                matches = true;
                let new_group = format!(
                    "    - {{ name: '{}', type: select, proxies: ['{}']}}",
                    group, to
                );
                new_group
            } else {
                l.to_owned()
            }
        })
        .collect();
    if !matches {
        return Err(anyhow::Error::msg("no match"));
    }
    let f_name = format!(
        "{}/profile_{}.yaml",
        std::env::var("CLASH_PROFILE_PATH")?,
        chrono::Local::now().to_rfc3339()
    );
    let mut new_config = std::fs::File::create(&f_name)?;
    for l in new_content {
        new_config.write_all(l.as_bytes())?;
        new_config.write_all(&[b'\n'])?;
    }
    new_config.flush()?;
    stop();
    start(&f_name)?;
    Ok(f_name)
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceData {
    current_profile: String,
}

#[test]
fn regex_test() {
    let group = "telegram";
    let re_str = format!(".*name: 'telegram'.*proxies: \\[.*\\]");
    let re = Regex::new(&re_str).unwrap();
    // let re = Regex::new(r".*name: 'telegram'.*proxies: \[.*\]").unwrap();
    let input = "     - { name: 'telegram', type: select, proxies: ['US'] }";
    assert!(re.is_match(input));
    let re = Regex::new(r"name: '(?P<name>.*)',.*proxies: \[(?P<proxies>.*)\]").unwrap();
    assert!(re.is_match(input));
    let caps = re.captures(input).unwrap();
    let name = caps.name("name").unwrap().as_str();
    let proxies = caps.name("proxies").unwrap().as_str();
    assert_eq!(name, "telegram");
    assert_eq!(proxies, "'US'");
}

pub async fn status() -> Result<()> {
    let data = std::fs::read_to_string(std::env::var("CLASH_SERVICE_DATA")?)?;
    let data: ServiceData = serde_json::from_str(&data)?;
    let f_profile = std::fs::read_to_string(&data.current_profile)?;
    println!("current profile: {}", &data.current_profile);
    let mut groups = HashMap::new();
    let re = Regex::new(r"name: (?P<name>.*),.*proxies: \[(?P<proxies>.*)\]")?;
    for l in f_profile.lines() {
        if let Some(caps) = re.captures(l) {
            let name = caps.name("name").unwrap().as_str();
            let proxies = caps.name("proxies").unwrap().as_str();
            groups.insert(name, proxies);
        }
    }
    println!("current groups:");
    for (k, v) in groups {
        println!("{}: {}", k, v);
    }
    Ok(())
}
