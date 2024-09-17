use crate::{
    argument_parser::ArgumentParser, top_calculator::TopCalculator, weapon_stats::WeaponStats,
    writer::Writer,
};
use rayon::iter::{IntoParallelRefIterator, IterBridge, ParallelBridge, ParallelIterator};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::PathBuf,
    time::Instant,
};

type NumberOfDeathsAndDistances = HashMap<String, WeaponStats>;
type PlayersWeapons = HashMap<String, HashMap<String, i32>>;
type MappedItem = (NumberOfDeathsAndDistances, PlayersWeapons);
type ProcessedData = (NumberOfDeathsAndDistances, PlayersWeapons);
type ReducedItem = (NumberOfDeathsAndDistances, PlayersWeapons);

const KILLER_NAME: usize = 1;
const KILLER_POSITION_X: usize = 3;
const KILLER_POSITION_Y: usize = 4;
const VICTIM_POSITION_X: usize = 10;
const VICTIM_POSITION_Y: usize = 11;

pub struct Processor {
    start: Instant,
}

impl Processor {
    pub fn new(start: Instant) -> Self {
        Self { start }
    }

    fn process_murder_coordinates(&self, fields: &[&str], weapon_stats: &mut WeaponStats) {
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
            weapon_stats.set_death_distance(
                ((killer_x - victim_x).powi(2) + (killer_y - victim_y).powi(2)).sqrt(),
            );
            weapon_stats.set_number_of_kills_with_valid_distance(1);
        }
    }

    fn process_weapon(
        &self,
        fields: &[&str],
        mut weapon_stats: WeaponStats,
        weapons: &mut NumberOfDeathsAndDistances,
    ) {
        if let Some(weapon) = fields.first() {
            weapon_stats.set_total_kills_caused_by_weapon(1);
            weapons.insert(weapon.to_string(), weapon_stats);
        }
    }
    /// Processes player data.
    ///
    /// # Arguments
    ///
    /// * `fields` - Vector of fields from a CSV line.
    ///
    /// * `player_kills` - HashMap with player kills.
    ///
    /// # Returns
    ///
    /// Returns a HashMap with player kills.
    fn process_player(&self, fields: &[&str], player_kills: &mut PlayersWeapons) {
        if let Some(player) = fields.get(KILLER_NAME) {
            if player != &"" {
                if let Some(weapon) = fields.first() {
                    let player_weapons = player_kills.entry(player.to_string()).or_default();
                    player_weapons.insert(weapon.to_string(), 1);
                }
            }
        }
    }

    /// Maps lines from a CSV file to processed data.
    ///
    /// # Arguments
    /// * `lines_iter` - Parallel iterator over lines from a CSV file.
    ///
    /// # Returns
    /// Returns a parallel iterator over mapped items.
    fn map_lines<'a>(
        &'a self,
        lines_iter: impl ParallelIterator<Item = Result<String, std::io::Error>> + 'a,
    ) -> impl ParallelIterator<Item = MappedItem> + 'a {
        lines_iter.filter_map(|l| match l {
            Ok(line) => {
                let fields: Vec<&str> = line.split(',').collect();
                let mut player_kills = HashMap::new();
                let mut weapon_stats = WeaponStats::new(0.0, 0, 0);
                let mut weapons: HashMap<String, WeaponStats> = HashMap::new();

                self.process_murder_coordinates(&fields, &mut weapon_stats);
                self.process_weapon(&fields, weapon_stats, &mut weapons);
                self.process_player(&fields, &mut player_kills);

                Some((weapons, player_kills))
            }
            Err(e) => {
                eprintln!("Error al leer la linea: {}", e);
                None
            }
        })
    }

    /// Reduces a parallel iterator of mapped items into a single result.
    ///
    /// # Arguments
    /// * `mapped_iter` - Parallel iterator over mapped items.
    ///
    /// # Returns
    /// Returns a single `Reduced` with accumulated statistics.
    fn reduce_mapped_iter(
        &self,
        mapped_iter: impl ParallelIterator<Item = MappedItem>,
    ) -> ReducedItem {
        mapped_iter.reduce(
            || (HashMap::new(), HashMap::new()),
            |(mut acc_weapons, mut acc_players_weapons), (weapon, player_kills)| {
                self.add_deaths_and_distances(&mut acc_weapons, &weapon);
                self.add_players_weapons(&mut acc_players_weapons, &player_kills);
                (acc_weapons, acc_players_weapons)
            },
        )
    }

    /// Processes CSV files and returns processed data.
    ///
    /// # Arguments
    /// * `paths` - Vector of CSV file paths.
    ///
    /// # Returns
    /// Returns `ProcessedData` with weapon and player statistics.
    fn process_csvs(&self, paths: &Vec<PathBuf>) -> ProcessedData {
        let lines_iter = paths
            .par_iter()
            .filter_map(open_file)
            .flat_map(|lines_iter| lines_iter);

        let mapped_iter = self.map_lines(lines_iter);
        self.reduce_mapped_iter(mapped_iter)
    }

    fn add_deaths_and_distances(
        &self,
        acc_weapons: &mut NumberOfDeathsAndDistances,
        current_weapon: &NumberOfDeathsAndDistances,
    ) {
        current_weapon.iter().for_each(|(k, v)| {
            let acc_weapon_stats = acc_weapons
                .entry(k.to_string())
                .or_insert(WeaponStats::new(0.0, 0, 0));
            acc_weapon_stats.set_total_kills_caused_by_weapon(v.get_total_kills_caused_by_weapon());
            acc_weapon_stats.set_death_distance(v.get_death_distance());
            acc_weapon_stats.set_number_of_kills_with_valid_distance(
                v.get_number_of_kills_with_valid_distance(),
            );
        });
    }

    fn add_players_weapons(
        &self,
        acc_players_weapons: &mut PlayersWeapons,
        player_kills: &PlayersWeapons,
    ) {
        player_kills.iter().for_each(|(k, v)| {
            let player_weapons = acc_players_weapons.entry(k.to_string()).or_default();
            v.iter().for_each(|(weapon, count)| {
                let player_weapon_count = player_weapons.entry(weapon.to_string()).or_default();
                *player_weapon_count += count;
            });
        });
    }

    fn get_duration(&self) -> Instant {
        self.start
    }

    /// Processes CSV files and writes results to an output file.
    /// 
    /// # Arguments
    /// 
    /// * `parser` - Argument parser with command line arguments.
    /// 
    /// # Returns
    /// 
    /// No return value.
    pub fn process_and_write_results(&self, parser: &ArgumentParser) {
        let top_calculator = TopCalculator::new();
        let writer = Writer::new(parser.get_output_file_name());
        let (weapons, player_kills) = self.process_csvs(&parser.get_paths());
        let duration = self.get_duration().elapsed();
        let (top_killers, top_weapons) =
            top_calculator.calculate_and_sort_results(weapons, player_kills);
        match writer.write_results_in_file(top_killers, top_weapons) {
            Ok(_) => println!("Archivo escrito correctamente"),
            Err(e) => eprintln!("Error al escribir el archivo: {}", e),
        }
        println!("Tiempo total de lectura: {:?}", duration);
    }
}

fn open_file(path: &PathBuf) -> Option<IterBridge<Lines<BufReader<File>>>> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            Some(reader.lines().par_bridge())
        }
        Err(e) => {
            eprintln!("Error al abrir el archivo: {}", e);
            None
        }
    }
}
