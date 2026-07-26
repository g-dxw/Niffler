param(
  [int]$AppPort = 8084,
  [switch]$SkipInfra
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not (Test-Path '.env')) {
  if (Test-Path '.env.example') {
    Copy-Item '.env.example' '.env'
    Write-Host 'Created .env from .env.example. Please review it.'
  } else {
    throw 'Missing .env and .env.example.'
  }
}

Get-Content '.env' | ForEach-Object {
  $line = $_.Trim()
  if ($line -and -not $line.StartsWith('#') -and $line -match '^\s*([^=]+?)\s*=\s*(.*)\s*$') {
    [Environment]::SetEnvironmentVariable($matches[1].Trim(), $matches[2].Trim().Trim('"').Trim("'"), 'Process')
  }
}
$env:APP_PORT = $AppPort
$env:RUST_LOG = if ($env:RUST_LOG) { $env:RUST_LOG } else { 'aether_gateway=info' }

if (-not $SkipInfra) {
  if (-not (Get-Command docker -ErrorAction SilentlyContinue)) { throw 'Docker is required. Start Docker Desktop or use -SkipInfra.' }
  docker compose up -d postgres redis
  if ($LASTEXITCODE -ne 0) { throw 'Failed to start Postgres/Redis.' }
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'cargo is required. Install the Rust toolchain.' }
if (-not (Get-Command npm.cmd -ErrorAction SilentlyContinue)) { throw 'npm.cmd is required. Install Node.js.' }

$backend = Start-Process -FilePath 'cargo' -ArgumentList @('run', '-p', 'aether-gateway', '--', '--app-port', $AppPort) -WorkingDirectory $repoRoot -PassThru -NoNewWindow
try {
  $healthUrl = "http://127.0.0.1:$AppPort/_gateway/health"
  for ($i = 0; $i -lt 60; $i++) {
    Start-Sleep -Seconds 1
    try { Invoke-WebRequest -UseBasicParsing -Uri $healthUrl -TimeoutSec 2 | Out-Null; break }
    catch { if ($backend.HasExited) { throw 'Rust gateway failed to start. Check cargo output.' } }
  }
  $frontend = Start-Process -FilePath 'npm.cmd' -ArgumentList @('run', 'dev') -WorkingDirectory (Join-Path $repoRoot 'frontend') -PassThru -NoNewWindow
  try {
    Write-Host "Dev environment started: frontend http://127.0.0.1:5173, backend http://127.0.0.1:$AppPort"
    Wait-Process -Id $frontend.Id
  } finally { if (-not $frontend.HasExited) { Stop-Process -Id $frontend.Id -Force } }
} finally { if (-not $backend.HasExited) { Stop-Process -Id $backend.Id -Force } }
