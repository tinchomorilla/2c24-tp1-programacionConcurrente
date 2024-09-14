mod parser;
mod top_calculator;
mod weapon_stats;
mod writer;
use top_calculator::TopCalculator;
use weapon_stats::WeaponStats;
use writer::Writer;

use parser::Parser;
use rayon::{prelude::*, ThreadPoolBuilder};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    time::Instant,
};

type NumberOfDeathsAndDistances = HashMap<String, WeaponStats>;
type PlayersWeapons = HashMap<String, HashMap<String, i32>>;
type MappedItem = (NumberOfDeathsAndDistances, PlayersWeapons);
type ProcessedData = (NumberOfDeathsAndDistances, PlayersWeapons);

const KILLER_NAME: usize = 1;
const KILLER_POSITION_X: usize = 3;
const KILLER_POSITION_Y: usize = 4;
const VICTIM_POSITION_X: usize = 10;
const VICTIM_POSITION_Y: usize = 11;

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

fn main() -> std::io::Result<()> {
    let parser = Parser::new();
    let top_calculator = TopCalculator::new();
    let writer = Writer::new(parser.get_output_file_name());
    let start = Instant::now();
    let pool = ThreadPoolBuilder::new()
        .num_threads(parser.get_num_threads())
        .build()
        .unwrap();

    pool.install(|| {
        let paths = parser.get_paths();
        let (weapons, player_kills) = process_csvs(&paths);
        let duration = start.elapsed();
        let (top_killers, top_weapons) =
            top_calculator.calculate_and_sort_results(weapons, player_kills);
        let write_result = writer.write_results_in_file(top_killers, top_weapons);
        match write_result {
            Ok(_) => println!("Archivo escrito correctamente"),
            Err(e) => eprintln!("Error al escribir el archivo: {}", e),
        }
        println!("Tiempo total de lectura: {:?}", duration);
    });

    Ok(())
}
