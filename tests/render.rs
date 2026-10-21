use std::collections::HashMap;

use armada_fleet_print::card_image::CardImage;
use armada_fleet_print::card_name::CardName;
use armada_fleet_print::fleet::{Fleet, Ship, SquadronGroup};
use armada_fleet_print::layout::lay_out;
use armada_fleet_print::pdf::{RenderError, render};

fn card(name: &str) -> CardName {
    CardName::new(name).unwrap()
}

fn image(alpha: Option<Vec<u8>>) -> CardImage {
    CardImage {
        width: 2,
        height: 3,
        rgb: vec![200; 2 * 3 * 3],
        alpha,
    }
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

fn sample() -> (Fleet, HashMap<CardName, CardImage>) {
    let fleet = Fleet {
        name: "Flotte été".to_owned(),
        total_points: 400,
        squadron_points: 0,
        ships: vec![Ship {
            card: card("ship.png"),
            upgrades: vec![card("up.png")],
        }],
        squadrons: vec![SquadronGroup {
            card: card("squad.png"),
            count: 3,
        }],
        objectives: Vec::new(),
    };
    let images = HashMap::from([
        (card("ship.png"), image(None)),
        (card("up.png"), image(Some(vec![0, 255, 255, 255, 255, 0]))),
        (card("squad.png"), image(None)),
    ]);
    (fleet, images)
}

#[test]
fn renders_pdf_with_images_text_and_masks() {
    let (fleet, images) = sample();
    let ratios = images
        .iter()
        .map(|(card, image)| (card.clone(), image.aspect_ratio()))
        .collect();
    let plan = lay_out(&fleet, &ratios).unwrap();

    let pdf = render(&plan, &images, &fleet.name).unwrap();

    assert!(pdf.starts_with(b"%PDF-"));
    assert!(contains(&pdf, "/Count 1"));
    assert!(contains(&pdf, "/Subtype /Image"));
    assert_eq!(pdf.windows(6).filter(|w| w == b"/SMask").count(), 1);
    assert!(contains(&pdf, "(page 1 sur 1) Tj"));
    assert!(contains(&pdf, "(3) Tj"));
    // Non-ASCII text is written as a hex string of its WinAnsi bytes: "Flotte été".
    assert!(contains(&pdf.to_ascii_lowercase(), "<466c6f74746520e974e9"));
}

#[test]
fn reports_missing_images() {
    let (fleet, mut images) = sample();
    let ratios = images
        .iter()
        .map(|(card, image)| (card.clone(), image.aspect_ratio()))
        .collect();
    let plan = lay_out(&fleet, &ratios).unwrap();
    images.remove(&card("up.png"));

    let err = render(&plan, &images, &fleet.name).unwrap_err();
    assert!(matches!(err, RenderError::MissingImage(name) if name.as_str() == "up.png"));
}
