use std::io::{stdin, BufRead, Write, BufReader};
use std::fs::File;

fn main() {
    let reader: Box<dyn BufRead> = if let Some(arg) = std::env::args().skip(1).next() {
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
        let costs = columns.map(|cost| {
            if let Some((grist, amount)) = cost.split_once('=') {
                Ok((grist.to_string(), amount))
            } else if let Some((grist, amount)) = cost.rsplit_once(':') {
                Ok((grist.to_string(), amount))
            } else {
                Err(format!("Error: invalid grist cost format: \"{}\"", cost))
            }.and_then(|(mut grist, amount)| {
                if !grist.contains(':') {
                    grist = format!("minestuck:{}", grist);
                }
                amount.trim().parse::<i32>().map(|amount| format!("    \"{}\": {}", grist.trim(), amount)).map_err(|e| format!("{} in \"{}\"", e.to_string(), amount))
            })
        }).collect::<Result<Vec<_>, _>>();
        let costs = match costs {
            Ok(costs) => costs,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        if costs.len() == 0 {
            continue;
        }
        let cost = costs.join(",\n");
        let json = format!(r#"{{
  "type": "minestuck:grist_cost",
  "priority": 101,
  "ingredient": {{
    "item": "{}"
  }},
  "grist_cost": {{
{}
  }}
}}"#, item, cost);
        if let Some((mod_id, item_name)) = item.split_once(':') {
            std::fs::create_dir_all(mod_id).unwrap();
            let mut file = File::create(&format!("{}/{}.json", mod_id, item_name)).unwrap();
            file.write(json.as_bytes()).unwrap();
        }
        else {
            eprintln!("Invalid item: \"{}\"", item);
        }
    }
}
