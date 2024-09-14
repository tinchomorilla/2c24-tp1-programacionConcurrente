mod weapon_stats;

use rayon::{prelude::*, ThreadPoolBuilder};
use serde_json::json;
use std::{
    cmp::Ordering,
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::Instant,
};
use weapon_stats::WeaponStats;
type NumberOfDeathsAndDistances = HashMap<String, WeaponStats>;
type PlayersWeapons = HashMap<String, HashMap<String, i32>>;
type MappedItem = (NumberOfDeathsAndDistances, PlayersWeapons);
type ProcessedData = (NumberOfDeathsAndDistances, PlayersWeapons);

const INPUT_PATH_CONSOLE_ARGUMENT: usize = 1;
const NUMBER_OF_THREADS_CONSOLE_ARGUMENT: usize = 2;
const OUTPUT_FILE_CONSOLE_ARGUMENT: usize = 3;
const EXPECTED_ARGS: usize = 4;
const KILLER_NAME: usize = 1;
const KILLER_POSITION_X: usize = 3;
const KILLER_POSITION_Y: usize = 4;
const VICTIM_POSITION_X: usize = 10;
const VICTIM_POSITION_Y: usize = 11;
const TOP_PLAYERS: usize = 10;
const TOP_WEAPONS: usize = 3;

fn parse_args() -> (String, usize, String) {
    let args: Vec<String> = env::args().collect();
    if args.len() != EXPECTED_ARGS {
        eprintln!("Uso: cargo run <input-path> <num-threads> <output-file-name>");
        std::process::exit(1);
    }
    let input_path = args[INPUT_PATH_CONSOLE_ARGUMENT].clone();
    let num_threads: usize = args[NUMBER_OF_THREADS_CONSOLE_ARGUMENT]
        .parse()
        .expect("El segundo argumento debe ser un entero");
    let output_file_name = args[OUTPUT_FILE_CONSOLE_ARGUMENT].clone();
    (input_path, num_threads, output_file_name)
}

fn get_paths(input_path: &str) -> Vec<PathBuf> {
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
    paths_iter.collect::<Vec<PathBuf>>()
}

fn map_lines(
    lines_iter: impl ParallelIterator<Item = Result<String, std::io::Error>>,
) -> impl ParallelIterator<Item = MappedItem> {
    lines_iter.map(|l| {
        let line = l.unwrap();
        let fields: Vec<&str> = line.split(',').collect();
        let mut player_kills = HashMap::new();
        let mut weapons_stats = WeaponStats::new(0.0, 0, 0);
        let mut number_of_deaths_and_distances: HashMap<String, WeaponStats> = HashMap::new();

        if let Some(weapon) = fields.first() {
            if let (Some(killer_x), Some(killer_y), Some(victim_x), Some(victim_y)) = (
                fields
                    .get(KILLER_POSITION_X)
                    .and_then(|x| x.parse::<f64>().ok()),
                fields
                    .get(KILLER_POSITION_Y)
                    .and_then(|y| y.parse::<f64>().ok()),
                fields
                    .get(VICTIM_POSITION_X)
                    .and_then(|x| x.parse::<f64>().ok()),
                fields
                    .get(VICTIM_POSITION_Y)
                    .and_then(|y| y.parse::<f64>().ok()),
            ) {
                weapons_stats.set_death_distance(
                    ((killer_x - victim_x).powi(2) + (killer_y - victim_y).powi(2)).sqrt(),
                );
                weapons_stats.set_number_of_kills_with_distance(1);
            }
            weapons_stats.set_total_kills_caused_by_weapon(1);
            number_of_deaths_and_distances.insert(weapon.to_string(), weapons_stats);
            if let Some(player) = fields.get(KILLER_NAME) {
                if player != &"" {
                    let player_weapons = player_kills
                        .entry(player.to_string())
                        .or_insert(HashMap::new());
                    player_weapons.insert(weapon.to_string(), 1);
                }
            }
        }

        (number_of_deaths_and_distances, player_kills)
    })
}

fn reduce_mapped_iter(mapped_iter: impl ParallelIterator<Item = MappedItem>) -> MappedItem {
    mapped_iter.reduce(
        || (HashMap::new(), HashMap::new()),
        |(mut acc_number_of_deaths_and_distances, mut acc_players_weapons),
         (counts, player_kills)| {
            counts.iter().for_each(|(k, v)| {
                let acc_weapon_stats = acc_number_of_deaths_and_distances
                    .entry(k.to_string())
                    .or_insert(WeaponStats::new(0.0, 0, 0));
                acc_weapon_stats
                    .set_total_kills_caused_by_weapon(v.get_total_kills_caused_by_weapon());
                acc_weapon_stats.set_death_distance(v.get_death_distance());
                acc_weapon_stats
                    .set_number_of_kills_with_distance(v.get_number_of_kills_with_distance());
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
    )
}

fn process_csvs(paths: &Vec<PathBuf>) -> ProcessedData {
    // par_paths es un iterador paralelo sobre Vec<PathBuf>:
    // par_iter![
    //     PathBuf::from("input_dir/file1.txt"),
    //     PathBuf::from("input_dir/file2.txt")
    // ]
    let par_paths = paths.par_iter();

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
    let mapped_iter = map_lines(lines_iter);

    // Reduce todos los HashMaps a un solo HashMap, que contiene todas las claves juntas (armas)
    // y sus valores acumulados.
    // result = { "arma1": 10, "arma2": 20, ...}
    let (weapons, player_kills) = reduce_mapped_iter(mapped_iter);

    (weapons, player_kills)
}

fn calculate_top_weapons(
    weapons: HashMap<String, WeaponStats>,
) -> HashMap<String, serde_json::Value> {
    let mut weapons_vec = weapons.iter().collect::<Vec<_>>();
    weapons_vec.sort_unstable_by(|a, b| {
        let count_cmp =
            b.1.get_total_kills_caused_by_weapon()
                .cmp(&a.1.get_total_kills_caused_by_weapon()); // Ordenar por conteo en orden descendente
        if count_cmp == Ordering::Equal {
            a.0.cmp(b.0) // Si hay empate, ordenar alfabéticamente por el nombre del arma
        } else {
            count_cmp
        }
    });

    let total_deaths_caused_by_weapons: u32 = weapons
        .values()
        .map(|weapon_stats| weapon_stats.get_total_kills_caused_by_weapon())
        .sum();

    let top_weapons: HashMap<String, serde_json::Value> = weapons_vec
        .iter()
        .take(10)
        .map(|(weapon, weapon_stats)| {
            let percentage = (weapon_stats.get_total_kills_caused_by_weapon() as f64
                / total_deaths_caused_by_weapons as f64)
                * 100.0;
            let rounded_percentage = (percentage * 100.0).round() / 100.0;
            let avg_distance = (weapon_stats.get_death_distance()
                / weapon_stats.get_number_of_kills_with_distance() as f64
                * 100.0)
                .round()
                / 100.0;
            (
                (*weapon).clone(),
                json!({
                    "average_distance": avg_distance,
                    "deaths_percentage": rounded_percentage,
                }),
            )
        })
        .collect();

    top_weapons
}

fn calculate_top_killers(
    player_kills: HashMap<String, HashMap<String, i32>>,
) -> HashMap<String, serde_json::Value> {
    let mut players_weapons_vec: Vec<(&String, &HashMap<String, i32>)> =
        player_kills.iter().collect();
    players_weapons_vec.sort_unstable_by(|a, b| {
        let sum_a = a.1.values().sum::<i32>();
        let sum_b = b.1.values().sum::<i32>();
        let sum_cmp = sum_b.cmp(&sum_a); // Ordenar por suma en orden descendente
        if sum_cmp == Ordering::Equal {
            a.0.cmp(b.0) // Si hay empate, ordenar alfabéticamente por el nombre del jugador
        } else {
            sum_cmp
        }
    });

    let top_10_players: Vec<(&String, &HashMap<String, i32>)> = players_weapons_vec
        .iter()
        .take(TOP_PLAYERS)
        .cloned()
        .collect();

    let top_killers: HashMap<String, serde_json::Value> = top_10_players
        .iter()
        .map(|(player, weapons)| {
            let total_deaths_caused_by_player: i32 = weapons.values().sum();
            let mut weapons_vec: Vec<(&String, &i32)> = weapons.iter().collect();
            weapons_vec.sort_unstable_by(|a, b| {
                let count_cmp = b.1.cmp(a.1); // Ordenar por conteo en orden descendente
                if count_cmp == Ordering::Equal {
                    a.0.cmp(b.0) // Si hay empate, ordenar alfabéticamente por el nombre del arma
                } else {
                    count_cmp
                }
            });
            let top_3_weapons = weapons_vec
                .iter()
                .take(TOP_WEAPONS)
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

    top_killers
}

fn calculate_and_sort_results(
    weapons: HashMap<String, WeaponStats>,
    player_kills: HashMap<String, HashMap<String, i32>>,
) -> (
    HashMap<String, serde_json::Value>,
    HashMap<String, serde_json::Value>,
) {
    let top_killers = calculate_top_killers(player_kills);
    let top_weapons = calculate_top_weapons(weapons);

    (top_killers, top_weapons)
}

fn write_results_in_file(
    output_file_name: &str,
    top_killers: HashMap<String, serde_json::Value>,
    top_weapons: HashMap<String, serde_json::Value>,
) -> std::io::Result<()> {
    let output = json!({
        "padron": 108091,
        "top_killers": top_killers,
        "top_weapons": top_weapons
    });

    let mut file = File::create(output_file_name)?;
    file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let (input_path, num_threads, output_file_name) = parse_args();

    let start = Instant::now();
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        let paths = get_paths(&input_path);
        let (weapons, player_kills) = process_csvs(&paths);
        let duration = start.elapsed();
        let (top_killers, top_weapons) = calculate_and_sort_results(weapons, player_kills);
        let _ = write_results_in_file(&output_file_name, top_killers, top_weapons);
        println!("Tiempo total de lectura: {:?}", duration);
    });

    Ok(())
}
