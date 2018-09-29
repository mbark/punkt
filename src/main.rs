extern crate dirs;
extern crate getopts;
extern crate git2;
extern crate pathdiff;
#[macro_use]
extern crate failure;

use failure::Error;
use getopts::Options;
use git2::Repository;
use pathdiff::diff_paths;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn print_usage(program: &str, opts: &Options) {
  let brief = format!("Usage: {} [options] command", program);
  print!("{}", opts.usage(&brief));
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optflag("h", "help", "print this help");
  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(f) => panic!(f.to_string()),
  };

  if matches.opt_present("h") {
    print_usage(&program, &opts);
    return;
  }

  if matches.free.is_empty() {
    panic!("must provide command to run");
  }

  let res = match matches.free[0].clone().as_ref() {
    "init" => init_repo(),
    "add" => index_fn(&matches.free[1].clone(), add_file),
    "remove" => index_fn(&matches.free[1].clone(), remove_file),
    cmd => Err(format_err!("invalid command: {}", cmd)),
  };

  match res {
    Ok(_) => (),
    Err(e) => print!("{:?}", e),
  }
}

fn init_repo() -> Result<(), Error> {
  let home = dirs::home_dir().ok_or_else(|| format_err!("unable to get home directory"))?;
  let dots = home.join(".dots");
  fs::create_dir_all(&dots)?;

  let mut config = Repository::init_bare(&dots)?.config()?;
  config.set_str("status.showUntrackedFiles", "no")?;

  print!("initialised new bare dotfiles-reop at {:?}", dots);
  Ok(())
}

fn index_fn<F>(file: &str, mut operation: F) -> Result<(), Error>
where
  F: FnMut(&mut git2::Index, &PathBuf) -> Result<(), git2::Error>,
{
  let home = dirs::home_dir().ok_or_else(|| format_err!("unable to get home directory"))?;
  let repo = get_dots()?;
  let mut index = repo.index()?;

  let path = Path::new(&file);
  let rel = diff_paths(path, home.as_path()).unwrap_or_else(|| path.to_path_buf());

  operation(&mut index, &rel)?;

  index.write()?;

  Ok(())
}

fn add_file(index: &mut git2::Index, path: &PathBuf) -> Result<(), git2::Error> {
  index.add_path(path)
}

fn remove_file(index: &mut git2::Index, path: &PathBuf) -> Result<(), git2::Error> {
  index.remove_path(path)
}

fn get_dots() -> Result<git2::Repository, Error> {
  let home = dirs::home_dir().ok_or_else(|| format_err!("unable to get home directory"))?;
  let dots = home.join(".dots");

  let repo = Repository::open_bare(dots)?;
  repo.set_workdir(home.as_path(), false)?;

  Ok(repo)
}
