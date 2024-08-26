param (
    [switch]$local
)

# Define paths and ensure Cargo.toml exists
$cargoTomlPath = "./Cargo.toml"
if (-Not (Test-Path $cargoTomlPath)) {
    Write-Output "❌ Cargo.toml not found at path: $cargoTomlPath"
    Write-Output "Please ensure the script is run from the root directory of your Rust project."
    exit 1
}

# Increment version in Cargo.toml
$cargoTomlContent = Get-Content -Path $cargoTomlPath -Raw
$versionPattern = 'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"'

if ($cargoTomlContent -match $versionPattern) {
    $major = $matches[1]
    $minor = $matches[2]
    $patch = [int]$matches[3] + 1
    $newVersion = "$major.$minor.$patch"
    
    $newCargoTomlContent = $cargoTomlContent -replace $versionPattern, "version = `"$newVersion`""
    Set-Content -Path $cargoTomlPath -Value $newCargoTomlContent
    Write-Output "✅ Updated version to $newVersion in Cargo.toml"
} else {
    Write-Output "❌ Version line not found in Cargo.toml"
    exit 1
}

# Prepare Git commit and tag
$publishDate = Get-Date -Format "yyyy-MM-dd"
$commitMessage = if ($local) { "🔧 Bump version to $newVersion ($publishDate)" } else { "🚀 Bump version to $newVersion ($publishDate) and release 📦" }
$releaseMessage = "Release v$newVersion ($publishDate)"

# Build in release mode
Write-Output "🔨 Building the crate in release mode..."
cargo build --release

# Prepare Git operations
git add .
git commit -m "$commitMessage"
git tag -a "v$newVersion" -m "$releaseMessage"

if ($local) {
    Write-Output "🏠 Running in local mode, skipping publishing to crates.io."
} else {
    Write-Output "🎉 Pushing changes and tags to the repository..."
    git push && git push --tags

    $cargoToken = $env:CARGO_TOKEN
    if ($cargoToken) {
        Write-Output "📦 Publishing package to crates.io..."
        cargo publish
        if ($LASTEXITCODE -eq 0) {
            Write-Output "✨ Package successfully published to crates.io!"
        } else {
            Write-Output "❌ Failed to publish package to crates.io. Check output for details."
        }
    } else {
        Write-Output "⚠️ CARGO_TOKEN not found in environment variables. Skipping publishing to crates.io."
    }
}

Write-Output "🎉 Release v$newVersion completed!"
