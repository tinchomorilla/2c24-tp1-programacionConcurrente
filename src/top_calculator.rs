use std::{cmp::Ordering, collections::HashMap};

use rayon::iter::IntoParallelRefIterator;
use serde_json::json;

use rayon::iter::ParallelIterator;
use rayon::iter::IndexedParallelIterator;


use crate::weapon_stats::WeaponStats;

const TOP_PLAYERS: usize = 10;
const TOP_WEAPONS: usize = 3;

pub struct TopCalculator {}

impl TopCalculator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn calculate_top_weapons(
        &self,
        weapons: HashMap<String, WeaponStats>,
    ) -> HashMap<String, serde_json::Value> {
        let weapons_vec = sort_weapons_by_kills(&weapons);
        let total_deaths_caused_by_weapons = calculate_total_deaths(&weapons);

        let top_weapons = weapons_vec
            .par_iter()
            .take(10)
            .map(|(weapon, weapon_stats)| {
                let percentage = calculate_percentage(
                    weapon_stats.get_total_kills_caused_by_weapon(),
                    total_deaths_caused_by_weapons,
                );
                let avg_distance = calculate_average_distance(weapon_stats);
                (
                    (*weapon).clone(),
                    json!({
                        "average_distance": avg_distance,
                        "deaths_percentage": percentage,
                    }),
                )
            })
            .collect();

        top_weapons
    }

    pub fn calculate_and_sort_results(
        &self,
        weapons: HashMap<String, WeaponStats>,
        player_kills: HashMap<String, HashMap<String, i32>>,
    ) -> (
        HashMap<String, serde_json::Value>,
        HashMap<String, serde_json::Value>,
    ) {
        let top_killers = self.calculate_top_killers(player_kills);
        let top_weapons = self.calculate_top_weapons(weapons);

        (top_killers, top_weapons)
    }

    pub fn calculate_top_killers(
        &self,
        player_kills: HashMap<String, HashMap<String, i32>>,
    ) -> HashMap<String, serde_json::Value> {
        let players_weapons_vec = sort_players_by_kills(&player_kills);
        let top_10_players: Vec<_> = players_weapons_vec
            .par_iter()
            .take(TOP_PLAYERS)
            .cloned()
            .collect();

        let top_killers = top_10_players
            .par_iter()
            .map(|(player, weapons)| {
                let total_deaths_caused_by_player = weapons.values().sum();
                let top_3_weapons =
                    calculate_top_weapons_for_player(weapons, total_deaths_caused_by_player);
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
}

fn sort_players_by_kills(
    player_kills: &HashMap<String, HashMap<String, i32>>,
) -> Vec<(&String, &HashMap<String, i32>)> {
    let mut players_weapons_vec: Vec<(&String, &HashMap<String, i32>)> =
        player_kills.par_iter().collect();
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
    players_weapons_vec
}

fn calculate_top_weapons_for_player(
    weapons: &HashMap<String, i32>,
    total_deaths_caused_by_player: i32,
) -> HashMap<String, f64> {
    let mut weapons_vec = weapons.par_iter().collect::<Vec<_>>();
    weapons_vec.sort_unstable_by(|a, b| {
        let count_cmp = b.1.cmp(a.1); // Ordenar por conteo en orden descendente
        if count_cmp == Ordering::Equal {
            a.0.cmp(b.0) // Si hay empate, ordenar alfabéticamente por el nombre del arma
        } else {
            count_cmp
        }
    });

    let top_weapons = weapons_vec
        .par_iter()
        .take(TOP_WEAPONS)
        .map(|(weapon, &count)| {
            let percentage = (count as f64 / total_deaths_caused_by_player as f64) * 100.0;
            let rounded_percentage = (percentage * 100.0).round() / 100.0;
            (weapon.to_string(), rounded_percentage)
        })
        .collect::<HashMap<String, f64>>();
    top_weapons
}

fn sort_weapons_by_kills(weapons: &HashMap<String, WeaponStats>) -> Vec<(&String, &WeaponStats)> {
    let mut weapons_vec = weapons.par_iter().collect::<Vec<_>>();
    weapons_vec.sort_unstable_by(|a, b| {
        let count_cmp =
            b.1.get_total_kills_caused_by_weapon()
                .cmp(&a.1.get_total_kills_caused_by_weapon());
        if count_cmp == Ordering::Equal {
            a.0.cmp(b.0)
        } else {
            count_cmp
        }
    });
    weapons_vec
}
fn calculate_total_deaths(weapons: &HashMap<String, WeaponStats>) -> u32 {
    weapons
        .values()
        .map(|weapon_stats| weapon_stats.get_total_kills_caused_by_weapon())
        .sum()
}

fn calculate_percentage(part: u32, total: u32) -> f64 {
    let percentage = (part as f64 / total as f64) * 100.0;
    (percentage * 100.0).round() / 100.0
}

fn calculate_average_distance(weapon_stats: &WeaponStats) -> f64 {
    let avg_distance = (weapon_stats.get_death_distance()
        / weapon_stats.get_number_of_kills_with_valid_distance() as f64
        * 100.0)
        .round()
        / 100.0;
    avg_distance
}
