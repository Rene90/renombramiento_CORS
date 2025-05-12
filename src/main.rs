use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::io::prelude::*;
use chrono::{Local, DateTime, Utc};
use zip::write::FileOptions;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::env;

fn main() -> io::Result<()> {
    setup_logging()?;
    log_message("INFO", "Iniciando procesamiento de archivos RINEX")?;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        log_message("ERROR", "No se proporcionó ruta de carpeta como argumento.")?;
        eprintln!("Uso: rinex_processor.exe <carpeta>");
        std::process::exit(1);
    }

    let base_dir = PathBuf::from(&args[1]);

    if !base_dir.exists() || !base_dir.is_dir() {
        log_message("ERROR", &format!("Directorio no válido: {}", base_dir.display()))?;
        eprintln!("La ruta proporcionada no existe o no es un directorio.");
        std::process::exit(1);
    }

    process_batch(base_dir)
}

fn process_batch(base_dir: PathBuf) -> io::Result<()> {
    setup_logging()?;
    log_message("INFO", "Iniciando procesamiento de archivos RINEX")?;
    //let base_dir = PathBuf::from("..").join("testcors");
    
    // Verificar existencia del directorio
    if !base_dir.exists() {
        let current_dir = std::env::current_dir()?;
        panic!(
            "ERROR CRÍTICO:\n\
            • No se encontró 'testcors'\n\
            • Ruta buscada: {}\n\
            • Directorio actual: {}\n\
            • Solución: Crea la carpeta 'testcors' junto a tu proyecto",
            base_dir.display(),
            current_dir.display()
        );
    }

    let mut files_by_hour = HashMap::new();
    let letter_mapping = create_letter_mapping();

    // Procesar cada archivo y preparar los nuevos nombres
    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with("FICU") && file_name.len() >= 13 {
                let day_of_year = &file_name[4..7]; // Extrae "028"
                let hour_part = &file_name[7..9];   // Ej: "00", "A0"
                let file_type = &file_name[12..13]; // "C", "G", etc.
            
                // Determinar la nueva letra
                let new_letter = if let Some(first_char) = hour_part.chars().next() {
                    if first_char.is_alphabetic() {
                        // 'a' a 'j' → 'k' a 't'
                        let mapped_char = ((first_char.to_ascii_lowercase() as u8) + 10) as char;
                        mapped_char.to_string()
                    } else {
                        letter_mapping
                            .get(hour_part)
                            .unwrap_or(&hour_part.to_string())
                            .clone()
                    }
                } else {
                    hour_part.to_string()//si no puede mapear deja el nombre tal cual
                };
            
                // Nuevo nombre base: FICU + día + letra
                let new_base_name = format!("FICU{}{}", day_of_year, new_letter);
            
                files_by_hour.entry(new_base_name.clone())
                    .or_insert_with(Vec::new)
                    .push((path.clone(), file_type.to_lowercase()));
            }
        }
    }

    // Crear archivos ZIP con los nuevos nombres
    /*
    for (base_name, files) in files_by_hour {
        let zip_path = base_dir.join(format!("{}.zip", base_name));
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path)?);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        
        for (file_path, file_type) in files {
            if file_type != "c" {  // Excluir archivos BeiDou (.25c)
                let new_file_name = format!("{}.25{}", base_name, file_type);
                let new_file_path = file_path.with_file_name(&new_file_name);
                
                // Renombrar el archivo
                fs::rename(&file_path, &new_file_path)?;
                
                // Agregar al ZIP
                let file_content = fs::read(&new_file_path)?;
                zip.start_file(new_file_name, options)?;
                zip.write_all(&file_content)?;
            }
        }
        println!("Created: {}", zip_path.display());
    }*/
//Modificar encabezado de archivo rinex de observacioin y Crear archivos ZIP, 
for (base_name, files) in files_by_hour {
    let zip_path = base_dir.join(format!("{}.zip", base_name));
    let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path)?);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (file_path, file_type) in files {
            if file_type != "c" {
                let new_file_name = format!("{}.25{}", base_name, file_type);
                let new_file_path = file_path.with_file_name(&new_file_name);

                if file_type == "o" {
                    // Ejecutar convbin para archivo de observación
                    let status = Command::new("./convbin")
                        .args(&[
                            "-r", "rinex",
                            "-o", new_file_path.to_str().unwrap(), // nombre de salida
                            "-ho", "FI/UNAM",
                            file_path.to_str().unwrap()             // nombre original
                        ])
                        .status()?;

                    if !status.success() {
                        eprintln!("Error ejecutando convbin para {:?}", file_path);
                    } else {
                        println!("convbin ejecutado: {:?}", new_file_path);
                    }
                } else {
                    // Para los demás, solo renombrar
                    fs::rename(&file_path, &new_file_path)?;
                }

                // Agregar al ZIP
                let file_content = fs::read(&new_file_path)?;
                zip.start_file(new_file_name, options)?;
                zip.write_all(&file_content)?;
            }
        }
    }

    //mover los archivos despues de que se hayan cambiado los nombres y zippeado los archivos
    let dir_name = base_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("testcors");
    let beidou_dir = base_dir.with_file_name(format!("{}_beidou", dir_name));
    if !beidou_dir.exists() {
        fs::create_dir(&beidou_dir)?;
    }

    // Mover archivos no ZIP al directorio _beidou
    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if !file_name.ends_with(".zip") && !path.is_dir() {
                let destination = beidou_dir.join(file_name);
                fs::rename(&path, &destination)?;
                println!("Moved to {}: {}", beidou_dir.display(), file_name);
            }
        }
    }

    Ok(())
}



/*
fn main() -> io::Result<()> {
    setup_logging()?;
    log_message("INFO", "Iniciando procesamiento de archivos RINEX")?;
    let base_dir = PathBuf::from("..").join("testcors");
    
    // Verificar existencia del directorio
    if !base_dir.exists() {
        let current_dir = std::env::current_dir()?;
        panic!(
            "ERROR CRÍTICO:\n\
            • No se encontró 'testcors'\n\
            • Ruta buscada: {}\n\
            • Directorio actual: {}\n\
            • Solución: Crea la carpeta 'testcors' junto a tu proyecto",
            base_dir.display(),
            current_dir.display()
        );
    }

    let mut files_by_hour = HashMap::new();
    let letter_mapping = create_letter_mapping();

    // Procesar cada archivo y preparar los nuevos nombres
    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with("FICU") && file_name.len() >= 13 {
                let day_of_year = &file_name[4..7]; // Extrae "028"
                let hour_part = &file_name[7..9];   // Ej: "00", "A0"
                let file_type = &file_name[12..13]; // "C", "G", etc.
            
                // Determinar la nueva letra
                let new_letter = if hour_part.chars().next().unwrap().is_alphabetic() {
                    hour_part.chars().next().unwrap().to_lowercase().to_string()
                } else {
                    letter_mapping.get(hour_part).unwrap_or(&hour_part.to_string()).clone()
                };
            
                // Nuevo nombre base: FICU + día + letra
                let new_base_name = format!("FICU{}{}", day_of_year, new_letter);
            
                files_by_hour.entry(new_base_name.clone())
                    .or_insert_with(Vec::new)
                    .push((path.clone(), file_type.to_lowercase()));
            }
        }
    }

    // Crear archivos ZIP con los nuevos nombres
    /*
    for (base_name, files) in files_by_hour {
        let zip_path = base_dir.join(format!("{}.zip", base_name));
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path)?);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        
        for (file_path, file_type) in files {
            if file_type != "c" {  // Excluir archivos BeiDou (.25c)
                let new_file_name = format!("{}.25{}", base_name, file_type);
                let new_file_path = file_path.with_file_name(&new_file_name);
                
                // Renombrar el archivo
                fs::rename(&file_path, &new_file_path)?;
                
                // Agregar al ZIP
                let file_content = fs::read(&new_file_path)?;
                zip.start_file(new_file_name, options)?;
                zip.write_all(&file_content)?;
            }
        }
        println!("Created: {}", zip_path.display());
    }*/
//Modificar encabezado de archivo rinex de observacioin y Crear archivos ZIP, 
    for (file_path, file_type) in files {
        if file_type != "c" {
            let new_file_name = format!("{}.25{}", base_name, file_type);
            let new_file_path = file_path.with_file_name(&new_file_name);

            if file_type == "o" {
                // Ejecutar convbin para archivo de observación
                let status = Command::new("./convbin")
                    .args(&[
                        "-r", "rinex",
                        "-o", new_file_path.to_str().unwrap(), // nombre de salida
                        "-ho", "FI/UNAM",
                        file_path.to_str().unwrap()             // nombre original
                    ])
                    .status()?;

                if !status.success() {
                    eprintln!("Error ejecutando convbin para {:?}", file_path);
                } else {
                    println!("convbin ejecutado: {:?}", new_file_path);
                }
            } else {
                // Para los demás, solo renombrar
                fs::rename(&file_path, &new_file_path)?;
            }

            // Agregar al ZIP
            let file_content = fs::read(&new_file_path)?;
            zip.start_file(new_file_name, options)?;
            zip.write_all(&file_content)?;
        }
    }

    //mover los archivos despues de que se hayan cambiado los nombres y zippeado los archivos
    let dir_name = base_dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("testcors");
    let beidou_dir = base_dir.with_file_name(format!("{}_beidou", dir_name));
    if !beidou_dir.exists() {
        fs::create_dir(&beidou_dir)?;
    }

    // Mover archivos no ZIP al directorio _beidou
    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if !file_name.ends_with(".zip") && !path.is_dir() {
                let destination = beidou_dir.join(file_name);
                fs::rename(&path, &destination)?;
                println!("Moved to {}: {}", beidou_dir.display(), file_name);
            }
        }
    }

    Ok(())

}*/


fn create_letter_mapping() -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    // Secuencia: 00=a, 10=b, 20=c, 30=d, ..., 90=i
    let letters = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'];
    for (i, c) in letters.iter().enumerate() {
        mapping.insert(format!("{}0", i ), c.to_string());
    }
    mapping
}
fn setup_logging() -> std::io::Result<()> {
    const LOG_FILE: &str = "rinex_processor.log";
    
    // Verificar si el archivo existe para añadir un separador
    let needs_separator = Path::new(LOG_FILE).exists();
    
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)?;
        
    if needs_separator {
        writeln!(
            log_file,
            "\n{} === Nueva ejecución ===",
            Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
        )?;
    } else {
        writeln!(
            log_file,
            "{} === Inicio de sesión ===",
            Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
        )?;
    }
    
    // Escribe metadatos iniciales útiles
    writeln!(
        log_file,
        "{} [SISTEMA] Versión {} | Plataforma: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    )?;
    
    Ok(())
}
fn log_message(level: &str, message: &str) -> std::io::Result<()> {
    let mut log_file = OpenOptions::new()
        .append(true)
        .open("rinex_processor.log")?;
        
    writeln!(
        log_file,
        "{} [{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        level,
        message
    )?;
    
    // También imprime a consola si es un error
    if level == "ERROR" {
        eprintln!("[{}] {}", level, message);
    }
    
    Ok(())
}