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
    $doyToday = $utcNow.DayOfYear

    return @(
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
function Start-Watcher {
    param (
        [string]$folderPath
    )

    Log-Message "Iniciando watcher para: $folderPath"

    $watcher = New-Object System.IO.FileSystemWatcher
    $watcher.Path = (Resolve-Path $folderPath).Path
    $watcher.Filter = "FICU*.*"
    $watcher.IncludeSubdirectories = $false
    $watcher.EnableRaisingEvents = $true

    return Register-ObjectEvent -InputObject $watcher -EventName Changed -Action {
        Start-Sleep -Seconds 90  # Esperar copia completa desde la estación
        try {
            Process-New-Files
        }
        catch {
            Log-Message "ERROR al procesar archivos: $_"
        }
    }
}

# Inicialización
$lastUtcDate = (Get-Date -AsUTC).Date
$folder = Get-UtcDOYFolders

if (-not (Test-Path $folder)) {
    Log-Message "Carpeta del día actual no encontrada: $folder"
    exit 1
}

# Iniciar watcher inicial
$global:watcherSubscription = Start-Watcher -folderPath $folder

Write-Host "Monitoreando: $folder"
Write-Host "Presiona Ctrl+C para salir."

# Loop para detectar cambio de día UTC y reiniciar watcher
while ($true) {
    Start-Sleep -Seconds 60

    $currentUtcDate = (Get-Date -AsUTC).Date
    if ($currentUtcDate -ne $lastUtcDate) {
        # Cambió el día UTC
        Log-Message "Cambio detectado de día UTC. Reiniciando watcher..."

        # Actualizar fecha y carpeta
        $lastUtcDate = $currentUtcDate
        $folder = Get-UtcDOYFolders

        if (Test-Path $folder) {
            # Detener watcher anterior
            Unregister-Event -SourceIdentifier $watcherSubscription.Name -ErrorAction SilentlyContinue
            $watcherSubscription = Start-Watcher -folderPath $folder
            Write-Host "Watcher activo en: $folder"
        } else {
            Log-Message "Carpeta aún no disponible: $folder"
        }
    }
}
