use clap::Parser;

#[derive(Parser,Default,Debug)]
#[clap(author="Matteo Torromacco", version, about="Applicazione per il test del carico")]
pub struct Cli {

    #[clap(forbid_empty_values = true)]
    /// Current platform
    pub url: String,

    #[clap(short, long, forbid_empty_values = true, default_value = "10")]
    /// Richieste per secondo
    pub requests_per_second: u32,

    #[clap(short, long, forbid_empty_values = true, default_value = "60")]
    /// Secondi di esecuzione
    pub seconds: u32,

    #[clap(short, long, default_value = "0")]
    /// Incremento percentuale delle richieste ad ogni secondo
    pub increment: u8,

    #[clap(short, long)]
    /// Eventuali headers da aggiungere alla richiesta
    pub headers: Vec<String>
}