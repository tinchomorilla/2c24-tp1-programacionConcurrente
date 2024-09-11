use rayon::prelude::*;
use serde_json::json;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::Instant,
};

const EXPECTED_ARGS: usize = 4;

#[allow(unused_variables)]
fn main() -> std::io::Result<()> {
    let start = Instant::now();
    // Parsear los argumentos de la línea de comandos
    let args: Vec<String> = env::args().collect();
    if args.len() != EXPECTED_ARGS {
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
        let mut player_kills = HashMap::new();
        let mut number_of_deaths_and_distances: HashMap<String, (i32, f64, i32)> = HashMap::new();

        if let Some(weapon) = fields.first() {
            let (count, distances, len) = number_of_deaths_and_distances
                .entry(weapon.to_string())
                .or_insert((0, 0.0, 0));
            *count += 1;
            if let (Some(killer_x), Some(killer_y), Some(victim_x), Some(victim_y)) = (
                fields.get(3).and_then(|x| x.parse::<f64>().ok()),
                fields.get(4).and_then(|y| y.parse::<f64>().ok()),
                fields.get(10).and_then(|x| x.parse::<f64>().ok()),
                fields.get(11).and_then(|y| y.parse::<f64>().ok()),
            ) {
                let distance =
                    ((killer_x - victim_x).powi(2) + (killer_y - victim_y).powi(2)).sqrt();
                *distances += distance;
                *len += 1;
            }
            if let Some(player) = fields.get(1) {
                if player != &"" {
                    let player_weapons = player_kills
                        .entry(player.to_string())
                        .or_insert(HashMap::new());
                    let number_of_deaths_caused =
                        player_weapons.entry(weapon.to_string()).or_insert(0);
                    *number_of_deaths_caused += 1;
                }
            }
        }

        (number_of_deaths_and_distances, player_kills)
    });

    // Reduce todos los HashMaps a un solo HashMap, que contiene todas las claves juntas (armas)
    // y sus valores acumulados.
    // result = { "arma1": 10, "arma2": 20, ...}
    let result = mapped_iter.reduce(
        || (HashMap::new(), HashMap::new()),
        |(mut acc_number_of_deaths_and_distances, mut acc_players_weapons),
         (counts, player_kills)| {
            counts.iter().for_each(|(k, v)| {
                let (acc_count, acc_distances, len) = acc_number_of_deaths_and_distances
                    .entry(k.to_string())
                    .or_insert((0, 0.0, 0));
                *acc_count += v.0;
                *acc_distances += v.1;
                *len += v.2;
            });

            player_kills.iter().for_each(|(k, v)| {
                let player_weapons = acc_players_weapons
                    .entry(k.to_string())
                    .or_insert(HashMap::new());
                v.iter().for_each(|(weapon, count)| {
                    let player_weapon_count = player_weapons.entry(weapon.to_string()).or_insert(0);
                    *player_weapon_count += count;
                });
            });

            (acc_number_of_deaths_and_distances, acc_players_weapons)
        },
    );

    let (weapons, player_kills) = result;

    let duration = start.elapsed();

    println!("Tiempo total de lectura: {:?}", duration);

    //Ordenar las armas por cantidad de muertes causadas
    let mut weapons_vec = weapons.iter().collect::<Vec<_>>();
    weapons_vec.sort_unstable_by_key(|&(_, (count, distances, _))| -count);

    // Calcular las distancias para las 10 armas principales
    let top_10_distances: HashMap<&String, f64> = weapons_vec
        .iter()
        .take(10)
        .map(|(k, (count, distances, len))| (*k, distances / *len as f64))
        .collect::<HashMap<_, _>>();

    // Calcular el total de muertes
    let total_deaths_caused_by_weapons: i32 = weapons.values().map(|(count, _, _)| *count).sum();

    //Ordenar los jugadores por cantidad de muertes causadas
    let mut players_weapons_vec: Vec<(&String, &HashMap<String, i32>)> =
        player_kills.iter().collect();
    players_weapons_vec.sort_unstable_by_key(|&(_, weapons)| -weapons.values().sum::<i32>());
    // Tomar los 10 primeros jugadores
    let top_10_players: Vec<(&String, &HashMap<String, i32>)> =
        players_weapons_vec.iter().take(10).cloned().collect();

    let top_killers: HashMap<String, serde_json::Value> = top_10_players
        .iter()
        .map(|(player, weapons)| {
            let total_deaths_caused_by_player: i32 = weapons.values().sum();
            let mut weapons_vec: Vec<(&String, &i32)> = weapons.iter().collect();
            weapons_vec.sort_unstable_by_key(|&(_, &count)| -count);
            let top_3_weapons = weapons_vec
                .iter()
                .take(3)
                .map(|(weapon, &count)| {
                    let percentage = (count as f64 / total_deaths_caused_by_player as f64) * 100.0;
                    let rounded_percentage = (percentage * 100.0).round() / 100.0;
                    (weapon, rounded_percentage)
                })
                .collect::<HashMap<_, _>>();
            (
                (*player).clone(),
                json!({
                    "deaths": total_deaths_caused_by_player,
                    "weapons_percentage": top_3_weapons
                }),
            )
        })
        .collect();

    let top_weapons: HashMap<String, serde_json::Value> = weapons_vec
        .iter()
        .take(10)
        .map(|(weapon, (count, distances, len))| {
            let percentage = (*count as f64 / total_deaths_caused_by_weapons as f64) * 100.0;
            let rounded_percentage = (percentage * 100.0).round() / 100.0;
            let avg_distance = (distances / *len as f64 * 100.0).round() / 100.0;
            (
                (*weapon).clone(),
                json!({
                    "deaths_percentage": rounded_percentage,
                    "average_distance": avg_distance
                }),
            )
        })
        .collect();

    let output = json!({
        "padron": 108091,
        "top_killers": top_killers,
        "top_weapons": top_weapons
    });

    let mut file = File::create(output_file_name)?;
    file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;

    Ok(())
}
