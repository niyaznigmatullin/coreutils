use chrono::{DateTime, Local};
use clap::Command;
use uucore::error::UResult;


fn file_last_modified_time(path: &str) -> String {
    std::fs::metadata(path)
        .map(|i| {
            i.modified()
                .map(|x| {
                    let date_time: DateTime<Local> = x.into();
                    date_time.format("%b %d %H:%M %Y").to_string()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let args = args.collect_ignore();
    if args.len() < 2 {
        println!("{}", get_time());
    } else {
        println!("{}", file_last_modified_time(&args[1]));
    }
    Ok(())
}

pub fn uu_app<'a>() -> Command<'a> {
    Command::new(uucore::util_name())
}

pub fn get_time() -> DateTime<Local> {
    Local::now()
}
