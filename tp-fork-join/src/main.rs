use std::env;

fn main() -> std::io::Result<()> {
    // Parsear los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Uso: cargo run <input-path> <num-threads> <output-file-name>");
        std::process::exit(1);
    }
    let input_path = &args[1];
    let num_threads: usize = args[2]
        .parse()
        .expect("El segundo argumento debe ser un entero");
    let output_file_name = &args[3];

    println!("Procesando el archivo: {}", input_path);
    println!("Número de hilos: {}", num_threads);
    println!("Nombre del archivo de salida: {}", output_file_name);

    Ok(())
}
