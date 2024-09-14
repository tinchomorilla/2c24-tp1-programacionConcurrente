#[derive(Debug)]
pub struct WeaponStats {
    death_distance: f64,
    number_of_kills: u32,
}

impl WeaponStats {
    pub fn new(death_distance: f64, number_of_kills: u32) -> Self {
        Self {
            death_distance,
            number_of_kills,
        }
    }

    pub fn set_death_distance(&mut self, death_distance: f64) {
        self.death_distance = death_distance;
    }

    pub fn set_number_of_kills(&mut self, number_of_kills: u32) {
        self.number_of_kills = number_of_kills;
    }

    pub fn get_death_distance(&self) -> f64 {
        self.death_distance
    }

    pub fn get_number_of_kills(&self) -> u32 {
        self.number_of_kills
    }
}
