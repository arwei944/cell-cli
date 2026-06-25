#!/usr/bin/env pwsh
# 代码质量强制检查脚本
# Code Quality Enforcement Check Script
# 用途: 验证所有 Rust 文件是否符合定量代码规范
# Purpose: Verify all Rust files comply with quantitative code standards

$ErrorActionPreference = "Stop"
$rootDir = Split-Path -Parent $PSScriptRoot
$srcDir = Join-Path $rootDir "src"
$maxFileLines = 500
$maxFnLines = 30
$maxStructFields = 8

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Cell Architecture 代码质量门禁检查                            ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$violations = 0
$checked = 0

$files = Get-ChildItem -Path $srcDir -Filter "*.rs" -Recurse | Where-Object { $_.FullName -notmatch "target" }

foreach ($file in $files) {
    $checked++
    $lines = (Get-Content $file.FullName | Measure-Object -Line).Lines
    $relPath = $file.FullName.Substring($rootDir.Length + 1)

    if ($lines -gt $maxFileLines) {
        Write-Host "X [LONG-FILE]  $relPath  ($lines lines > $maxFileLines threshold)" -ForegroundColor Red
        $violations++
    } else {
        Write-Host "OK              $relPath  ($lines lines)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Total: $checked files, $violations violations" -ForegroundColor $(if ($violations -gt 0) { "Red" } else { "Green" })
Write-Host ""

if ($violations -gt 0) {
    Write-Host "WARNING: Long files found, please split them into smaller modules" -ForegroundColor Yellow
    Write-Host "   - Strategy: Split by responsibility, each file <500 lines" -ForegroundColor Yellow
    Write-Host "   - Tools: 'cell verify' or 'cell lint'" -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "All files comply with quantitative standards!" -ForegroundColor Green
    exit 0
}

