use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::str::FromStr;

use minestuck_datapack_generator::{Ingredient, Recipe, CombinationRecipe, CombinationMode, ResultItem};

fn main() {
    let reader: Box<dyn BufRead> = if let Some(arg) = std::env::args().nth(1) {
        let file = File::open(arg).unwrap();
        Box::new(BufReader::new(file))
    } else {
        let stdin = stdin();
        Box::new(stdin.lock())
    };
    let combination_dir = "data/minestuck/recipes/combination";
    std::fs::create_dir_all(combination_dir).unwrap();
    std::env::set_current_dir(combination_dir).unwrap();
    for line in reader.lines() {
        let input = line.unwrap();

        let columns: Vec<_> = input.split(',').map(str::trim).collect();
        if columns.len() < 4 {
            //Silently ignore blank lines
            if !(columns.len() == 1 && columns[0].is_empty()) {
                eprintln!("{input} only has {} fields, needs 4", columns.len());
            }
            continue;
        }
        let input1 = columns[0];
        let mode = columns[1];
        let input2 = columns[2];
        let output = columns[3];

        let recipe: Recipe = if let Ok(mode) = CombinationMode::from_str(mode) {
            CombinationRecipe {
                input1: Ingredient::Item(input1.to_string()),
                input2: Ingredient::Item(input2.to_string()),
                mode,
                output: ResultItem::Item(output.to_string())
            }.into()
        } else {
            eprintln!("Invalid mode {mode}");
            continue;
        };

        if recipe.is_valid() {
            let (mod_id, item_name) = output.split_once(':').unwrap();
            std::fs::create_dir_all(mod_id).unwrap();
            let file = File::create(&format!("{mod_id}/{item_name}.json")).unwrap();
            serde_json::to_writer_pretty(file, &recipe).unwrap();
        } else {
            eprintln!("Invalid recipe: \"{recipe:#?}\"");
        }
    }
}
