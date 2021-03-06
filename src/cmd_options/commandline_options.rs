use std::path::PathBuf;


use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "basic", about = "Basic command line config")]
pub struct CommandlineOptions {
    #[structopt(parse(from_os_str))]
    pub config_directory: Option<PathBuf>,
}