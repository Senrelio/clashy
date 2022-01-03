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
        .get_matches();

    match matches.subcommand_name() {
        Some("start") => {
            let default = std::env::var("DEFAULT_PROFILE")?;
            let config = matches
                .subcommand_matches("start")
                .unwrap()
                .value_of("config").unwrap_or(&default);
            handle::start(config)?;
            println!("start clash with {}.", config);
        }
        Some("stop") => {
            handle::stop();
            println!("clash stopped.");
        }
        Some("update") => {
            let new_config = handle::update().await?;
            println!("start clash with {}.", new_config);
        }
        Some("switch") => {
            let subcommand = matches.subcommand_matches("switch").unwrap();
            let group = subcommand.value_of("group").unwrap_or("Choice");
            let to = subcommand.value_of("to").unwrap();
            handle::switch(group, to)?;
        }
        Some(_) => panic!("unknown subcommand"),
        None => panic!(),
    }

    Ok(())
}
