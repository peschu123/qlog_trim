extern crate walkdir;
use anyhow::{Context, Error};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[clap(name = "qlog_trim")]
/// This program reads a qlik log file (or any other text file) and removes leading
/// spaces (left trim) from every line.
/// Log files in Qlik Sense Enterprise can (sometimes) have leading whitespaces.
/// It is not clear why and when they have these whitespaces(at least to me).
/// This is quite annoying if you want to (or have to) work with log files created by QS Enterprise.
/// For example if you process the logs with qlik itself (fixed length) :).

#[clap(author, version, about, long_about = None)]
struct Args {
    /// File or directory for input
    /// current directory is assumed used when empty
    #[clap(required = true, short, long)]
    source: String,

    /// Target directory where processed logs will be stored
    /// If file exists in the target it will be overwritten
    #[clap(required = true, short, long)]
    target: String,

    /// search subfolders for files with .log
    /// max value 65535
    #[clap(short, long, default_value = "1")]
    max_depth: u16,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let source_dir = Path::new(&args.source);
    let target_dir = Path::new(&args.target);
    if source_dir.eq(target_dir) {
        println!("Error: source and target are equal, please select a different target directory");
        process::exit(1)
    }
    if !source_dir.is_dir() {
        println!("Error: source must be directory not a file");
        process::exit(1)
    }

    let mut count = 0;

    for entry in WalkDir::new(source_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_type().is_dir())
        .filter(|entry| is_log(entry))
    {
        count += 1;
        println!("{}", &entry.path().display());
        trim(entry.path(), target_dir)?;
    }
    println!("processed files: {}", count);
    println!("{:?}", source_dir);
    Ok(())
}

fn is_log(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .and_then(|entry| {
            if entry.ends_with(".log") {
                Some(true)
            } else {
                Some(false)
            }
        })
        .unwrap_or(false)
}

fn trim(log_file: &Path, target_dir: &Path) -> Result<(), Error> {
    let f = File::open(log_file).with_context(|| format!("Failed to open file {:?}", log_file))?;
    let reader = BufReader::new(f);
    let outfile = target_dir.join(log_file.file_name().expect("Target is not a directory"));
    let file = File::create(&outfile)?;
    let mut writer = BufWriter::new(file);
    for line in reader.lines() {
        //println!("{}\r\n", line?.trim_start());
        write!(writer, "{}\r\n", line?.trim_start())?;
    }
    Ok(())
}
