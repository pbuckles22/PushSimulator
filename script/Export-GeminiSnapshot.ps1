<#
.SYNOPSIS
  Export this repo to Gemini_Snapshot.txt with XML file tags.

.DESCRIPTION
  Each source file is wrapped as:
    <file path="src/Main.cs"><![CDATA[ ... ]]></file>
  Build folders, binaries, secrets, and the Gemini drop itself are skipped.
  The dump is gitignored. Focused packs still use doc/gemini/.

.EXAMPLE
  .\script\Export-GeminiSnapshot.ps1
  .\script\Export-GeminiSnapshot.ps1 -OutputFileName "Gemini_Snapshot_review.txt" -Label review
#>
[CmdletBinding()]
param(
    [string]$Root = "",
    [string]$OutputFileName = "Gemini_Snapshot.txt",
    [string]$Label = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$root = if ([string]::IsNullOrWhiteSpace($Root)) { Split-Path -Parent $PSScriptRoot } else { $Root }
if ([string]::IsNullOrWhiteSpace($root)) {
    $root = (Get-Location).Path
}
$root = [System.IO.Path]::GetFullPath($root)
$outputFilePath = if ([System.IO.Path]::IsPathRooted($OutputFileName)) {
    $OutputFileName
} else {
    Join-Path $root $OutputFileName
}

$excludeDirNames = @(
    ".git", ".vs", ".idea", ".vscode",
    "bin", "obj", "Library", "Temp", "Logs",
    "Build", "Builds", "build", "dist", "out", "coverage",
    "node_modules", "Packages", "Managed",
    "gemini", "dropzone"
)

$excludeExts = @(
    ".dll", ".exe", ".pdb", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico",
    ".mp4", ".wav", ".zip", ".7z", ".rar", ".cache", ".user", ".suo", ".binlog",
    ".pem", ".pfx", ".key"
)

$excludeFileNames = @(
    [System.IO.Path]::GetFileName($outputFilePath),
    "Gemini_Snapshot.txt",
    "Directory.Build.targets",
    "build_number.txt"
)

function Test-InExcludedDirectory {
    param([string]$FullName)
    $rel = $FullName.Substring($root.Length).TrimStart("\", "/")
    $parts = $rel.Split([char[]]@("\", "/"))
    if ($parts.Length -lt 2) {
        return $false
    }
    foreach ($part in $parts[0..($parts.Length - 2)]) {
        if ($excludeDirNames -contains $part) {
            return $true
        }
    }
    return $false
}

function Test-SensitiveName {
    param([string]$Name)
    if ($Name -eq ".env" -or $Name -like ".env.*") { return $true }
    if ($Name -like "*credentials*") { return $true }
    if ($Name -like "Gemini_Snapshot*.txt") { return $true }
    return $false
}

function Test-LooksBinary {
    param([string]$Path)
    try {
        $fs = [System.IO.File]::Open($Path, "Open", "Read", "ReadWrite")
        try {
            $buf = New-Object byte[] 512
            $n = $fs.Read($buf, 0, $buf.Length)
            for ($i = 0; $i -lt $n; $i++) {
                if ($buf[$i] -eq 0) { return $true }
            }
        }
        finally { $fs.Dispose() }
    }
    catch { return $true }
    return $false
}

Write-Host "Scanning repository for Gemini export..."

$allFiles = @(Get-ChildItem -LiteralPath $root -Recurse -File -Force | Where-Object {
    $excludeFileNames -notcontains $_.Name -and
    -not (Test-SensitiveName $_.Name) -and
    $_.Name -notmatch '^\d{4}-handoff-' -and
    $_.Name -notmatch 'HANDOFF-' -and
    -not (Test-InExcludedDirectory $_.FullName) -and
    ($excludeExts -notcontains $_.Extension.ToLowerInvariant()) -and
    -not (Test-LooksBinary $_.FullName)
} | Sort-Object FullName)

$utf8 = New-Object System.Text.UTF8Encoding $false
$sb = New-Object System.Text.StringBuilder
$dateStr = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$labelAttr = if ([string]::IsNullOrWhiteSpace($Label)) { "" } else { " label=`"$Label`"" }
[void]$sb.AppendLine("<repository_snapshot generated=`"$dateStr`" repo=`"$root`"$labelAttr>")
[void]$sb.AppendLine("<directory_structure>")
foreach ($file in $allFiles) {
    $rel = $file.FullName.Substring($root.Length).TrimStart("\", "/").Replace("\", "/")
    [void]$sb.AppendLine($rel)
}
[void]$sb.AppendLine("</directory_structure>")
[void]$sb.AppendLine()
[void]$sb.AppendLine("<files>")

Write-Host "Appending $($allFiles.Count) files..."
foreach ($file in $allFiles) {
    $rel = $file.FullName.Substring($root.Length).TrimStart("\", "/").Replace("\", "/")
    [void]$sb.AppendLine("<file path=`"$rel`">")
    try {
        $content = [System.IO.File]::ReadAllText($file.FullName)
        $content = $content.Replace("]]>", "]]]]><![CDATA[>")
        [void]$sb.AppendLine("<![CDATA[")
        [void]$sb.AppendLine($content)
        [void]$sb.AppendLine("]]>")
    }
    catch {
        [void]$sb.AppendLine("<!-- Error reading file: $($_.Exception.Message) -->")
    }
    [void]$sb.AppendLine("</file>")
}

[void]$sb.AppendLine("</files>")
[void]$sb.AppendLine("</repository_snapshot>")
[System.IO.File]::WriteAllText($outputFilePath, $sb.ToString(), $utf8)

Write-Host "Done. $($allFiles.Count) files -> $outputFilePath"
Write-Host "Upload that .txt. It is gitignored."
