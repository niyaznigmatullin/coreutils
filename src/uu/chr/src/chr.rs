use chrono::{DateTime, Local};
use clap::Command;
use uucore::error::UResult;

#[uucore::main]
pub fn uumain(_args: impl uucore::Args) -> UResult<()> {
    println!("{}", get_time());
    Ok(())
}

pub fn uu_app<'a>() -> Command<'a> {
    Command::new(uucore::util_name())
}

pub fn get_time() -> DateTime<Local> {
    Local::now()
}
