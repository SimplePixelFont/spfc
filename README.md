# SimplePixelFont Compiler (SPFC)

A compiler toolchain Command Line Interface (CLI) for SimplePixelFonts (SPFs).

> [!IMPORTANT]
> SPFC is experimental software, expect bugs and limited functionality. Regardless, efforts are dedicated to bridging the gap between SPF and the standardized and traditional TrueType Fonts (TTF).

# Usage

This tool is written in Rust, you can bulld and run the project with the following pre-requisites:

* [Rust toolchain](https://www.rust-lang.org/tools/install)
* [Git](https://git-scm.com/install/) (Optional)

Begin by cloning the repository with the following command:
```bash
# Alternativly, download the repository.
git clone https://github.com/SimplePixelFont/spfc.git
# Change into the downloaded directory. ex.
cd spfc
```

Then build and install the binary with Cargo.
```bash
cargo install --path .
```

Finally, run `spfc`.
```bash
spfc --help
```
```
SimplePixelFont Compiler Toolchain

Usage: spfc [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>
          Input SimplePixelFont file path
  -o, --output <OUTPUT>
          Output TTF file path [default: output.ttf]
  -f, --family-name <FAMILY_NAME>
          Name of the font family [default: SimplePixelFont]
  -c, --copyright <COPYRIGHT>
          Description of the font copyright [default: "Copyright (c) 2026 SimplePixelFont"]
  -m, --manufacturer <MANUFACTURER>
          Name of the font manufacturer [default: SimplePixelFont]
  -v, --vendor-url <VENDOR_URL>
          URL of the font vendor [default: https://github.com/SimplePixelFont]
  -l, --license-description <LICENSE_DESCRIPTION>
          Description of the font license [default: "Licensed under the Apache License, Version 2.0; you may not use this file except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0"]
  -f, --family-version <FAMILY_VERSION>
          Version of the font family [default: 1]
  -p, --pixel-size <PIXEL_SIZE>
          Pixel size in font units [default: 64]
  -d, --decender-pixels <DECENDER_PIXELS>
          Decender size in pixels [default: 0]
  -h, --help
          Print help
  -V, --version
          Print version
```
