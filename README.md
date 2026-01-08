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

### Batch Processing
Convert multiple files at once:
```bash
spc-convert -p data/*.spc
```

### Full Options
```
Usage: spc-convert [OPTIONS] <INPUT>...

Arguments:
  <INPUT>...  Input .spc file(s)

Options:
  -o, --output <OUTPUT>  Output file path (for single input) or directory
  -f, --format <FORMAT>  Output format [default: json] [possible values: json, csv]
  -p, --pretty           Pretty-print JSON output
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

## Specification

For a deep dive into the binary format internals, see [spc.md](spc.md).

## Roadmap / Known Issues

- [ ] **Calibration polynomial**: The current implementation uses a standard polynomial expansion (`c[0] + c[1]*x + c[2]*x² + c[3]*x³`) for wavelength calculation. However, the original Spectrum Analyzer Suite uses **Legendre polynomial** basis functions (`Σ aₖPₖ(x)` where `P₀=1`, `P₁=x`, `P₂=½(3x²-1)`, `P₃=½(5x³-3x)`). This will produce incorrect wavelength/Raman shift axes. See [spc.md § 3.2](spc.md#32-buffer-calibration-optional) for the correct formula.
- [ ] **Config fields**: Only `raman_wavelength`, `exposure`, `gain`, and `smoothing` are extracted. Additional fields documented in the spec (`average`, `sgolay_*`, `medfilt`, `baseline`, `axis`) are not yet parsed.

## Third-Party Licenses

The file format specification in [spc.md](spc.md) was derived from the [Spectrum Analyzer Suite](https://www.open-raman.org/build/software/) source code, which is licensed under the **CERN Open Hardware Licence Version 2 - Weakly Reciprocal (CERN-OHL-W v2)**.

See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for the full license text.
