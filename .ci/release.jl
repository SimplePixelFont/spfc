### Version Number & Component ###
tag_name = ENV["TAG_NAME"]
component = ENV["COMPONENT"]
sha = ENV["SHA"]

println("Building component: $component for tag: $tag_name")

### BinaryBuilder Builds ###
jl_platforms = [
    "'i686-linux-gnu'"
    "'x86_64-linux-gnu'"
    "'aarch64-linux-gnu'"
    "'armv7l-linux-gnueabihf'"
    "'x86_64-linux-musl'"
    "'aarch64-linux-musl'"
    "'x86_64-apple-darwin'"
    "'aarch64-apple-darwin'"
    "'x86_64-unknown-freebsd'"
    "'x86_64-w64-mingw32'"
]

ENV["VERSION"] = "0.0.1"

for plat in jl_platforms
    try
        run(`julia .ci/build_tarballs.jl $plat`)
        
        plat_clean = replace(plat, "'" => "")
        
        mkpath("artifacts")
        for file in readdir("products")
            if endswith(file, ".tar.gz")
                # Rename e.g., spfc.v0.0.1.x86_64-linux-gnu.tar.gz -> spfc-target-ttf.target-ttf-v1.0.0.x86_64-linux-gnu.tar.gz
                new_name = "$component.$tag_name.$plat_clean.tar.gz"
                mv(joinpath("products", file), joinpath("artifacts", new_name), force=true)
            end
        end
    catch e
        println("Failed to build for $plat. Skipping.")
    end
end