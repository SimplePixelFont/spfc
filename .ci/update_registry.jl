# .ci/update_registry.jl
# Updates registry.json with a new target version and pushes to main.
# Called by builder.yml only when type == 'target'.
#
# Required env vars:
#   TARGET_NAME — e.g. "ttf"
#   VERSION     — e.g. "1.0.0"

using Pkg
Pkg.add("JSON"; io=devnull)
using JSON

target_name = ENV["TARGET_NAME"]
version     = ENV["VERSION"]
registry_path = "registry.json"

println("Updating registry: target=$target_name version=$version")

data = isfile(registry_path) ? JSON.parsefile(registry_path) : Dict("targets" => Dict())

targets = data["targets"]
if !haskey(targets, target_name)
    targets[target_name] = Dict(
        "description" => "Plugin target for $target_name",
        "repo"        => "SimplePixelFont/spfc",
        "versions"    => String[],
        "latest"      => ""
    )
end

entry = targets[target_name]
if !(version in entry["versions"])
    push!(entry["versions"], version)
    sort!(entry["versions"])
end
entry["latest"] = version

open(registry_path, "w") do f
    JSON.print(f, data, 2)
end

println("Registry written.")

try
    run(`git config user.name  "github-actions[bot]"`)
    run(`git config user.email "github-actions[bot]@users.noreply.github.com"`)
    run(`git switch -c chore/update-registry`)
    run(`git add $registry_path`)
    run(`git commit -m "update registry for $target_name v$version"`)
    run(`git push -u origin chore/update-registry`)
    # Capture the PR number from the create output
    pr_output = read(`gh pr create --title "Update registry for $target_name v$version" --json number -q .number`, String)
    pr_number = strip(pr_output)
    
    # Enable auto-merge with the PR number
    run(`gh pr merge $pr_number --auto --merge`)
    println("✅ Registry pushed.")
catch e
    println("⚠️  Could not push registry: $e")
end