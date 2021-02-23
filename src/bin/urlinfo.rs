
// TODO: move this into it's own lib crate with the cli binary
#[path = "../url_info.rs"]
mod url_info;
#[path = "../urlize.rs"]
mod urlize;

use url_info::ResourceInfo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use clap::{App, AppSettings, Arg, SubCommand};

    let matches = App::new("urlinfo")
        .version("1.0")
        .author("Leandro Lucarella <luca@llucax.com")
        .about("Get info about a URL")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("fetch")
                .about("fetch information about an URL")
                .arg(
                    Arg::with_name("URL")
                        .help("URL to retrieve info from")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("parse")
                .about("parse information form HTML")
                .arg(
                    Arg::with_name("FILE")
                        .default_value("-")
                        .help("File to parse, if is '-' or omitted, it reads from stdin")
                        .index(1),
                ),
        )
        .get_matches();

    let info = {
        if let Some(matches) = matches.subcommand_matches("fetch") {
            let url = matches.value_of("URL").unwrap();
            ResourceInfo::fetch(url).await.unwrap()
        } else {
            let matches = matches.subcommand_matches("parse").unwrap();
            // must be parse
            let file = matches.value_of("FILE").unwrap();
            if file == "-" {
                let mut buffer = String::new();
                use std::io::Read;
                std::io::stdin().read_to_string(&mut buffer)?;
                ResourceInfo::parse_str(&buffer, None).unwrap()
            } else {
                ResourceInfo::parse_file(file, None).unwrap()
            }
        }
    };

    dbg!(&info);

    println!("{}", info.urlize(1024).unwrap());

    Ok(())
}
