$tag = $args[0]
if (-not $tag) {
  Write-Host "Usage: pwsh update-hashes.ps1 <tag>"
  Write-Host "  pwsh update-hashes.ps1 v1.0.0"
  exit 1
}

$repo = "Elephant-on-github/tldscan"
$artifacts = @{
  windows = "tldscan-x86_64-pc-windows-msvc.zip"
  macos   = "tldscan-x86_64-apple-darwin.tar.gz"
  linux   = "tldscan-x86_64-unknown-linux-gnu.tar.gz"
}

Push-Location (Join-Path $PSScriptRoot "..")

foreach ($key in $artifacts.Keys) {
  $file = $artifacts[$key]
  $url = "https://github.com/$repo/releases/download/$tag/$file"
  Write-Host "Downloading $url ..."
  Invoke-WebRequest -Uri $url -OutFile $file
}

foreach ($key in $artifacts.Keys) {
  $file = $artifacts[$key]
  $hash = (Get-FileHash $file -Algorithm SHA256).Hash.ToLower()
  Write-Host "$file : $hash"
  $scoopPath = "packaging/scoop/tldscan.json"
  $brewPath = "packaging/homebrew/tldscan.rb"

  if ($key -eq "windows") {
    (Get-Content $scoopPath) -replace '"hash": ".*"', "`"hash`": `"$hash`"" | Set-Content $scoopPath
  } elseif ($key -eq "macos") {
    (Get-Content $brewPath) -replace '(on_macos.*\n.*sha256 ").*(")', "`${1}$hash`$2" | Set-Content $brewPath
  } elseif ($key -eq "linux") {
    (Get-Content $brewPath) -replace '(on_linux.*\n.*sha256 ").*(")', "`${1}$hash`$2" | Set-Content $brewPath
  }
  Remove-Item $file
}

Pop-Location
Write-Host "Hashes updated in packaging/scoop/tldscan.json and packaging/homebrew/tldscan.rb"
