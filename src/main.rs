use clap::Parser;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use systemd_tmpfile::{Entry, LineType};
use tracing::info;

/// Creates, deletes and cleans up volatile and temporary files and directories
///
/// If invoked with no arguments, it applies all directives from all configuration files. If one or
/// more filenames are passed on the command line, only the directives in these files are applied.
/// If only the basename of a configuration file is specified, all configuration directories as
/// specified in tmpfiles.d(5) are searched for a matching file.
#[derive(Parser, Debug)]
struct Options {
    /// Create missing files and directories.
    ///
    /// Files and directories marked with f, F, w, d, D, v, p, L, c, b, m in the configuration files
    /// are created or written to. Files and directories marked with z, Z, t, T, a, and A have their
    /// ownership, access mode and security labels set.
    #[arg(long)]
    create: bool,

    /// Execute lines with an exclamation mark.
    #[arg(long)]
    boot: bool,

    /// Only apply rules with paths that start with the specified prefix.
    ///
    /// This option can be specified multiple times.
    #[arg(long)]
    prefix: Vec<PathBuf>,

    /// Configuration file.
    ///
    /// If one or more filenames are passed on the command line, only the directives in these files
    /// are applied.
    #[arg()]
    config_file: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let opts = Options::parse();
    info!("opts: {:?}", opts);

    let entries = entries_from_config_files(opts.config_file);
    for entry in entries {
        if !opts.prefix.is_empty() {
            let has_allowed_prefix = opts.prefix.iter().any(|prefix| {
                if entry.path().starts_with(prefix) {
                    return true;
                }
                false
            });
            if !has_allowed_prefix {
                info!(
                    "Skipping entry {} because it doesn't start with an allowed prefix",
                    entry.path().display()
                );
                continue;
            }
        }
        if *entry.line_type() == LineType::DirectoryCreateAndClean {
            fs::create_dir_all(entry.path())?;
            info!("Created directory {}", entry.path().display());
        }
        info!("{:?}", entry);
    }

    info!("Done!");
    Ok(())
}

fn entries_from_config_files(config_files: Vec<PathBuf>) -> impl Iterator<Item = Entry> {
    config_files
        .into_iter()
        .flat_map(|config_file| {
            if let Ok(file) = File::open(&config_file) {
                let reader = io::BufReader::new(file);
                Some(reader.lines().filter_map(|line| {
                    let line = line.unwrap();
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        return None;
                    }
                    Some(Entry::from_str(trimmed))
                }))
            } else {
                None
            }
        })
        .flatten()
}
