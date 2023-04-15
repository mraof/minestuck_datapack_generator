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

        let mut columns = input.split(',');
        let input1 = columns.next().unwrap().trim();
        let mode = columns.next().unwrap().trim();
        let input2 = columns.next().unwrap().trim();
        let output = columns.next().unwrap().trim();

        let recipe: Recipe = CombinationRecipe {
            input1: Ingredient::Item(input1.to_string()),
            input2: Ingredient::Item(input2.to_string()),
            mode: CombinationMode::from_str(mode).unwrap(),
            output: ResultItem::Item(output.to_string())
        }
        .into();
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
