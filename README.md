# spc-converter

`spc-converter` is a simple CLI tool for converting proprietary `.spc` files from the [Spectrum Analyzer Suite](https://www.open-raman.org/build/software/) into open, machine-readable formats like JSON and CSV.

Unlocks your spectral data by extracting:
- **Raw Intensity Data**: The primary spectral measurements.
- **Reference Data**: Blank/dark spectrum readings.
- **Calibration Settings**: Wavelength calibration coefficients (polynomial weights).
- **Configuration**: Metadata such as Raman laser wavelength, exposure, gain, and smoothing.
- **Calculated Axes**: Wavelengths (nm) and Raman shifts (cm⁻¹), if calibration is present.

## Usage

You can run the tool using the compiled binary `spc-convert`.

### Basic Conversion
Convert a single file to JSON (default):
```bash
spc-convert path/to/spectrum.spc
```
This will create `path/to/spectrum.json`.

### Pretty-Print JSON
For human-readable JSON output:
```bash
spc-convert -p path/to/spectrum.spc
```

### Format Selection
Convert to CSV:
```bash
spc-convert -f csv path/to/spectrum.spc
```

Convert to LLM-friendly pairs format:
```bash
spc-convert -f pairs path/to/spectrum.spc
```

### Batch Processing
Convert multiple files at once:
```bash
spc-convert -p data/*.spc
```

### Generate Spectrum Plots
Generate a PNG visualization alongside the output:
```bash
spc-convert --plot path/to/spectrum.spc
```
This creates both `spectrum.json` and `spectrum.png`.

### Full Options
```
Usage: spc-convert [OPTIONS] <INPUT>...

Arguments:
  <INPUT>...  Input .spc file(s)

Options:
  -o, --output <OUTPUT>  Output file path (for single input) or directory
  -f, --format <FORMAT>  Output format [default: json] [possible values: json, csv, pairs]
  -p, --pretty           Pretty-print JSON output
      --plot             Generate PNG plot(s) of the spectrum
  -v, --verbose          Show verbose output
  -h, --help             Print help
  -V, --version          Print version
```

## Output Format (JSON)
The JSON output contains all extracted and computed data:

```json
{
  "uid": "Camera-123",
  "data": [100.0, 150.5, ...],
  "blank": [5.0, 5.2, ...],
  "calibration": {
    "coefficients": [532.0, 100.0, 0.5, 0.01]
  },
  "config": {
    "raman_wavelength": 785.0,
    "exposure": 1000.0,
    "gain": 1.0,
    "smoothing": 5
  },
  "wavelength_axis": [400.0, 400.5, ...],
  "raman_shift_axis": [0.0, 10.5, ...]
}
```

Note: Fields like `calibration`, `config`, `wavelength_axis`, and `raman_shift_axis` are omitted from the output if not present in the source file.

## Output Format (CSV)
The CSV output provides tabular data suitable for spreadsheets and data analysis tools. Columns are dynamically included based on available calibration data:

```csv
index,wavelength_nm,raman_shift_cm-1,intensity,blank
0,545.045,449.899,0.4488,0.12
1,545.101,451.764,0.4563,0.11
2,545.156,453.628,0.4686,0.13
...
```

**Column behavior:**
- `index`: Always present (0-based pixel index)
- `wavelength_nm`: Included if wavelength calibration is available
- `raman_shift_cm-1`: Included if both wavelength and Raman laser wavelength are configured
- `intensity`: Always present (spectral intensity values)
- `blank`: Included if blank/reference data exists

## Output Format (Pairs)
The pairs format is optimized for LLM consumption, with a minimal header and x,y value pairs:

```
# Raman Spectrum
# X-axis: Raman Shift (cm⁻¹), Y-axis: Intensity
# Laser: 785nm, Points: 2048

176.5, 1024.3
180.2, 1089.7
...
```

The x-axis automatically uses Raman shift if available, otherwise wavelength, or pixel index as fallback.

## Plotting
The `--plot` option generates PNG visualizations of the spectrum data. The plot automatically selects the most appropriate x-axis:

1. **Raman Shift (cm⁻¹)**: Used when Raman calibration is available. The axis is reversed (high to low) following spectroscopy convention.
2. **Wavelength (nm)**: Used when wavelength calibration is present but no Raman data.
3. **Pixel Index**: Fallback when no calibration data exists.

```bash
# Generate plot with default JSON output
spc-convert --plot spectrum.spc

# Generate plot with CSV output
spc-convert --plot -f csv spectrum.spc
```

Plots are saved as PNG files with the same base name as the input (e.g., `spectrum.png`).

## Specification

For a deep dive into the binary format internals, see [spc.md](spc.md).

## Third-Party Licenses

The file format specification in [spc.md](spc.md) was derived from the [Spectrum Analyzer Suite](https://www.open-raman.org/build/software/) source code, which is licensed under the **CERN Open Hardware Licence Version 2 - Weakly Reciprocal (CERN-OHL-W v2)**.

See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for the full license text.
