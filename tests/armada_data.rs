use armada_fleet_print::armada_data::{
    ArmadaDatabase, BsCatalogue, DataSource, Dice, SCHEMA_VERSION, Ship, Upgrade,
};
use serde_json::json;

/// Reduced BSData files, in the order `data sync` reads them.
const FIXTURES: [&str; 4] = [
    include_str!("fixtures/bsdata/Star_Wars_Armada.gst"),
    include_str!("fixtures/bsdata/Galactic Republic.cat"),
    include_str!("fixtures/bsdata/Imperial Navy.cat"),
    include_str!("fixtures/bsdata/Rebel Fleet.cat"),
];

fn database() -> ArmadaDatabase {
    let catalogues: Vec<BsCatalogue> = FIXTURES
        .iter()
        .map(|xml| BsCatalogue::parse(xml).unwrap())
        .collect();
    let source = DataSource {
        repo: "BSData/star-wars-armada".to_owned(),
        git_ref: "1.7.8".to_owned(),
        commit: "0".repeat(40),
        generated_at: "2026-11-07T00:00:00Z".to_owned(),
    };
    ArmadaDatabase::build(source, &catalogues)
}

fn ship<'a>(database: &'a ArmadaDatabase, name: &str) -> &'a Ship {
    database
        .ships
        .iter()
        .find(|ship| ship.name == name)
        .unwrap_or_else(|| panic!("no ship {name}"))
}

fn upgrade<'a>(database: &'a ArmadaDatabase, name: &str) -> &'a Upgrade {
    database
        .upgrades
        .iter()
        .find(|upgrade| upgrade.name == name)
        .unwrap_or_else(|| panic!("no upgrade {name}"))
}

#[test]
fn reads_gladiator() {
    let database = database();
    let gladiator = ship(&database, "Gladiator I-class Star Destroyer");

    assert_eq!(gladiator.faction.as_deref(), Some("Imperial Navy"));
    assert_eq!(gladiator.points, Some(56));
    assert_eq!(gladiator.size.as_deref(), Some("Small Ship"));
    assert_eq!(gladiator.kind, "Ships");
    assert!(!gladiator.unique);
    assert_eq!(
        (
            gladiator.hull,
            gladiator.command,
            gladiator.squadron,
            gladiator.engineering
        ),
        (Some(5), Some(2), Some(2), Some(3))
    );
    assert_eq!(gladiator.defense_tokens, ["Evade", "Brace", "Redirect"]);
    // Base value: the modifier adding Defensive Retrofit is not applied.
    assert_eq!(
        gladiator.upgrade_bar,
        ["Officer", "Weapons Team", "Support Team", "Ordnance"]
    );
    assert_eq!(gladiator.max_speed, Some(3));
    assert_eq!(
        serde_json::to_value(&gladiator.speed_chart).unwrap(),
        json!({"1": [2], "2": [1, 1], "3": [0, 1, 1]})
    );
}

#[test]
fn reads_gladiator_armament() {
    let database = database();
    let gladiator = ship(&database, "Gladiator I-class Star Destroyer");

    let front = &gladiator.batteries["front"];
    assert_eq!((front.red, front.blue, front.black), (2, 0, 2));
    assert_eq!(gladiator.batteries["left"].black, 4);
    let anti_squadron = gladiator.anti_squadron.as_ref().unwrap();
    assert_eq!(
        (anti_squadron.blue, anti_squadron.raw.as_str()),
        (1, "1 Blue")
    );
    let shields: Vec<(&str, u32)> = gladiator
        .shields
        .iter()
        .map(|(zone, value)| (zone.as_str(), *value))
        .collect();
    assert_eq!(
        shields,
        [("front", 3), ("left", 2), ("right", 2), ("rear", 1)]
    );
    assert!(gladiator.keywords.is_empty());
    assert!(gladiator.other.is_empty());
}

#[test]
fn sorts_shuffled_speed_chart_profiles() {
    let database = database();
    let acclamator = ship(&database, "Acclamator I-class Assault Ship");

    assert_eq!(acclamator.faction.as_deref(), Some("Galactic Republic"));
    assert_eq!(
        serde_json::to_value(&acclamator.speed_chart).unwrap(),
        json!({"1": [1], "2": [1, 0], "3": [0, 1, 0]})
    );
}

#[test]
fn unique_from_entry_name() {
    let database = database();
    let mon_karren = upgrade(&database, "Mon Karren");

    assert!(mon_karren.unique);
    assert_eq!(mon_karren.upgrade_type, "Title");
    assert_eq!(mon_karren.factions, ["Rebel Fleet"]);
    assert_eq!(mon_karren.points, Some(6));
}

#[test]
fn merges_generic_upgrades() {
    let database = database();
    let copies = database
        .upgrades
        .iter()
        .filter(|upgrade| upgrade.name == "Gunnery Team")
        .count();
    let gunnery_team = upgrade(&database, "Gunnery Team");

    assert_eq!(copies, 1);
    assert_eq!(gunnery_team.points, Some(7));
    assert_eq!(gunnery_team.upgrade_type, "Weapons Team");
    assert!(!gunnery_team.unique);
    assert_eq!(
        gunnery_team.factions,
        ["Galactic Republic", "Imperial Navy", "Rebel Fleet"]
    );
    assert!(
        gunnery_team.text.starts_with("[Concentrate Fire]:"),
        "{}",
        gunnery_team.text
    );
}

#[test]
fn reads_squadrons_and_objectives() {
    let database = database();
    let [tie] = database.squadrons.as_slice() else {
        panic!("expected one squadron");
    };
    assert_eq!(
        (tie.name.as_str(), tie.points, tie.speed, tie.hull),
        ("TIE Fighter Squadron", Some(8), Some(4), Some(3))
    );
    assert_eq!(tie.keywords, ["Swarm"]);

    let [objective] = database.objectives.as_slice() else {
        panic!("expected one objective");
    };
    assert_eq!(
        (objective.name.as_str(), objective.kind.as_str()),
        ("Advanced Gunnery", "Assault")
    );
    assert!(objective.special_rule.is_some());
}

#[test]
fn parses_dice_text() {
    let dice = Dice::parse("2 Red, 1 Black");
    assert_eq!((dice.red, dice.blue, dice.black), (2, 0, 1));
    assert_eq!(dice.raw, "2 Red, 1 Black");
}

#[test]
fn serializes_expected_layout() {
    let value = serde_json::to_value(database()).unwrap();

    assert_eq!(value["schema_version"], SCHEMA_VERSION);
    assert_eq!(value["source"]["ref"], "1.7.8");
    assert_eq!(value["upgrades"][0]["type"], "Weapons Team");
    assert_eq!(value["ships"][0]["faction"], "Galactic Republic");
    assert_eq!(
        value["ships"][1]["anti_squadron"],
        json!({"red": 0, "blue": 1, "black": 0, "raw": "1 Blue"})
    );
}
