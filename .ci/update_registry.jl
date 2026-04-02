using Pkg
Pkg.add("JSON")
using JSON

target_name = ENV["TARGET_NAME"]
version = ENV["VERSION"]
registry_path = "registry.json"

println("Updating registry for $target_name v$version...")

if isfile(registry_path)
    data = JSON.parsefile(registry_path)
else
    data = Dict("targets" => Dict())
end

targets = data["targets"]
if !haskey(targets, target_name)
    targets[target_name] = Dict(
        "description" => "Plugin target for $target_name",
        "repo" => "SimplePixelFont/spfc",
        "versions" => String[],
        "latest" => ""
    )
end

if !(version in targets[target_name]["versions"])
    push!(targets[target_name]["versions"], version)
    sort!(targets[target_name]["versions"])
end
targets[target_name]["latest"] = version

open(registry_path, "w") do f
    JSON.print(f, data, 2)
end

try
    run(`git config user.name "github-actions[bot]"`)
    run(`git config user.email "github-actions[bot]@users.noreply.github.com"`)
    run(`git add $registry_path`)
    run(`git commit -m "chore: update registry for $target_name v$version"`)
    run(`git push origin main`)
    println("✅ Registry successfully updated and pushed.")
catch e
    println("⚠️ Could not push registry update. This might be due to branch protection or no changes.")
end