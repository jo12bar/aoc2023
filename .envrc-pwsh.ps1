# This file sets up environment variables for powershell users
# in case if you (like me) can't get direnv working properly
# on Windows.
#
# Setup these environment variables by dot-sourcing this file:
#
#    . .\.envrc-pwsh.ps1
#

$env:AOC2023_CONFIG="$(Get-Location)\.config"
$env:AOC2023_DATA="$(Get-Location)\.data"
$env:AOC2023_LOG_LEVEL="debug"
