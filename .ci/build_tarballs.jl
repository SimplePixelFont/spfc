using Pkg
Pkg.instantiate()
try
    using BinaryBuilder
catch
    Pkg.add("BinaryBuilder")
    using BinaryBuilder
end

component = ENV["COMPONENT"]
# Rust replaces hyphens with underscores in compiled library names
crate_name = replace(component, "-" => "_")

name = component
version = VersionNumber(ENV["VERSION"])
sha = ENV["SHA"]

# Point this to your new SPFC repository
sources = [
    GitSource("https://github.com/SimplePixelFont/spfc.git", sha)
]

script = """
COMPONENT="$(component)"
CRATE_NAME="$(crate_name)"
""" * raw"""
cd $WORKSPACE/srcdir/spfc
mkdir target

# If we are building the CLI tool, it produces an executable
if [[ "$COMPONENT" == "spfc" ]]; then
    cargo build -p ${COMPONENT} --release
    install -D -m 755 "target/${rust_target}/release/spfc${exeext}" "${bindir}/spfc${exeext}"

# If we are building a target plugin, it produces a dynamic library (cdylib)
else
    cargo rustc -p ${COMPONENT} --release --crate-type cdylib
    
    # Windows typically generates .dll without a 'lib' prefix
    if [[ "${rust_target}" == *"windows"* ]]; then
        install -D -m 755 "target/${rust_target}/release/${CRATE_NAME}.${dlext}" "${libdir}/${CRATE_NAME}.${dlext}"
    else
        install -D -m 755 "target/${rust_target}/release/lib${CRATE_NAME}.${dlext}" "${libdir}/lib${CRATE_NAME}.${dlext}"
    fi
fi

install_license LICENSE
"""

platforms = [
    Platform("armv7l", "linux"; call_abi="eabihf", libc="glibc"),
    Platform("i686", "linux"; libc="glibc"),
    Platform("x86_64", "macos";),
    Platform("aarch64", "macos";),
    Platform("x86_64", "linux"; libc="glibc"),
    Platform("aarch64", "linux"; libc="musl"),
    Platform("x86_64", "linux"; libc="musl"),
    Platform("x86_64", "freebsd";),
    Platform("x86_64", "windows";),
]

if component == "spfc"
    products = [ExecutableProduct("spfc", :spfc)]
else
    products = [LibraryProduct(crate_name, Symbol(crate_name))]
end

dependencies = Dependency[
    Dependency(PackageSpec(name="OpenSSL_jll", uuid="458c3c95-2e84-50aa-8efc-19380b2a3a95"))
]

build_tarballs(ARGS, name, version, sources, script, platforms, products, dependencies; julia_compat="1.6", compilers=[:rust, :c])