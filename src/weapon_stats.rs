#[derive(Debug, Clone)]
pub struct WeaponStats {
    death_distance: f64,
    number_of_kills_with_distance: u32,
    number_of_kills_without_distance: u32, // Representa
}

impl WeaponStats {
    pub fn new(
        death_distance: f64,
        number_of_kills_with_distance: u32,
        number_of_kills_without_distance: u32,
    ) -> Self {
        Self {
            death_distance,
            number_of_kills_with_distance,
            number_of_kills_without_distance,
        }
    }

    pub fn set_death_distance(&mut self, death_distance: f64) {
        self.death_distance += death_distance;
    }

    pub fn set_number_of_kills_with_valid_distance(&mut self, number_of_kills: u32) {
        self.number_of_kills_with_distance += number_of_kills;
    }

    pub fn set_total_kills_caused_by_weapon(&mut self, number_of_kills: u32) {
        self.number_of_kills_without_distance += number_of_kills;
    }

    pub fn get_death_distance(&self) -> f64 {
        self.death_distance
    }

    pub fn get_number_of_kills_with_valid_distance(&self) -> u32 {
        self.number_of_kills_with_distance
    }

    pub fn get_total_kills_caused_by_weapon(&self) -> u32 {
        self.number_of_kills_without_distance
    }
}
