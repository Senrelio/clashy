use clap::{App, Arg, SubCommand};
use clash_clap::handle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::from_path("/home/iwazaki/.config/clash/.env")?;
    let matches = App::new("clashclap")
        .version("alpha-1")
        .author("songfanzhen@gmail.com")
        .about("manipulate clash core")
        .subcommand(
            SubCommand::with_name("start")
                .arg(Arg::with_name("config").short("-c").takes_value(true)),
        )
        .subcommand(SubCommand::with_name("stop"))
        .subcommand(SubCommand::with_name("update"))
        .subcommand(
            SubCommand::with_name("switch")
                .arg(Arg::with_name("group").short("-g").takes_value(true))
                .arg(Arg::with_name("to").takes_value(true)),
        )
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("edit"))
        .get_matches(); 

    match matches.subcommand_name() {
        Some("start") => {
            // let default = std::env::var("DEFAULT_PROFILE")?;
            let recent_profile = clash_clap::config::get_recent_config()?;
            let recent_profile = recent_profile.to_str().unwrap();
            let config = matches
                .subcommand_matches("start")
                .unwrap()
                .value_of("config")
                .unwrap_or(recent_profile);
            handle::start(config)?;
        }
        Some("stop") => {
            handle::stop();
        }
        Some("update") => {
            let _new_config = handle::update().await?;
        }
        Some("switch") => {
            let subcommand = matches.subcommand_matches("switch").unwrap();
            let group = subcommand.value_of("group").unwrap_or("Choice");
            let to = subcommand.value_of("to").unwrap();
            handle::switch(group, to)?;
        }
        Some("status") => {
            handle::status().await?;
        }
        Some("edit") => {
            handle::edit().await?;
        }
        Some(_) => panic!("unknown subcommand"),
        None => panic!(),
    }

    Ok(())
}
