use rayon::prelude::*;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    time::Instant,
};

#[allow(unused_variables)]
fn main() -> std::io::Result<()> {
    let start = Instant::now();
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

    // entries es un iterador de Result<DirEntry, Error>:
    // - DirEntry es un objeto que representa un directorio en el sistema de archivos
    let entries = fs::read_dir(input_path).unwrap();

    // flatten() convierte un iterador de iteradores en un iterador simple
    let valid_entries = entries.flatten();

    // Si tengo File1 y File2 en mi directorio, entonces paths_iter será un iterador de PathBuf
    // que contiene los paths de File1 y File2
    // [
    //     PathBuf::from("input_dir/file1.txt"),
    //     PathBuf::from("input_dir/file2.txt")
    // ]
    let paths_iter = valid_entries.map(|d| d.path());

    // Convertir el iterador en un vector
    // vec![
    //     PathBuf::from("input_dir/file1.txt"),
    //     PathBuf::from("input_dir/file2.txt")
    // ]
    let paths_vec = paths_iter.collect::<Vec<PathBuf>>();

    // par_paths es un iterador paralelo sobre Vec<PathBuf>:
    // par_iter![
    //     PathBuf::from("input_dir/file1.txt"),
    //     PathBuf::from("input_dir/file2.txt")
    // ]
    let par_paths = paths_vec.par_iter();

    // lines_iter es un iterador paralelo de líneas de todos los archivos.
    // Si tuviese File1 y File2 , con 2 lineas en cada archivo, luego del flat_map() quedaria:
    // par_iter![
    //     Ok("line1_file1"),
    //     Ok("line2_file1"),
    //     Ok("line1_file2"),
    //     Ok("line2_file2")
    // ]
    let lines_iter = par_paths.flat_map(|path| {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        reader.lines().par_bridge()
    });

    // Cada linea del csv es una muerte causada por un arma, por lo que mapeamos cada linea a un HashMap
    // Finalmente, map nos devuelve un iterador paralelo que contiene todos los HashMaps representado por:
    // key: el nombre del arma,  value: la cantidad de muertes causadas por cada arma.
    // mapped_iter = [ {"arma1": 1}, {"arma2": 1}, {"arma1": 1}, {"arma3": 1}, {"arma2": 1} ]
    let mapped_iter = lines_iter.map(|l| {
        let line = l.unwrap();
        let fields: Vec<&str> = line.split(',').collect();
        let mut counts = HashMap::new();

        if let Some(killed_by) = fields.first() {
            let number_of_deaths_caused = counts.entry(killed_by.to_string()).or_insert(0);
            *number_of_deaths_caused += 1;
        }
        counts
    });

    // Reduce todos los HashMaps a un solo HashMap, que contiene todas las claves juntas (armas)
    // y sus valores acumulados.
    // result = { "arma1": 10, "arma2": 20, ...}
    let result = mapped_iter.reduce(
        HashMap::new,
        |mut acc, map| {
            map.iter().for_each(|(k, v)| {
                let count = acc.entry(k.to_string()).or_insert(0);
                *count += v;
            });
            acc
        },
    );

    let duration = start.elapsed();

    println!("Tiempo total de lectura: {:?}", duration);

    let mut top_weapons: Vec<(&String, &i32)> = result.iter().collect();
    top_weapons.sort_by(|a: &(&String, &i32), b| b.1.cmp(a.1));

    // Calcular el total de muertes
    let total_deaths: i32 = result.values().sum();

    println!("Top 10 armas que causaron más muertes:");
    for (weapon, count) in top_weapons.iter().take(10) {
        let percentage = (**count as f64 / total_deaths as f64) * 100.0;
        let rounded_percentage = (percentage * 100.0).round() / 100.0; // Redondear a dos decimales
        println!("{}: {} ({:.2}%)", weapon, count, rounded_percentage);
    }


 

    Ok(())
}
