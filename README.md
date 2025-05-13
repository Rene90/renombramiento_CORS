# 🛰️ Renombramiento automatico de archivos RINEX

Este proyecto contiene dos scripts para automatizar el renombramiento de archivos RINEX (observación y navegación) provenientes de la estación GNSS CORS South S9 de la cúpula del edificio A de la Facultad de Ingenieria en Ciudad Universitario. El renombramiento de archivos generados por la estación es un requisito para cumplir con las normas del INEGI y que esta estación al cumplir los lineamientos, sea incorporada a la Red Geodesica Nacional Activa .  

El sistema detecta archivos nuevos, los clasifica, ejecuta el modulo de RTKLIB`convbin` para agregar los campos de OBSERVER/AGENCY, los renombra y los comprime automáticamente descartando el archivo de navegación de la constelación Beidou.

---

## 📁 Estructura del repositorio

```
├── monitor_rinex.ps1        # Script de monitoreo en PowerShell
├── src/
│   └── main.rs              # Procesador de archivos RINEX (Rust)
├── rinex_processor.exe      # Ejecutable generado de main.rs
├── rinex_monitor.log        # Log generado automáticamente
```

---

## ⚙️ Requisitos del sistema

### 🖥 Windows (recomendado)
- PowerShell 5.1 o superior.
- [.NET Framework 4.7+](https://dotnet.microsoft.com/en-us/download/dotnet-framework)
- [`convbin`](https://rtkexplorer.com/downloads/) del paquete RTKLIB (debe estar junto al `.exe` Rust o en el PATH), el ejecutable lo he creado compilando el código fuente del modulo rtkconv presente en el repositorio mencionado de RTKLIB.
- [Rust (opcional)](https://www.rust-lang.org/) si deseas compilar desde código fuente.

---

## 📌 Características

### `monitor_rinex.ps1`
- Monitorea continuamente un directorio por archivos nuevos RINEX (`FICU*.25*`).
- Ejecuta `rinex_processor.exe` automáticamente cuando detecta archivos nuevos.
- Mantiene un registro de archivos ya procesados para evitar duplicados.
- Genera logs con marcas de tiempo en `rinex_monitor.log`.

### `main.rs` (Rust)
- Procesa todos los archivos en un directorio:
  - Agrupa por hora y tipo de archivo.
  - Ejecuta `convbin` en los archivos `.25O` (observación) para insertar campos *observer* y *agency*.
  - Renombra los archivos con formato estándar (`FICU<doy><hora>.25<tipo>`).
  - Comprime los archivos en `.zip` (uno por hora).
  - Mueve los archivos BeiDou (`.25C`) a una subcarpeta `*_beidou`.

---

## 🚀 Ejecución

### 1. Compila el binario Rust (si no usas el `.exe` incluido)

```bash
cargo build --release
```

> El ejecutable se generará en `target/release/rinex_processor.exe`.

### 2. Configura el script PowerShell

Edita los valores en `monitor_rinex.ps1` según tu sistema:

```powershell
$folder     = "C:\ruta\a\testcors"                  # Carpeta que será monitoreada
$rustScript = "C:\ruta\a\rinex_processor.exe"       # Ruta al ejecutable Rust
```

### 3. Ejecuta el script PowerShell

```powershell

Set-ExecutionPolicy RemoteSigned -Scope Process -Force
.\monitor_rinex.ps1
```

---

## 🧠 Notas adicionales


- Asegura de que `convbin` esté disponible en el mismo directorio que `rinex_processor.exe` o en el PATH.


---

## 📄 Licencia

MIT License.
