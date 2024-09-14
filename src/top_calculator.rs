use std::{cmp::Ordering, collections::HashMap};

use serde_json::json;

use crate::weapon_stats::WeaponStats;

const TOP_PLAYERS: usize = 10;
const TOP_WEAPONS: usize = 3;

pub struct TopCalculator {}

impl TopCalculator {
    pub fn new() -> Self {
        Self {}
    }

    fn calculate_top_weapons(
        &self,
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
        &self,
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
                        let percentage =
                            (count as f64 / total_deaths_caused_by_player as f64) * 100.0;
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
}
