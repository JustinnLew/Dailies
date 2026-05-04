use std::{collections::HashMap, sync::LazyLock};

use crate::state::geoguessr::Location;

pub(crate) struct Map {
    pub center: (f32, f32),
    pub locations: Vec<Location>,
    pub zoom: u8,
}

macro_rules! load_locations {
    ($path:literal) => {
        serde_json::from_str::<Vec<Location>>(include_str!($path))
            .expect(concat!("Failed to parse ", $path))
    };
}

pub static MAPS: LazyLock<HashMap<String, Map>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
        "World".to_string(),
        Map {
            center: (0.0, 0.0),
            locations: load_locations!("../geo_data/world.json"),
            zoom: 3,
        },
    );
    m.insert(
        "Australian Cities".to_string(),
        Map {
            center: (-25.2744, 133.7751),
            locations: load_locations!("../geo_data/australian_cities.json"),
            zoom: 4,
        },
    );
    m.insert(
        "Sydney".to_string(),
        Map {
            center: (-33.8688, 151.2093),
            locations: load_locations!("../geo_data/sydney.json"),
            zoom: 10,
        },
    );
    m
});