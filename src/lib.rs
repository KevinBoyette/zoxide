#![forbid(unsafe_code)]

pub mod config;
pub mod db;
pub mod error;
pub mod fzf;
pub mod subcommand;
pub mod util;

use structopt::StructOpt;

use crate::error::SilentExit;

#[derive(Debug, StructOpt)]
#[structopt(about, version = env!("ZOXIDE_VERSION"))]
enum Options {
    Add(subcommand::Add),
    Import(subcommand::Import),
    Init(subcommand::Init),
    Query(subcommand::Query),
    Remove(subcommand::Remove),
}

use anyhow::Result;

use std::process;

pub fn run() -> Result<()> {
    let opt = Options::from_args();

    let res = match opt {
        Options::Add(add) => add.run(),
        Options::Import(import) => import.run(),
        Options::Init(init) => init.run(),
        Options::Query(query) => query.run(),
        Options::Remove(remove) => remove.run(),
    };

    res.map_err(|e| match e.downcast::<SilentExit>() {
        Ok(SilentExit { code }) => process::exit(code),
        Err(e) => e,
    })
}
