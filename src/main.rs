extern crate dirs;
extern crate getopts;
extern crate git2;
extern crate pathdiff;
#[macro_use]
extern crate failure;

use failure::Error;
use std::path::Path;
use getopts::Options;
use git2::Repository;
use std::env;
use std::fs;
use pathdiff::diff_paths;

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options] target", program);
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
    print_usage(&program, opts);
    return;
  }

  if matches.free.is_empty() {
    panic!("must provide command to run");
  }

  let res = match matches.free[0].clone().as_ref() {
    "init" => init_repo(),
    "add" => add_file(matches.free[1].clone()),
    cmd => Err(format_err!("invalid command: {}", cmd)),
  };

  match res {
    Ok(_) => print!("ok!"),
    Err(e) => print!("{:?}", e),
  }
}

fn init_repo() -> Result<(), Error> {
  let home = dirs::home_dir().expect("unable to find home directory");
  let dots = home.join(".dots");
  fs::create_dir_all(&dots)?;

  let mut config = Repository::init_bare(dots)?.config()?;
  config.set_str("status.showUntrackedFiles", "no")?;

  Ok(())
}

fn add_file(file: String) -> Result<(), Error> {
  let home = dirs::home_dir().expect("unable to find home directory");
  let dots = home.join(".dots");
  let path = Path::new(&file);

  let rel = diff_paths(path, home.as_path()).unwrap_or(path.to_path_buf());

  let repo = Repository::open_bare(dots)?;
  repo.set_workdir(home.as_path(), false)?;
  let mut index = repo.index()?;

  index.add_path(&rel)?; 

  index.write()?;

  Ok(())
}
