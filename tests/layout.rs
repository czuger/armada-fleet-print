use std::collections::HashMap;

use armada_fleet_print::card_name::CardName;
use armada_fleet_print::fleet::{Fleet, Ship, SquadronGroup};
use armada_fleet_print::layout::{DocumentPlan, Element, LayoutError, Page, lay_out};

fn card(name: &str) -> CardName {
    CardName::new(name).unwrap()
}

fn ship(name: &str, upgrades: usize) -> Ship {
    Ship {
        card: card(name),
        upgrades: (0..upgrades)
            .map(|i| card(&format!("{name}-up{i}")))
            .collect(),
    }
}

fn squadrons(count: usize) -> Vec<SquadronGroup> {
    (0..count)
        .map(|i| SquadronGroup {
            card: card(&format!("sq{i}")),
            count: 2,
        })
        .collect()
}

fn fleet(ships: Vec<Ship>, squadrons: Vec<SquadronGroup>, objectives: &[&str]) -> Fleet {
    Fleet {
        name: "Test".to_owned(),
        total_points: 400,
        squadron_points: 100,
        ships,
        squadrons,
        objectives: objectives.iter().map(|name| card(name)).collect(),
    }
}

/// Lays out `fleet` with every card 1.5 times taller than wide.
fn plan(fleet: &Fleet) -> DocumentPlan {
    let ratios = fleet.cards().into_iter().map(|c| (c, 1.5)).collect();
    lay_out(fleet, &ratios).unwrap()
}

fn cards_on(page: &Page) -> Vec<&str> {
    page.elements
        .iter()
        .filter_map(|element| match element {
            Element::Card { card, .. } => Some(card.as_str()),
            _ => None,
        })
        .collect()
}

fn position_of(page: &Page, name: &str) -> (f32, f32, f32, f32) {
    page.elements
        .iter()
        .find_map(|element| match element {
            Element::Card {
                card,
                x,
                y,
                width,
                height,
            } if card.as_str() == name => Some((*x, *y, *width, *height)),
            _ => None,
        })
        .unwrap()
}

fn texts_on(page: &Page) -> Vec<&str> {
    page.elements
        .iter()
        .filter_map(|element| match element {
            Element::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn badge_count(page: &Page) -> usize {
    page.elements
        .iter()
        .filter(|element| matches!(element, Element::CountBadge { count: 2, .. }))
        .count()
}

#[test]
fn puts_ships_without_upgrades_side_by_side() {
    let plan = plan(&fleet(
        (1..=4).map(|i| ship(&format!("s{i}"), 0)).collect(),
        Vec::new(),
        &[],
    ));

    assert_eq!(plan.pages.len(), 1);
    assert_eq!(
        position_of(&plan.pages[0], "s1"),
        (36.0, 456.0, 200.0, 300.0)
    );
    assert_eq!(
        position_of(&plan.pages[0], "s2"),
        (239.0, 456.0, 200.0, 300.0)
    );
    assert_eq!(
        position_of(&plan.pages[0], "s3"),
        (36.0, 146.0, 200.0, 300.0)
    );
}

#[test]
fn ship_with_upgrades_takes_full_row() {
    let plan = plan(&fleet(vec![ship("s1", 4), ship("s2", 0)], Vec::new(), &[]));

    assert_eq!(plan.pages.len(), 1);
    assert_eq!(
        position_of(&plan.pages[0], "s1-up2"),
        (466.0, 606.0, 100.0, 150.0)
    );
    assert_eq!(
        position_of(&plan.pages[0], "s1-up3"),
        (246.0, 446.0, 100.0, 150.0)
    );
    // The upgrade rows (310 pt) are taller than the ship, so the next line starts below them.
    assert_eq!(
        position_of(&plan.pages[0], "s2"),
        (36.0, 136.0, 200.0, 300.0)
    );
}

#[test]
fn ship_with_one_upgrade_stays_alone_on_its_line() {
    let plan = plan(&fleet(
        vec![ship("s1", 0), ship("s2", 1), ship("s3", 0)],
        Vec::new(),
        &[],
    ));

    assert_eq!(plan.pages.len(), 2);
    assert_eq!(
        position_of(&plan.pages[0], "s2"),
        (36.0, 146.0, 200.0, 300.0)
    );
    assert_eq!(cards_on(&plan.pages[1]), ["s3"]);
}

#[test]
fn tall_upgrade_rows_push_next_ship_down() {
    let plan = plan(&fleet(vec![ship("s1", 7), ship("s2", 0)], Vec::new(), &[]));

    assert_eq!(plan.pages.len(), 2);
    assert_eq!(cards_on(&plan.pages[1]), ["s2"]);
}

#[test]
fn squadron_fits_beside_lone_ship() {
    let plan = plan(&fleet(vec![ship("s1", 0)], squadrons(2), &[]));

    assert_eq!(plan.pages.len(), 1);
    assert_eq!(
        position_of(&plan.pages[0], "sq0"),
        (239.0, 486.0, 180.0, 270.0)
    );
    assert_eq!(
        position_of(&plan.pages[0], "sq1"),
        (36.0, 176.0, 180.0, 270.0)
    );
}

#[test]
fn objectives_start_on_their_own_page() {
    let plan = plan(&fleet(Vec::new(), squadrons(4), &["o1", "o2"]));

    assert_eq!(plan.pages.len(), 2);
    assert_eq!(cards_on(&plan.pages[0]), ["sq0", "sq1", "sq2", "sq3"]);
    assert_eq!(position_of(&plan.pages[0], "sq1").0, 36.0 + 183.0);
    assert_eq!(position_of(&plan.pages[0], "sq3").1, 206.0);
    assert_eq!(badge_count(&plan.pages[0]), 4);
    assert_eq!(cards_on(&plan.pages[1]), ["o1", "o2"]);
    assert_eq!(
        position_of(&plan.pages[1], "o1"),
        (36.0, 486.0, 180.0, 270.0)
    );
}

#[test]
fn objectives_alone_fit_on_one_page() {
    let plan = plan(&fleet(Vec::new(), Vec::new(), &["o1", "o2", "o3"]));

    assert_eq!(plan.pages.len(), 1);
    assert_eq!(cards_on(&plan.pages[0]), ["o1", "o2", "o3"]);
}

#[test]
fn drops_trailing_empty_page() {
    let plan = plan(&fleet(Vec::new(), squadrons(6), &[]));
    assert_eq!(plan.pages.len(), 1);
}

#[test]
fn adds_footer_to_every_page() {
    let plan = plan(&fleet(Vec::new(), squadrons(6), &["o1"]));

    assert_eq!(
        texts_on(&plan.pages[1]),
        [
            "Test (100/400) - https://apps.ieroe.com/armada/",
            "page 2 sur 2"
        ]
    );
}

#[test]
fn empty_fleet_gives_one_page() {
    let plan = plan(&fleet(Vec::new(), Vec::new(), &[]));
    assert_eq!(plan.pages.len(), 1);
    assert!(cards_on(&plan.pages[0]).is_empty());
}

#[test]
fn reports_cards_without_size() {
    let err = lay_out(
        &fleet(vec![ship("s1", 0)], Vec::new(), &[]),
        &HashMap::new(),
    )
    .unwrap_err();
    assert!(matches!(err, LayoutError::UnknownCard(name) if name.as_str() == "s1"));
}
