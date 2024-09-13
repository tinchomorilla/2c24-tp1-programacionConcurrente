use assert_json_diff::assert_json_eq;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[test]
fn test_output_with_expected_json() {
    // Definir el path del archivo JSON esperado
    let expected_file_path =
        "/home/tincho/Documents/Facultad/Concurrentes/2024-2c-tp1-tinchhoo/tp-fork-join/expected_output.json";

    // Definir los argumentos para ejecutar el programa
    let input_path =
        "/home/tincho/Documents/Facultad/Concurrentes/2024-2c-tp1-tinchhoo/tp-fork-join/deaths";
    let num_threads = "2";
    let output_file_path = "/home/tincho/Documents/Facultad/Concurrentes/2024-2c-tp1-tinchhoo/tp-fork-join/output.json";

    // Ejecutar el programa para generar el archivo JSON
    let _output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg(input_path)
        .arg(num_threads)
        .arg(output_file_path)
        .output()
        .expect("Error al ejecutar el programa");

    // Leer el archivo JSON generado por el programa
    let mut generated_file =
        File::open(output_file_path).expect("Error al abrir el archivo generado");
    let mut generated_content = String::new();
    generated_file
        .read_to_string(&mut generated_content)
        .expect("Error al leer el archivo generado");

    // Leer el archivo JSON esperado
    let mut expected_file =
        File::open(expected_file_path).expect("Error al abrir el archivo esperado");
    let mut expected_content = String::new();
    expected_file
        .read_to_string(&mut expected_content)
        .expect("Error al leer el archivo esperado");

    // Deserializar el contenido JSON generado por el programa
    let generated_json: Value =
        serde_json::from_str(&generated_content).expect("Error al parsear el JSON generado");

    // Deserializar el contenido JSON esperado
    let expected_json: Value =
        serde_json::from_str(&expected_content).expect("Error al parsear el JSON esperado");

    // Comparar ambos JSON sin importar el orden de los campos
    assert_json_eq!(generated_json, expected_json);
}
