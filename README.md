# epc

A command-line tool for converting epoch timestamps.

## Releasing

The release workflow runs when a tag matching `v*` is pushed. It builds and
attaches Linux (x86_64 and ARM64) and macOS (ARM64) archives to the GitHub
release, then notifies the Homebrew tap.

Use the following process to cut a release. Replace `0.1.3` with the intended
next semantic version.

```bash
version=0.1.3

# Start from a clean working copy and check the current version.
jj status
jj diff -r @
cargo metadata --no-deps --format-version 1 \
  | python3 -c 'import json, sys; print(json.load(sys.stdin)["packages"][0]["version"])'

# Describe the release change before editing, then update Cargo.toml.
jj describe -m "bump version to $version"
# Change: version = "$version" in Cargo.toml

# Validate the package and refresh the root package entry in Cargo.lock.
cargo test --all-targets
cargo package --allow-dirty
jj diff -r @

# Publish the version commit on main.
jj bookmark set main -r @
jj git push --bookmark main

# Create and publish the GitHub release; gh creates the v<version> tag at main.
gh release create "v$version" --target main --title "v$version" --generate-notes
```

### Verify the release

Wait for the tag-triggered `release` workflow and require success:

```bash
run_id=$(gh run list --workflow release.yml --event push --branch "v$version" \
  --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$run_id" --exit-status

gh release view "v$version" --json url,tagName,isDraft,isPrerelease,assets
```

The workflow is complete only when all three target builds and the `bump
homebrew tap` job succeed, and the release is published (not draft or
prerelease) with these assets:

- `epc-x86_64-unknown-linux-gnu.tar.gz`
- `epc-aarch64-unknown-linux-gnu.tar.gz`
- `epc-aarch64-apple-darwin.tar.gz`

Finally, confirm the tag points at the published `main` commit and leave the
working copy clean:

```bash
gh api "repos/dfontana/epc/git/ref/tags/v$version" --jq '.object.sha'
jj log -r main -n 1
jj status
```
