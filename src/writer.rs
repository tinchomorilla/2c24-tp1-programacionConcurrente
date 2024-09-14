use std::{collections::HashMap, fs::File, io::Write};

use serde_json::json;

pub struct Writer {
    output_file_name: String,
}

impl Writer {
    pub fn new(output_file_name: &str) -> Self {
        Self {
            output_file_name: output_file_name.to_string(),
        }
    }

    fn get_output_file_name(&self) -> &str {
        &self.output_file_name
    }

    pub fn write_results_in_file(
        &self,
        top_killers: HashMap<String, serde_json::Value>,
        top_weapons: HashMap<String, serde_json::Value>,
    ) -> std::io::Result<()> {
        let output = json!({
            "padron": 108091,
            "top_killers": top_killers,
            "top_weapons": top_weapons
        });

        let mut file = File::create(self.get_output_file_name())?;
        file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;
        Ok(())
    }
}
