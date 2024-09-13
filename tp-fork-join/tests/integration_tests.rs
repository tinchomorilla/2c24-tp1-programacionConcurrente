use assert_json_diff::assert_json_eq;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

#[test]
fn test_output_matches_expected() {
    let generated_file_path = "/home/tincho/Documents/Facultad/Concurrentes/2024-2c-tp1-tinchhoo/tp-fork-join/output.json";
    let expected_file_path =
        "/home/tincho/Documents/Facultad/Concurrentes/2024-2c-tp1-tinchhoo/tp-fork-join";

    let input_path =
        "/home/tincho/Documents/Facultad/Concurrentes/2024-2c-tp1-tinchhoo/tp-fork-join/deaths";
    let num_threads = "3";
    let output_file_name = generated_file_path;

    // Verificar que las rutas existen
    assert!(
        Path::new(input_path).exists(),
        "El directorio de entrada no existe: {}",
        input_path
    );
    assert!(
        Path::new(expected_file_path).exists(),
        "El archivo esperado no existe: {}",
        expected_file_path
    );

    // Ejecutar el programa para generar el archivo
    let _output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg(input_path)
        .arg(num_threads)
        .arg(output_file_name)
        .output()
        .expect("Failed to execute command");

    // Leer el archivo generado
    let mut generated_file =
        File::open(generated_file_path).expect("Unable to open generated file");
    let mut generated_content = String::new();
    generated_file
        .read_to_string(&mut generated_content)
        .expect("Unable to read generated file");

    // Leer el archivo esperado
    let mut expected_file = File::open(expected_file_path).expect("Unable to open expected file");
    let mut expected_content = String::new();
    expected_file
        .read_to_string(&mut expected_content)
        .expect("Unable to read expected file");

    // Deserializar el contenido JSON
    let generated_json: Value =
        serde_json::from_str(&generated_content).expect("Unable to parse generated JSON");
    let expected_json: Value =
        serde_json::from_str(&expected_content).expect("Unable to parse expected JSON");

    // Imprimir ambos JSON para comparación
    println!("Generated JSON: {}", generated_json);
    println!("Expected JSON: {}", expected_json);

    // Comparar los JSON ignorando el orden de los campos
    assert_json_eq!(generated_json, expected_json);
}
