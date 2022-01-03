use std::io::Write;

use anyhow::Ok;
use regex::Regex;
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

#[test]
fn regex_test() {
    let group = "telegram";
    let re_str = format!(".*name: 'telegram'.*proxies: \\[.*\\]");
    let re = Regex::new(&re_str).unwrap();
    // let re = Regex::new(r".*name: 'telegram'.*proxies: \[.*\]").unwrap();
    let input = "     - { name: 'telegram', type: select, proxies: ['US'] }";
    assert!(re.is_match(input));
}
