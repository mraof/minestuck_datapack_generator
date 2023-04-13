use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    path::Path,
};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GristCostRecipe {
    pub priority: Option<i32>,
    pub ingredient: Ingredient,
    pub grist_cost: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Recipe {
    #[serde(rename = "minestuck:grist_cost")]
    GristCost(GristCostRecipe),
}

impl Recipe {
    pub fn is_valid(&self) -> bool {
        match self {
            Recipe::GristCost(recipe) => recipe.is_valid(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ingredient {
    Item(String),
    Tag(String),
}

impl From<GristCostRecipe> for Recipe {
    fn from(value: GristCostRecipe) -> Self {
        Recipe::GristCost(value)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MCMeta {
    pub pack: Pack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub pack_format: i32,
    // It can also be a json object but I don't want to implement all that
    pub description: String,
}

impl Default for Pack {
    fn default() -> Self {
        Self {
            pack_format: 10,
            description: "Created by Minestuck Datapack Generator".to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Datapack {
    pub mcmeta: MCMeta,
    pub recipes: HashMap<String, Recipe>,
}

impl GristCostRecipe {
    pub fn is_valid(&self) -> bool {
        match &self.ingredient {
            Ingredient::Item(name) => {
                if !validate_resource_location(name) {
                    return false;
                }
            }
            Ingredient::Tag(_) => {
                eprintln!("Tags are unsupported");
                return false;
            }
        }
        self.grist_cost
            .keys()
            .all(|g| validate_resource_location(g))
    }
}

pub fn validate_resource_location(id: &str) -> bool {
    let valid_for_namespace = "abcdefghijklmnopqrstuvwxyz0123456789_-.";
    let valid_for_path = "abcdefghijklmnopqrstuvwxyz0123456789_-./";
    if let Some((namespace, path)) = id.split_once(':') {
        namespace.chars().all(|c| valid_for_namespace.contains(c))
            && path.chars().all(|c| valid_for_path.contains(c))
    } else {
        false
    }
}

/// Adds the "minestuck" namespace if none
pub fn grist_resource(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minestuck:{id}")
    }
}

impl Datapack {
    pub fn new() -> Datapack {
        Default::default()
    }

    pub fn load<P>(path: P) -> Datapack
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mcmeta = match File::open(path.join("pack.mcmeta")) {
            Ok(file) => serde_json::from_reader(file).unwrap_or_default(),
            Err(_) => MCMeta::default(),
        };
        let mut recipes = HashMap::new();
        let data_path = path.join("data");
        if data_path.is_dir() {
            for dir_entry in WalkDir::new(data_path) {
                let dir_entry = dir_entry.unwrap();
                let recipe_path = dir_entry.path();
                if recipe_path.extension().map_or(false, |e| e == "json") {
                    let location = recipe_path
                        .strip_prefix(path)
                        .unwrap()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let file = File::open(recipe_path).unwrap();
                    match serde_json::from_reader::<_, Recipe>(file) {
                        Ok(recipe) => { recipes.insert(location, recipe); }
                        Err(e) => eprintln!("Failed to parse json at {recipe_path:?}, {e:?}")
                    }
                }
            }
        } else {
            println!("No existing recipes");
        }
        Datapack { mcmeta, recipes }
    }

    pub fn save<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        {
            //Don't replace existing pack.mcmeta
            if let Ok(file) = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path.join("pack.mcmeta"))
            {
                serde_json::to_writer_pretty(file, &self.mcmeta).unwrap();
            }
        }
        let grist_costs_path = path.join("data/minestuck/recipes/grist_costs");
        //It'll be confusing why deleting an entry doesn't remove the recipe so let's just start fresh
        let _ = std::fs::remove_dir_all(grist_costs_path);
        for (location, recipe) in &self.recipes {
            if recipe.is_valid() {
                let recipe_path = path.join(&format!("{location}.json"));
                std::fs::create_dir_all(recipe_path.parent().unwrap()).unwrap();
                let file = File::create(recipe_path).unwrap();
                serde_json::to_writer_pretty(file, &recipe).unwrap();
            } else {
                eprintln!("Invalid recipe: \"{recipe:#?}\"");
            }
        }
    }
}
