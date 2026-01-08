//! SPC Converter CLI
//!
//! Convert Spectrum Analyzer Suite .spc files to JSON or CSV format.

use clap::{Parser, ValueEnum};
use spc_converter::{output, SpcFile};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "spc-convert")]
#[command(about = "Convert Spectrum Analyzer Suite .spc files to open formats")]
#[command(version)]
struct Cli {
    /// Input .spc file(s)
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output file path (for single input) or directory (for multiple inputs)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    format: OutputFormat,

    /// Pretty-print JSON output
    #[arg(short, long)]
    pretty: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Csv,
}

fn main() {
    let cli = Cli::parse();

    let mut success_count = 0;
    let mut error_count = 0;

    for input_path in &cli.input {
        if cli.verbose {
            eprintln!("Processing: {}", input_path.display());
        }

        match process_file(&cli, input_path) {
            Ok(output_path) => {
                success_count += 1;
                if cli.verbose {
                    eprintln!("  -> {}", output_path.display());
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Error processing {}: {}", input_path.display(), e);
            }
        }
    }

    if cli.input.len() > 1 {
        eprintln!(
            "\nProcessed {} file(s): {} success, {} errors",
            cli.input.len(),
            success_count,
            error_count
        );
    }

    if error_count > 0 {
        std::process::exit(1);
    }
}

fn process_file(cli: &Cli, input_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Parse the SPC file (now with calibration and config)
    let spc = SpcFile::from_file(input_path)?;

    if cli.verbose {
        eprintln!("  UID: {}", spc.uid);
        eprintln!("  Data points: {}", spc.data.len());
        eprintln!("  Blank points: {}", spc.blank.len());
        if let Some(ref cal) = spc.calibration {
            eprintln!("  Calibration: {:?}", cal.coefficients);
        }
        if let Some(ref cfg) = spc.config {
            if let Some(raman) = cfg.raman_wavelength {
                eprintln!("  Raman wavelength: {} nm", raman);
            }
        }
        if spc.has_raman_shift() {
            eprintln!("  Raman shift axis: available");
        } else if spc.has_calibration() {
            eprintln!("  Wavelength axis: available");
        }
    }

    // Determine output path
    let output_path = get_output_path(cli, input_path);

    // Write output
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    match cli.format {
        OutputFormat::Json => {
            output::write_json_spc(&spc, &mut writer, cli.pretty)?;
        }
        OutputFormat::Csv => {
            output::write_csv_spc(&spc, &mut writer)?;
        }
    }

    writer.flush()?;

    Ok(output_path)
}

fn get_output_path(cli: &Cli, input_path: &PathBuf) -> PathBuf {
    let extension = match cli.format {
        OutputFormat::Json => "json",
        OutputFormat::Csv => "csv",
    };

    if let Some(ref output) = cli.output {
        if cli.input.len() == 1 {
            // Single file: use output as-is if it has an extension, otherwise add one
            if output.extension().is_some() {
                output.clone()
            } else {
                output.with_extension(extension)
            }
        } else {
            // Multiple files: output is a directory
            let filename = input_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            output.join(format!("{}.{}", filename, extension))
        }
    } else {
        // No output specified: create alongside input
        input_path.with_extension(extension)
    }
}
