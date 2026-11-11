use armada_fleet_print::card_name::CardName;
use armada_fleet_print::fleet::{Fleet, FleetParseError, SquadronGroup};

const PAGE: &str = include_str!("fixtures/fleet_207536.html");

fn card(name: &str) -> CardName {
    CardName::new(name).unwrap()
}

fn page_with(fleet_json: &str) -> String {
    format!(
        "<script>window.fleetBuilder = {{\n\t\tfleet: {fleet_json},\n\t\tstock: null\n}};</script>"
    )
}

#[test]
fn reads_saved_fleet_page() {
    let fleet = Fleet::from_page_html(PAGE).unwrap();

    assert_eq!(fleet.name, "Campagne 1 copy");
    assert_eq!((fleet.total_points, fleet.squadron_points), (397, 131));
    assert_eq!(fleet.ships.len(), 5);
    assert_eq!(fleet.ships[0].card, card("cr90-corvette-a.png"));
    assert_eq!(fleet.ships[0].upgrades, [card("dodonnas-pride.png")]);
    assert_eq!(
        fleet.ships[3].upgrades,
        [card("general-dodonna.png"), card("boosted-comms.png")]
    );
    assert_eq!(
        fleet.squadrons[1],
        SquadronGroup {
            card: card("lancer-class-pursuit-craft.png"),
            count: 2
        }
    );
    assert_eq!(fleet.squadrons.len(), 8);
    assert_eq!(
        fleet.objectives,
        [
            card("precision-strike.png"),
            card("fighter-ambush.png"),
            card("superior-positions.png")
        ]
    );
}

#[test]
fn lists_each_card_once() {
    let fleet = Fleet::from_page_html(PAGE).unwrap();
    let cards = fleet.cards();

    let corvettes = cards
        .iter()
        .filter(|name| name.as_str() == "cr90-corvette-a.png")
        .count();
    assert_eq!(corvettes, 1);
    assert_eq!(cards.len(), 4 + 6 + 8 + 3);
}

#[test]
fn merges_squadrons_and_tolerates_missing_parts() {
    let html = page_with(
        r#"{"name":null,"ships":[{"image":"a.png","upgrades":[]}],
            "squadrons":[{"image":"x.png","quantity":"2"},{"image":"y.png","quantity":0},
                         {"image":"x.png","quantity":1}],
            "assault":null,"defense":{"image":"d.png"},"navigation":null}"#,
    );
    let fleet = Fleet::from_page_html(&html).unwrap();

    assert_eq!(fleet.name, "");
    assert!(fleet.ships[0].upgrades.is_empty());
    assert_eq!(
        fleet.squadrons,
        [SquadronGroup {
            card: card("x.png"),
            count: 3
        }]
    );
    assert_eq!(fleet.objectives, [card("d.png")]);
}

#[test]
fn rejects_pages_without_fleet() {
    let err = Fleet::from_page_html("<html><script>var a = 1;</script></html>").unwrap_err();
    assert!(matches!(err, FleetParseError::NoFleetData));
}

#[test]
fn rejects_unsafe_image_names() {
    let html = page_with(r#"{"ships":[{"image":"../../etc/passwd"}]}"#);
    let err = Fleet::from_page_html(&html).unwrap_err();
    assert!(matches!(err, FleetParseError::Json(_)));
}
