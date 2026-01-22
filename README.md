# SimplePixelFont Compiler (SPFC)

A compiler toolchain Command Line Interface (CLI) for SimplePixelFonts (SPFs).

> ![NOTE]
> SPFC is experimental software, expect bugs and limited functionality. Regardless, efforts are dedicated to bridging the gap between SPF and the standardized and traditional TrueType Fonts (TTF).

# Usage

This tool is written in Rust, you can bulld and run the project with the following pre-requisites:

* [Rust toolchain](https://www.rust-lang.org/tools/install)
* [Git](https://git-scm.com/install/) (Optional)

Begin by cloning the repository with the following command:
```bash
git clone
# Alternavtivly, download the repository and "cd" into the downloaded directory. ex.
# cd downloads/spfc
```

Then build and install the binary with Cargo.
```bash
cargo install --path .
```

Finally, run `spfc`.
```bash
spfc --help
```