// This file is part of the uutils coreutils package.
//
// (c) Vsevolod Velichko <torkvemada@sorokdva.net>
// (c) Jian Zeng <anonymousknight96 AT gmail.com>
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// spell-checker:ignore (ToDO) NEWROOT Userspec pstatus
mod error;

use crate::error::ChrootError;
use clap::{crate_version, Arg, Command};
use nix::unistd::{chdir, chroot, setgid, setgroups, setuid, Gid};
use std::path::Path;
use std::process;
use uucore::error::{set_exit_code, UResult};
use uucore::fs::{canonicalize, MissingHandling, ResolveMode};
use uucore::{entries, format_usage};

static ABOUT: &str = "Run COMMAND with root directory set to NEWROOT.";
static USAGE: &str = "{} [OPTION]... NEWROOT [COMMAND [ARG]...]";

mod options {
    pub const NEWROOT: &str = "newroot";
    pub const USER: &str = "user";
    pub const GROUP: &str = "group";
    pub const GROUPS: &str = "groups";
    pub const USERSPEC: &str = "userspec";
    pub const COMMAND: &str = "command";
    pub const SKIP_CHDIR: &str = "skip-chdir";
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    let args = args.collect_lossy();

    let matches = uu_app().get_matches_from(args);

    let default_shell: &'static str = "/bin/sh";
    let default_option: &'static str = "-i";
    let user_shell = std::env::var("SHELL");

    let newroot: &Path = match matches.value_of(options::NEWROOT) {
        Some(v) => Path::new(v),
        None => return Err(ChrootError::MissingNewRoot.into()),
    };

    if !newroot.is_dir() {
        return Err(ChrootError::NoSuchDirectory(format!("{}", newroot.display())).into());
    }

    let commands = match matches.get_many::<String>(options::COMMAND) {
        Some(v) => v.map(|s| s.as_str()).collect(),
        None => vec![],
    };

    // TODO: refactor the args and command matching
    // See: https://github.com/uutils/coreutils/pull/2365#discussion_r647849967
    let command: Vec<&str> = match commands.len() {
        0 => {
            let shell: &str = match user_shell {
                Err(_) => default_shell,
                Ok(ref s) => s.as_ref(),
            };
            vec![shell, default_option]
        }
        _ => commands,
    };

    assert!(!command.is_empty());
    let chroot_command = command[0];
    let chroot_args = &command[1..];

    // NOTE: Tests can only trigger code beyond this point if they're invoked with root permissions
    set_context(newroot, &matches)?;

    let pstatus = match process::Command::new(chroot_command)
        .args(chroot_args)
        .status()
    {
        Ok(status) => status,
        Err(e) => return Err(ChrootError::CommandFailed(command[0].to_string(), e).into()),
    };

    let code = if pstatus.success() {
        0
    } else {
        pstatus.code().unwrap_or(-1)
    };
    set_exit_code(code);
    Ok(())
}

pub fn uu_app<'a>() -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .arg(
            Arg::new(options::NEWROOT)
                .value_hint(clap::ValueHint::DirPath)
                .hide(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new(options::USER)
                .short('u')
                .long(options::USER)
                .help("User (ID or name) to switch before running the program")
                .value_name("USER"),
        )
        .arg(
            Arg::new(options::GROUP)
                .short('g')
                .long(options::GROUP)
                .help("Group (ID or name) to switch to")
                .value_name("GROUP"),
        )
        .arg(
            Arg::new(options::GROUPS)
                .short('G')
                .long(options::GROUPS)
                .help("Comma-separated list of groups to switch to")
                .value_name("GROUP1,GROUP2..."),
        )
        .arg(
            Arg::new(options::USERSPEC)
                .long(options::USERSPEC)
                .help(
                    "Colon-separated user and group to switch to. \
                     Same as -u USER -g GROUP. \
                     Userspec has higher preference than -u and/or -g",
                )
                .value_name("USER:GROUP"),
        )
        .arg(
            Arg::new(options::SKIP_CHDIR)
                .long(options::SKIP_CHDIR)
                .help("Do not change working directory to '/'"),
        )
        .arg(
            Arg::new(options::COMMAND)
                .value_hint(clap::ValueHint::CommandName)
                .hide(true)
                .multiple_occurrences(true)
                .index(2),
        )
}

fn set_context(root: &Path, options: &clap::ArgMatches) -> UResult<()> {
    let userspec_str = options.value_of(options::USERSPEC);
    let user_str = options.value_of(options::USER).unwrap_or_default();
    let group_str = options.value_of(options::GROUP).unwrap_or_default();
    let groups_str = options.value_of(options::GROUPS).unwrap_or_default();
    let userspec = match userspec_str {
        Some(u) => {
            let s: Vec<&str> = u.split(':').collect();
            if s.len() != 2 || s.iter().any(|&spec| spec.is_empty()) {
                return Err(ChrootError::InvalidUserspec(u.to_string()).into());
            };
            s
        }
        None => Vec::new(),
    };

    let (user, group) = if userspec.is_empty() {
        (user_str, group_str)
    } else {
        (userspec[0], userspec[1])
    };

    let skip_chdir = options.is_present(options::SKIP_CHDIR);

    if skip_chdir {
        if let Ok(path) = canonicalize(root, MissingHandling::Existing, ResolveMode::Physical) {
            if path.to_str().map(|p_str| p_str != "/").unwrap_or(true) {
                return Err(ChrootError::WrongSkipChdirUsage.into());
            }
        }
    }

    enter_chroot(root, skip_chdir)?;

    set_groups_from_str(groups_str)?;
    set_main_group(group)?;
    set_user(user)?;
    Ok(())
}

fn enter_chroot(root: &Path, skip_chdir: bool) -> UResult<()> {
    chroot(root)
        .map_err(|err| ChrootError::CannotChroot(format!("{}", root.display()), err.into()))?;
    if !skip_chdir {
        chdir("/").map_err(|err| ChrootError::CannotChdirRoot(err.into()))?;
    }
    Ok(())
}

fn set_main_group(group: &str) -> UResult<()> {
    if !group.is_empty() {
        let group_id = match entries::grp2gid(group) {
            Ok(g) => g,
            _ => return Err(ChrootError::NoSuchGroup(group.to_string()).into()),
        };
        if let Err(err) = setgid(group_id.into()) {
            return Err(ChrootError::SetGidFailed(group_id.to_string(), err.into()).into());
        }
    }
    Ok(())
}

fn set_groups_from_str(groups: &str) -> UResult<()> {
    if !groups.is_empty() {
        let mut groups_vec = vec![];
        for group in groups.split(',') {
            let gid = match entries::grp2gid(group) {
                Ok(g) => Gid::from(g),
                Err(_) => return Err(ChrootError::NoSuchGroup(group.to_string()).into()),
            };
            groups_vec.push(gid);
        }
        if let Err(e) = setgroups(&groups_vec) {
            return Err(ChrootError::SetGroupsFailed(e.into()).into());
        }
    }
    Ok(())
}

fn set_user(user: &str) -> UResult<()> {
    if !user.is_empty() {
        let user_id = entries::usr2uid(user).unwrap();
        if let Err(err) = setuid(user_id.into()) {
            return Err(ChrootError::SetUserFailed(user.to_string(), err.into()).into());
        }
    }
    Ok(())
}
