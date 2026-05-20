# SimplePixelFont Compiler (SPFC)

A compiler toolchain Command Line Interface (CLI) for SimplePixelFonts (SPFs).

`spfc` is dedicated to bridging the gap between SPF and multiple different font formats such as; the standardized and traditional TrueType Fonts (TTF), and a dedicated PICO8 custom font cartridge format (P8).

# Installing

Pre-built binaries for common platforms like Windows, MacOS, and Linux are provided in the [releases](https://github.com/SimplePixelFont/spfc/releases?q=cli&expanded=true). You can download the appropriate archive, extract it, and execute the binary found in the folder.

# Building

`spfc` is written in Rust, you can build and run the project with the following pre-requisites:

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

# Usage

```bash
spfc --help
```
```
SimplePixelFont Compiler Toolchain

Usage: spfc <COMMAND>

Commands:
  compile          
  install          Install a new target backend
  list             List all available targets from the registry
  update-registry  Update the plugin registry by fetching the latest information from the SimplePixelFont/spfc GitHub repository
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
