use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

use minestuck_datapack_generator::{GristCostRecipe, Ingredient, Recipe};

fn main() {
    let reader: Box<dyn BufRead> = if let Some(arg) = std::env::args().nth(1) {
        let file = File::open(arg).unwrap();
        Box::new(BufReader::new(file))
    } else {
        let stdin = stdin();
        Box::new(stdin.lock())
    };
    let grist_cost_dir = "data/minestuck/recipes/grist_costs";
    std::fs::create_dir_all(grist_cost_dir).unwrap();
    std::env::set_current_dir(grist_cost_dir).unwrap();
    for line in reader.lines() {
        let input = line.unwrap();

        let mut columns = input.split(',');
        let item = columns.next().unwrap().trim();
        let costs: Result<HashMap<String, i32>, _> = columns
            .map(|cost| {
                if let Some((grist, amount)) = cost.split_once('=') {
                    Ok((grist.to_string(), amount))
                } else if let Some((grist, amount)) = cost.rsplit_once(':') {
                    Ok((grist.to_string(), amount))
                } else {
                    Err(format!("Error: invalid grist cost format: \"{cost}\""))
                }
                .and_then(|(mut grist, amount)| {
                    if !grist.contains(':') {
                        grist = format!("minestuck:{grist}");
                    }
                    amount
                        .trim()
                        .parse::<i32>()
                        .map(|amount| (grist.trim().to_string(), amount))
                        .map_err(|e| format!("{e} in \"{amount}\""))
                })
            })
            .collect();
        let costs = match costs {
            Ok(costs) => costs,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        if costs.is_empty() {
            continue;
        }
        let recipe: Recipe = GristCostRecipe {
            priority: Some(101),
            ingredient: Ingredient::Item(item.to_string()),
            grist_cost: costs,
        }
        .into();
        if recipe.is_valid() {
            let (mod_id, item_name) = item.split_once(':').unwrap();
            std::fs::create_dir_all(mod_id).unwrap();
            let file = File::create(&format!("{mod_id}/{item_name}.json")).unwrap();
            serde_json::to_writer_pretty(file, &recipe).unwrap();
        } else {
            eprintln!("Invalid recipe: \"{recipe:#?}\"");
        }
    }
}
