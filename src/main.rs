extern crate clap;
extern crate clap_verbosity_flag;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate retry;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate structopt;

use std::io::Error;

use clap_verbosity_flag::Verbosity;
use structopt::StructOpt;

mod setup;
mod generate_token;
mod config_file_model;
mod sonar_api_model;
mod sonar_api;


#[derive(StructOpt, Debug)]
#[structopt(about = "Setup Sonarqube from a configuration file")]
pub struct MainCmd {
    #[structopt(flatten)]
    verbose: Verbosity,

    #[structopt(subcommand)]
    cmd: SubCmd,
}

#[derive(StructOpt, Debug)]
#[structopt()]
pub enum SubCmd {
    #[structopt(name = "setup", about = "Setup from a file")]
    SetupCmd {
        #[structopt(flatten)]
        setup: setup::SetupCmd,
    },

    #[structopt(name = "generate-token", about = "Generate a user token")]
    GenerateTokenCmd {
        #[structopt(flatten)]
        generate_token: generate_token::GenerateTokenCmd,
    },
}

fn main() -> Result<(), Error> {
    let cmd: MainCmd = MainCmd::from_args();

    match cmd.cmd {
        SubCmd::SetupCmd { setup: setup_cmd } => {
            setup::setup(setup_cmd);
        }
        SubCmd::GenerateTokenCmd {generate_token: generate_token_cmd } => {
            generate_token::generate_token(generate_token_cmd);
        }
    }

    debug!("Finish gracefully :-)");
    Ok(())
}
