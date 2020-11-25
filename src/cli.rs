pub fn create_parser<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new("noclickd")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("noclick.me API server")
        .arg(
            clap::Arg::with_name("bind")
                .short("b")
                .long("bind")
                .default_value("127.0.0.1:8080")
                .value_name("IP:PORT")
                .help("Set the address/port to bind the server to")
                .takes_value(true),
        )
}
