mod cli;

use clap::Parser;
use cli::Cli;

fn main() {

    let cli: Cli = Cli::parse();

    match loadhero::run(cli.seconds, cli.requests_per_second, cli.increment, cli.url, cli.headers, cli.query_strings) {
        Ok(_) => {},
        Err(err) => panic!("Si è verificato un errore! ERR: {}", err)
    }
    
}