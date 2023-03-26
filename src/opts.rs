use clap::{Parser, ValueHint};
use std::path;

pub fn init_logger() {
    pretty_env_logger::init_timed();
}

#[derive(Parser, Debug)]
pub struct SharedOpts {
    #[arg(short, long, value_name = "DIR", value_hint = ValueHint::DirPath)]
    pub data_dir: path::PathBuf,
}

impl SharedOpts {
    pub fn expand_data_dir(&mut self) -> crate::Result<()> {
        self.data_dir = self.data_dir.canonicalize()?;
        Ok(())
    }
}
