# Configuración
$baseFolder = "D:\EstacionDeReferenciaFiUNAM\Testcors\RINEX"
$rustScript = "D:\EstacionDeReferenciaFiUNAM\renombramiento_CORS-master\rinexInegi.exe"
$logFile = "D:\EstacionDeReferenciaFiUNAM\renombramiento_CORS-master\rinex_monitor.log"
$stateFile = "D:\EstacionDeReferenciaFiUNAM\renombramiento_CORS-master\rinex_processor_state.json"

function Log-Message {
    param([string]$message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss.fff"
    "[$timestamp] $message" | Out-File $logFile -Append
    Write-Host "[$timestamp] $message"
}

function Get-UtcDOYFolders {
    $utcNow = Get-Date -AsUTC
    $yearToday = $utcNow.Year
    $yearYesterday = ($utcNow.AddDays(-1)).Year
    $doyToday = $utcNow.DayOfYear
    $doyYesterday = ($utcNow.AddDays(-1)).DayOfYear

    return @(
        Join-Path -Path $baseFolder -ChildPath "$yearYesterday\$doyYesterday\FICU\CHA0",
        Join-Path -Path $baseFolder -ChildPath "$yearToday\$doyToday\FICU\CHA0"
    )
}

function Process-New-Files {
    $folders = Get-UtcDOYFolders

    # Cargar estado anterior
    $processedFiles = @{}
    if (Test-Path $stateFile) {
        $jsonContent = Get-Content $stateFile -Raw
        if ($jsonContent) {
            ($jsonContent | ConvertFrom-Json).PSObject.Properties | ForEach-Object {
                $processedFiles[$_.Name] = $_.Value
            }
        }
    }

    foreach ($folder in $folders) {
        if (-not (Test-Path $folder)) {
            Log-Message "Carpeta no encontrada (aún): $folder"
            continue
        }

        $currentFiles = Get-ChildItem -Path $folder -Filter "FICU*.*" -File

        foreach ($file in $currentFiles) {
            if (-not $processedFiles.ContainsKey($file.FullName)) {
                try {
                    Log-Message "Procesando nuevo archivo: $($file.FullName)"
                    & $rustScript $folder | Out-File $logFile -Append
                    $processedFiles[$file.FullName] = $true
                    Log-Message "Completado: $($file.FullName)"
                }
                catch {
                    Log-Message "ERROR procesando $($file.FullName): $_"
                }
            }
        }
    }

    $processedFiles | ConvertTo-Json | Out-File $stateFile -Force
}

Log-Message "=== INICIO MONITOREO (UTC) ==="
Log-Message "Carpeta base: $baseFolder"

try {
    while ($true) {
        Process-New-Files
        Start-Sleep -Seconds 60
    }
}
finally {
    Log-Message "=== MONITOR DETENIDO ==="
}
