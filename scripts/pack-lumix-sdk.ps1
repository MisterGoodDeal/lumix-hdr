# Packe vendor/lumix-sdk/Library → ci/lumix-sdk-library.zip.enc (AES-256-CBC + PBKDF2).
# Format binaire : magic "LHDR1" + salt(16) + IV(16) + ciphertext
# Secret GitHub : LUMIX_SDK_PASSPHRASE (court — le zip brut dépasse la limite 64 Ko).
#
# Usage: .\scripts\pack-lumix-sdk.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

. (Join-Path $PSScriptRoot "lumix-sdk-crypto.ps1")

$library = Join-Path $root "vendor\lumix-sdk\Library"
if (-not (Test-Path (Join-Path $library "Lmxptpif.dll"))) {
    throw "vendor/lumix-sdk/Library/Lmxptpif.dll introuvable"
}
if (-not (Test-Path (Join-Path $library "Lmxptpif.lib"))) {
    throw "vendor/lumix-sdk/Library/Lmxptpif.lib introuvable"
}

$secretsDir = Join-Path $root ".secrets"
New-Item -ItemType Directory -Force -Path $secretsDir | Out-Null
$passFile = Join-Path $secretsDir "lumix-sdk.passphrase"

$passphrase = $env:LUMIX_SDK_PASSPHRASE
if ([string]::IsNullOrWhiteSpace($passphrase) -and (Test-Path $passFile)) {
    $passphrase = (Get-Content $passFile -Raw).Trim()
}
if ([string]::IsNullOrWhiteSpace($passphrase)) {
    $bytes = New-Object byte[] 24
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $passphrase = [Convert]::ToBase64String($bytes)
    Set-Content -Path $passFile -Value $passphrase -NoNewline
    Write-Host "Passphrase générée → .secrets/lumix-sdk.passphrase"
}

$ciDir = Join-Path $root "ci"
New-Item -ItemType Directory -Force -Path $ciDir | Out-Null
$zip = Join-Path $env:TEMP "lumix-sdk-library.zip"
$enc = Join-Path $ciDir "lumix-sdk-library.zip.enc"

if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $library "*") -DestinationPath $zip -Force

Protect-LumixSdkZip -ZipPath $zip -EncPath $enc -Passphrase $passphrase
Remove-Item $zip -Force

Write-Host "ok → ci/lumix-sdk-library.zip.enc ($([math]::Round((Get-Item $enc).Length / 1KB)) Ko)"
Write-Host "Committe le .enc, colle .secrets/lumix-sdk.passphrase dans le secret LUMIX_SDK_PASSPHRASE."
