use std::{env, fs, path::PathBuf};

const INPUT_PATH_CONSOLE_ARGUMENT: usize = 1;
const NUMBER_OF_THREADS_CONSOLE_ARGUMENT: usize = 2;
const OUTPUT_FILE_CONSOLE_ARGUMENT: usize = 3;
const EXPECTED_ARGS: usize = 4;

pub struct ArgumentParser {
    input_path: String,
    num_threads: usize,
    output_file_name: String,
}

impl ArgumentParser {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        if args.len() != EXPECTED_ARGS {
            eprintln!("Uso: cargo run <input-path> <num-threads> <output-file-name>");
            std::process::exit(1);
        }
        Self {
            input_path: args[INPUT_PATH_CONSOLE_ARGUMENT].clone(),
            num_threads: args[NUMBER_OF_THREADS_CONSOLE_ARGUMENT]
                .parse()
                .expect("El segundo argumento debe ser un entero"),
            output_file_name: args[OUTPUT_FILE_CONSOLE_ARGUMENT].clone(),
        }
    }

    fn get_input_path(&self) -> &str {
        &self.input_path
    }

    pub fn get_num_threads(&self) -> usize {
        self.num_threads
    }

    pub fn get_output_file_name(&self) -> &str {
        &self.output_file_name
    }

    fn read_directory(&self, path: &str) -> fs::ReadDir {
        match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error al leer el directorio: {}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn get_paths(&self) -> Vec<PathBuf> {
        // entries es un iterador de Result<DirEntry, Error>:
        // DirEntry es un objeto que representa un directorio en el sistema de archivos
        let entries = self.read_directory(self.get_input_path());

        // flatten() filtra las entries que pueden ser Err y se queda con las entries validas,
        // es decir, las que son Ok
        let valid_entries = entries.flatten();

        // Convertir el iterador que devuelve map en un vector
        // vec![
        //     PathBuf::from("input_dir/file1.txt"),
        //     PathBuf::from("input_dir/file2.txt")
        // ]
        valid_entries.map(|d| d.path()).collect::<Vec<PathBuf>>()
    }
}
