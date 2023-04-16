use iced::{
    executor, theme,
    widget::{button, column, container, row, scrollable, text, text_input},
    Application, Color, Command, Element, Padding, Settings, Theme,
};
use minestuck_datapack_generator::{
    grist_resource, validate_resource_location, Datapack, GristCostRecipe, Ingredient, Recipe, ResultItem,
};

fn main() -> iced::Result {
    DatapackGui::run(Settings::default())
}

struct DatapackGui {
    costs: Vec<CostEntry>,
    errors: Vec<ExportError>,
}

#[derive(Debug)]
struct ExportError {
    text: String,
    position: usize,
    invalid: bool,
}

#[derive(Default)]
struct CostEntry {
    item_id: String,
    valid_item: bool,
    grist: Vec<GristField>,
}

impl CostEntry {
    fn new(item_id: &str, grist: Vec<GristField>) -> CostEntry {
        CostEntry {
            item_id: item_id.to_string(),
            valid_item: validate_resource_location(item_id),
            grist,
        }
    }
}

#[derive(Default)]
struct GristField {
    name: String,
    valid_name: bool,
    amount_string: String,
    amount: Option<i32>,
}

impl GristField {
    fn new(name: &str, amount: i32) -> GristField {
        GristField {
            name: name.to_string(),
            valid_name: validate_resource_location(&grist_resource(name)),
            amount_string: amount.to_string(),
            amount: Some(amount),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ItemId(usize, String),
    GristName(usize, usize, String),
    GristAmount(usize, usize, String),
    Export,
    Goto(usize),
}

impl Application for DatapackGui {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        //TODO load existing grist costs
        let datapack = Datapack::load("./datapack/");
        let costs = datapack
            .recipes
            .into_iter()
            .filter_map(|(_, recipe)| match recipe {
                Recipe::GristCost(recipe) => match &recipe.ingredient {
                    Ingredient::Item(id) => Some(CostEntry::new(
                        id,
                        recipe
                            .grist_cost
                            .iter()
                            .map(|(grist, amount)| GristField::new(grist.strip_prefix("minestuck:").unwrap_or(grist), *amount))
                            .collect(),
                    )),
                    _ => None,
                },
                Recipe::Combination(recipe) => match (&recipe.input1, &recipe.input2, &recipe.output) {
                    (Ingredient::Item(_), Ingredient::Item(_), ResultItem::Item(_)) => Some(CostEntry::new(
                        "",
                        vec![GristField::new("", 0)]
                    )),
                    (_, _, _) => None,
                },
            })
            .collect();
        (
            DatapackGui {
                costs,
                errors: Vec::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Minestuck Datapack Generator")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ItemId(i, new_id) => {
                if i == self.costs.len() {
                    self.costs.push(Default::default());
                }
                let cost = &mut self.costs[i];
                cost.item_id = new_id.to_lowercase();
                cost.valid_item = validate_resource_location(&cost.item_id);
                Command::none()
            }
            Message::GristName(i, j, new_name) => {
                if i == self.costs.len() {
                    self.costs.push(Default::default());
                }
                if j == self.costs[i].grist.len() {
                    self.costs[i].grist.push(Default::default());
                }
                let grist = &mut self.costs[i].grist[j];
                grist.name = new_name.to_lowercase();
                grist.valid_name = validate_resource_location(&grist_resource(&grist.name));
                Command::none()
            }
            Message::GristAmount(i, j, new_amount) => {
                if i == self.costs.len() {
                    self.costs.push(Default::default());
                }
                if j == self.costs[i].grist.len() {
                    self.costs[i].grist.push(Default::default());
                }
                let grist = &mut self.costs[i].grist[j];
                grist.amount = new_amount.parse().ok();
                grist.amount_string = new_amount;
                Command::none()
            }
            Message::Export => {
                self.errors.clear();
                self.costs.retain(|cost| {
                    !(cost.item_id.trim().is_empty()
                        && cost
                            .grist
                            .iter()
                            .all(|g| g.amount_string.is_empty() && g.name.is_empty()))
                });

                let mut datapack = Datapack::new();
                for (i, cost) in self.costs.iter().enumerate() {
                    if cost.valid_item {
                        if cost
                            .grist
                            .iter()
                            .all(|g| g.amount.is_some() && g.valid_name)
                        {
                            let (domain, path) = cost.item_id.split_once(':').unwrap();
                            let ingredient = Ingredient::Item(cost.item_id.to_string());
                            let grist_cost = cost
                                .grist
                                .iter()
                                .map(|g| (grist_resource(&g.name), g.amount.unwrap()))
                                .collect();
                            //Not invalid but probably want to mention anyways
                            let recipe = GristCostRecipe {
                                priority: Some(101),
                                ingredient,
                                grist_cost,
                            };
                            if datapack
                                .recipes
                                .insert(
                                    format!("data/minestuck/recipes/grist_costs/{domain}/{path}"),
                                    recipe.into(),
                                )
                                .is_some()
                            {
                                self.errors.push(ExportError {
                                    text: format!("Duplicate item {}", cost.item_id),
                                    position: i,
                                    invalid: true
                                });
                            }
                            //Not invalid but should probably mention it anyways
                            if cost.grist.is_empty() {
                                    self.errors.push(ExportError {
                                        text: format!("No grist for {}", cost.item_id), position: i, invalid: false});
                            }
                        } else {
                            self.errors.push(ExportError {
                                text: format!("Invalid grist for {}", cost.item_id), position: i, invalid: true});
                        }
                    } else {
                            self.errors.push(ExportError {
                                text: format!("Invalid item {}", cost.item_id), position: i, invalid: true});
                    }
                }
                datapack.save("./datapack/");
                Command::none()
            }
            Message::Goto(i) => {
                let heights: Vec<f32> = self.costs.iter().map(|cost| (cost.grist.len() as f32 + 1.0) * 30.0 + 4.0).collect();
                let total_height: f32 = heights.iter().sum();
                let cost_position: f32 = heights.iter().take(i).sum();
                let y = cost_position / total_height;
                scrollable::snap_to(
                    scrollable::Id::new("costs"),
                    scrollable::RelativeOffset { x: 0.0, y },
                )
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let mut cost_column = column(
            self.costs
                .iter()
                .enumerate()
                .map(|(i, cost)| {
                    let item_style = if cost.valid_item {
                        TextInputTheme::Valid
                    } else {
                        TextInputTheme::Invalid
                    };
                    let cost_row = row![text_input("modid:itemname", &cost.item_id, move |s| {
                        Message::ItemId(i, s)
                    })
                    .style(theme::TextInput::Custom(Box::new(item_style)))
                    .width(200)];

                    let mut grist_column = column!();
                    for (j, grist) in cost.grist.iter().enumerate() {
                        let name_style = if grist.valid_name {
                            TextInputTheme::Valid
                        } else {
                            TextInputTheme::Invalid
                        };
                        let amount_style = if grist.amount.is_some() {
                            TextInputTheme::Valid
                        } else {
                            TextInputTheme::Invalid
                        };
                        let grist_row = row![
                            text_input("grist", &grist.name, move |s| Message::GristName(i, j, s))
                                .width(150)
                                .style(theme::TextInput::Custom(Box::new(name_style))),
                            text("="),
                            text_input("amount", &grist.amount_string, move |s| {
                                Message::GristAmount(i, j, s)
                            })
                            .width(100)
                            .style(theme::TextInput::Custom(Box::new(amount_style))),
                        ].height(30);
                        grist_column = grist_column.push(grist_row);
                    }
                    let grist_count = cost.grist.len();
                    let empty_grist_row = row![
                        text_input("grist", "", move |s| Message::GristName(i, grist_count, s))
                            .width(150),
                        text("="),
                        text_input("amount", "", move |s| {
                            Message::GristAmount(i, grist_count, s)
                        })
                        .width(100),
                    ];
                    grist_column = grist_column.push(empty_grist_row);

                    let style = if (i & 1) == 0 {
                        ContainerTheme::Light
                    } else {
                        ContainerTheme::Dark
                    };
                    container(cost_row.push(grist_column))
                        .padding(Padding::new(2.0))
                        .style(theme::Container::Custom(Box::new(style)))
                        .into()
                })
                .collect(),
        );
        let cost_count = self.costs.len();
        let empty_style = if (cost_count & 1) == 0 {
            ContainerTheme::Light
        } else {
            ContainerTheme::Dark
        };
        let empty_cost = container(row![text_input("modid:itemname", "", move |s| {
            Message::ItemId(cost_count, s)
        })
        .width(200)])
        .padding(Padding::new(2.0))
        .style(theme::Container::Custom(Box::new(empty_style)));
        cost_column = cost_column.push(empty_cost);

        let errors = scrollable(column(
            self.errors
                .iter()
                .map(|s| {
                    let mut message = text(&s.text);
                    if s.invalid {
                        message = message.style(Color::from_rgb(0.9, 0.1, 0.1));
                    }
                    row![
                        message,
                        button(text("Goto")).on_press(
                            Message::Goto(s.position)
                        )
                    ]
                    .into()
                })
                .collect(),
        ));

        let export_column = column![button(text("Export")).on_press(Message::Export), errors,];

        let content = row![scrollable(cost_column).id(scrollable::Id::new("costs")), export_column];

        content.into()
    }
}

#[derive(Default, Clone)]
enum ContainerTheme {
    #[default]
    Light,
    Dark,
}

impl container::StyleSheet for ContainerTheme {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let value = match self {
            ContainerTheme::Light => 0.8,
            ContainerTheme::Dark => 0.6,
        };
        container::Appearance {
            background: Color::from([value, value, value]).into(),
            ..Default::default()
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum TextInputTheme {
    #[default]
    Valid,
    Invalid,
}

impl text_input::StyleSheet for TextInputTheme {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        style.active(&Default::default())
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        style.focused(&Default::default())
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        style.placeholder_color(&Default::default())
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        if *self == TextInputTheme::Valid {
            style.value_color(&Default::default())
        } else {
            Color::from([0.9, 0.1, 0.1])
        }
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        style.selection_color(&Default::default())
    }
}
