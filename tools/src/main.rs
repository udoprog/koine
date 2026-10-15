//! Downloads koine's vendored source data and regenerates its tables.
//!
//! Updating koine's data is: `cargo run -p tools -- download`, then
//! `cargo run -p tools`, then review the diff. `data/language/to-3166.csv` is
//! maintained by hand; no command here ever writes it.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};

mod cldr;
mod generate;

// Vendored source data, downloaded by the `download*` commands.
const LANGUAGES_TAB: &str = "data/language/iso-639-3.tab";
/// The files taken from the pinned Unicode CLDR release; see `cldr.rs`.
const CLDR_DIR: &str = "data/cldr";

// Hand-maintained source data. Read by the regenerate step, never written.
const LANGUAGES_TO_3166: &str = "data/language/to-3166.csv";

// Generated, committed table modules.
const LANGUAGE_GENERATED: &str = "src/language/generated.rs";
const COUNTRY_GENERATED: &str = "src/country/generated.rs";
const LOCALE_GENERATED: &str = "src/locale/generated.rs";

/// Download koine's vendored source data and regenerate its tables. With no
/// subcommand it regenerates every table from the committed data files.
#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Fail instead of writing when a generated table is out of date.
    #[arg(long, global = true)]
    check: bool,
    /// Also write the flag each language ended up with as CSV to this path.
    #[arg(long, global = true)]
    mapping_path: Option<PathBuf>,
    /// Read the CLDR files from this directory instead of `data/cldr`.
    #[arg(long, global = true)]
    cldr_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate every table from the committed data files (the default).
    Generate,
    /// Regenerate `src/language/generated.rs`.
    Language,
    /// Regenerate `src/country/generated.rs`.
    Country,
    /// Regenerate `src/locale/generated.rs`.
    Locale,
    /// Download the ISO 639-3 table into `data/language/iso-639-3.tab`.
    DownloadLanguages,
    /// Download the pinned Unicode CLDR release and vendor the files the
    /// country and locale tables need into `data/cldr/`.
    DownloadCldr,
    /// Download every data file. Regenerate afterwards and review the diff.
    Download,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = workspace_root();
    let cldr_dir = cli.cldr_dir.clone().unwrap_or_else(|| root.join(CLDR_DIR));

    let mut out = Output {
        root: &root,
        cldr_dir: &cldr_dir,
        check: cli.check,
        stale: Vec::new(),
    };

    match &cli.command {
        None | Some(Command::Generate) => {
            generate_language(&mut out, cli.mapping_path.as_deref())?;
            generate_country(&mut out)?;
            generate_locale(&mut out)?;
        }
        Some(Command::Language) => generate_language(&mut out, cli.mapping_path.as_deref())?,
        Some(Command::Country) => generate_country(&mut out)?,
        Some(Command::Locale) => generate_locale(&mut out)?,
        Some(Command::DownloadLanguages) => download_languages(&mut out)?,
        Some(Command::DownloadCldr) => download_cldr(&mut out)?,
        Some(Command::Download) => {
            download_languages(&mut out)?;
            download_cldr(&mut out)?;
        }
    }

    if !out.stale.is_empty() {
        bail!(
            "Out of date: {}. Run `cargo run -p tools` and review the diff.",
            out.stale.join(", ")
        );
    }

    Ok(())
}

/// The repository root, from this crate's manifest dir (`<root>/tools`) at
/// compile time, so paths don't depend on the current directory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tools crate lives one level below the repository root")
        .to_path_buf()
}

fn generate_language(out: &mut Output<'_>, mapping_path: Option<&Path>) -> Result<()> {
    let tab = out.read(LANGUAGES_TAB)?;
    let to_3166 = out.read(LANGUAGES_TO_3166)?;
    let countries = countries(out)?.into_iter().map(|c| c.alpha2).collect();

    let (module, mapping) = generate::language_module(&tab, &to_3166, &countries)?;
    out.write(LANGUAGE_GENERATED, &module)?;

    if let Some(path) = mapping_path {
        let mut o = csv::Writer::from_writer(
            File::create(path).with_context(|| anyhow!("Creating {}", path.display()))?,
        );

        for m in mapping {
            o.serialize(m)?;
        }

        o.flush()?;
    }

    Ok(())
}

fn generate_country(out: &mut Output<'_>) -> Result<()> {
    let countries = countries(out)?;
    out.write(COUNTRY_GENERATED, &generate::country_module(&countries)?)
}

fn generate_locale(out: &mut Output<'_>) -> Result<()> {
    let languages = generate::language_codes(&out.read(LANGUAGES_TAB)?)?;
    let countries = countries(out)?;
    let listing = out.read_cldr(cldr::MAIN_LOCALES)?;

    out.write(
        LOCALE_GENERATED,
        &generate::locale_module(&listing, &languages, &countries)?,
    )
}

/// The ISO 3166-1 countries of the vendored CLDR data.
fn countries(out: &Output<'_>) -> Result<Vec<cldr::Country>> {
    cldr::countries(
        &out.read_cldr(cldr::REGION_VALIDITY)?,
        &out.read_cldr(cldr::SUPPLEMENTAL_DATA)?,
        &out.read_cldr(cldr::ENGLISH)?,
    )
}

fn download_languages(out: &mut Output<'_>) -> Result<()> {
    let body = download(generate::LANGUAGES_URL, "the ISO 639-3 table")?;
    let body = String::from_utf8(body).context("The downloaded ISO 639-3 table is not UTF-8")?;
    generate::validate_languages(&body).context("The downloaded ISO 639-3 table")?;
    out.write(LANGUAGES_TAB, &body)
}

fn download_cldr(out: &mut Output<'_>) -> Result<()> {
    let zip = download(&cldr::url(), &format!("Unicode CLDR {}", cldr::VERSION))?;
    let files = cldr::extract(&zip)?;

    // Check the new files make a table before any of them is written.
    let find = |name: &str| {
        files
            .iter()
            .find(|(path, _)| path == name)
            .map(|(_, body)| body.as_str())
            .with_context(|| anyhow!("Nothing extracted for {name}"))
    };

    cldr::countries(
        find(cldr::REGION_VALIDITY)?,
        find(cldr::SUPPLEMENTAL_DATA)?,
        find(cldr::ENGLISH)?,
    )
    .context("The downloaded CLDR release")?;

    for (path, body) in &files {
        out.write(&format!("{CLDR_DIR}/{path}"), body)?;
    }

    Ok(())
}

fn download(url: &str, what: &str) -> Result<Vec<u8>> {
    println!("Downloading {what} from {url}");

    let body = reqwest::blocking::get(url)
        .and_then(|response| response.error_for_status())
        .with_context(|| anyhow!("Downloading {what}"))?
        .bytes()
        .with_context(|| anyhow!("Reading {what}"))?;

    Ok(body.to_vec())
}

/// Where files are read from and written to.
struct Output<'a> {
    root: &'a Path,
    cldr_dir: &'a Path,
    /// Record out-of-date files in `stale` instead of writing them.
    check: bool,
    stale: Vec<String>,
}

impl Output<'_> {
    /// Read a vendored CLDR file, checking first that the vendored release is
    /// the pinned one.
    fn read_cldr(&self, relative: &str) -> Result<String> {
        let version = read(&self.cldr_dir.join(cldr::VERSION_FILE))
            .context("No CLDR data is vendored; run `cargo run -p tools -- download-cldr`")?;

        if version.trim() != cldr::VERSION {
            bail!(
                "The vendored CLDR data is version {}, but the tools pin {}; run `cargo run -p tools -- download-cldr`",
                version.trim(),
                cldr::VERSION
            );
        }

        read(&self.cldr_dir.join(relative))
    }

    fn read(&self, relative: &str) -> Result<String> {
        read(&self.root.join(relative))
    }

    /// Write `body` to `relative`, leaving the file alone (and its mtime
    /// untouched) when it already holds exactly that.
    fn write(&mut self, relative: &str, body: &str) -> Result<()> {
        // The hand-maintained mapping has no upstream to restore it from.
        if relative == LANGUAGES_TO_3166 {
            bail!("{relative} is maintained by hand and is never written by the tools");
        }

        let path = self.root.join(relative);

        if fs::read(&path).is_ok_and(|existing| existing == body.as_bytes()) {
            return Ok(());
        }

        if self.check {
            self.stale.push(relative.to_owned());
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, body).with_context(|| anyhow!("Writing {}", path.display()))?;
        println!("Wrote {relative}");
        Ok(())
    }
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| anyhow!("Reading {}", path.display()))
}
