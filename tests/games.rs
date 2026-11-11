use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use armada_fleet_print::game::{
    FigureCount, FigureKind, FleetResult, FleetRoster, Game, GameFleet, GameReport, GameStore,
    GameStoreError, ModelCatalogue, Stock, StockParseError,
};

const CARDS_JS: &str = include_str!("fixtures/builder_cards.js");
const STOCK: &str = include_str!("fixtures/stock.tsv");
const FLEET_PAGE: &str = include_str!("fixtures/fleet_207536.html");

fn catalogue() -> ModelCatalogue {
    ModelCatalogue::parse_cards_js(CARDS_JS).unwrap()
}

fn temp_dir() -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    std::env::temp_dir().join(format!(
        "armada-fleet-print-games-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn fleet(id: u32, player: &str) -> GameFleet {
    GameFleet {
        id,
        player: player.to_owned(),
        url: format!("https://armada.ryankingston.com/fleet/{id}/"),
    }
}

#[test]
fn stock_sheet_names_match_builder_models() {
    let stock = Stock::parse(STOCK, &catalogue()).unwrap();
    let models: Vec<(&str, Option<&str>)> = stock
        .lines
        .iter()
        .map(|line| (line.name.as_str(), line.model.as_deref()))
        .collect();
    let expected = [
        ("Assault Frigate", "assault-frigate"),
        ("CR90 Corvette", "corvette"),
        ("GR-75 Medium Transports", "gr-75"),
        ("Hammerhead Corvette", "hammerhead"),
        ("MC30 Frigate", "mc30c-frigate"),
        ("MC75 Cruiser", "mc75-cruiser"),
        ("MC80 Command Cruiser", "mc80-cruiser"),
        ("MC80 Star Cruiser", "mc80-liberty"),
        ("Nebulon B Frigate", "nebulon-b"),
        ("Pelta Class Ship", "pelta"),
        ("Starhawk Battleship", "starhawk"),
        ("A-Wing", "a-wing"),
        ("B-Wing", "b-wing"),
        ("E-Wing", "e-wing"),
        ("HWK-290", "hwk-290"),
        ("Lancer Class Pursuit Craft", "lancer-pursuit-craft"),
        ("VCX-100 Freighter", "vcx-100-freighter"),
        ("Scurrg H-6 Bomber", "scurrg-h-6-bomber"),
        ("X-Wing", "x-wing"),
        ("Y-Wing", "y-wing"),
        ("YT-1300", "yt-1300"),
        ("YT-2400", "yt-2400"),
        ("Z-95 Headhunter", "z-95"),
        ("Arquitens Class Command Cruiser", "arquitens"),
        ("Gozanti Class Cruiser", "gozanti"),
        ("Imperial Star Destroyer", "imperial-class-star-destroyer"),
        ("Interdictor Class Destroyer", "interdictor"),
        ("Onager Class Destroyer", "onager"),
        ("Quasar Fire Class Cruiser Carrier", "quasar"),
        ("Raider Class Corvette", "raider-class-corvette"),
        ("Super Star Destroyer", "super-star-destroyer"),
        (
            "Victory Class Star Destroyer",
            "victory-class-star-destroyer",
        ),
        ("Aggressor Assault Fighter", "aggressor-assault-fighter"),
        ("Firespray-31", "firespray-31"),
        ("JumpMaster 5000", "jumpmaster-5000"),
        ("Lambda Class Shuttle", "lambda-shuttle"),
        ("Tie Advanced", "tie-advanced"),
        ("Tie Bomber", "tie-bomber"),
        ("Tie Defender", "tie-defender"),
        ("Tie Fighter", "tie-fighter"),
        ("Tie Interceptor", "tie-interceptor"),
        ("Tie Phantom", "tie-phantom"),
        ("VT-49 Decimator", "vt-49-decimator"),
        ("YV-666", "yv-666"),
    ];
    let expected: Vec<(&str, Option<&str>)> = expected
        .iter()
        .map(|(name, model)| (*name, Some(*model)))
        .collect();
    assert_eq!(models, expected);
}

#[test]
fn stock_reads_spaces_when_tabs_are_lost_and_skips_blank_lines() {
    let text = "Cyrille    Rebellion    X-Wing    -    Squadron    14\n\n  \nManu  Empire  Tie Fighter  -  Squadron  16\n";
    let stock = Stock::parse(text, &catalogue()).unwrap();
    assert_eq!(stock.lines.len(), 2);
    assert_eq!(stock.lines[0].owner, "Cyrille");
    assert_eq!(stock.lines[0].quantity, 14);
    assert_eq!(stock.lines[1].model.as_deref(), Some("tie-fighter"));
}

#[test]
fn unknown_figures_are_kept_unmatched() {
    let text = "Cyrille\tRebellion\tMillennium Pigeon\t-\tSquadron\t1\n";
    let stock = Stock::parse(text, &catalogue()).unwrap();
    assert_eq!(stock.unmatched().count(), 1);
}

#[test]
fn stock_errors_name_the_line() {
    let catalogue = catalogue();
    let parse = |text: &str| Stock::parse(text, &catalogue).unwrap_err();
    assert_eq!(
        parse("Owner\tFaction\tFigurine Name\tSize\tType\tQty\n"),
        StockParseError::Empty
    );
    assert_eq!(
        parse("Owner\tFaction\tFigurine Name\tSize\tType\tQty\nCyrille\tRebellion\tX-Wing\n"),
        StockParseError::Columns { line: 2, found: 3 }
    );
    assert_eq!(
        parse("Cyrille\tRebellion\tX-Wing\t-\tFighter\t2\n"),
        StockParseError::Kind {
            line: 1,
            value: "Fighter".to_owned()
        }
    );
    assert_eq!(
        parse(
            "Cyrille\tRebellion\tX-Wing\t-\tSquadron\t2\nCyrille\tRebellion\tY-Wing\t-\tSquadron\tsix\n"
        ),
        StockParseError::Quantity {
            line: 2,
            value: "six".to_owned()
        }
    );
}

#[test]
fn stock_sheet_round_trips() {
    let catalogue = catalogue();
    let stock = Stock::parse(STOCK, &catalogue).unwrap();
    assert_eq!(Stock::parse(&stock.to_sheet(), &catalogue).unwrap(), stock);
}

#[test]
fn roster_counts_models_and_unique_cards() {
    let roster = FleetRoster::from_page_html(FLEET_PAGE).unwrap();
    assert_eq!(roster.name, "Campagne 1 copy");
    assert_eq!(roster.total_points, 397);
    let count = |kind, model: &str| {
        roster
            .figures
            .iter()
            .find(|figure: &&FigureCount| figure.kind == kind && figure.model == model)
            .map(|figure| figure.count)
    };
    assert_eq!(count(FigureKind::Ship, "corvette"), Some(2));
    assert_eq!(count(FigureKind::Ship, "pelta"), Some(1));
    // Gold Squadron is a named Y-wing.
    assert_eq!(count(FigureKind::Squadron, "y-wing"), Some(3));
    assert_eq!(count(FigureKind::Squadron, "z-95"), Some(2));
    assert_eq!(roster.figure_total(FigureKind::Ship), 5);
    assert_eq!(roster.figure_total(FigureKind::Squadron), 11);
    assert_eq!(
        roster.unique_cards,
        [
            "Dodonna's Pride",
            "General Dodonna",
            "Gold Squadron",
            "Green Squadron"
        ]
    );
}

#[test]
fn report_finds_shortfalls_and_duplicates() {
    let catalogue = catalogue();
    let stock = Stock::parse(STOCK, &catalogue).unwrap();
    let roster = FleetRoster::from_page_html(FLEET_PAGE).unwrap();
    let fleets = vec![
        FleetResult {
            fleet: fleet(1, "Alice"),
            roster: Ok(roster.clone()),
        },
        FleetResult {
            fleet: fleet(2, "Bob"),
            roster: Ok(roster),
        },
    ];
    let report = GameReport::build(&stock, fleets, Some(&catalogue));

    let usage = |model: &str| {
        report
            .models
            .iter()
            .find(|usage| usage.model == model)
            .unwrap()
    };
    // Two fleets with one Pelta each, one in stock.
    assert_eq!((usage("pelta").used, usage("pelta").stock), (2, 1));
    assert_eq!(usage("pelta").shortfall(), 1);
    assert_eq!(usage("corvette").shortfall(), 0);
    assert_eq!(usage("x-wing").used, 0);
    assert!(report.has_shortfall());

    let duplicates: Vec<(&str, Vec<&str>)> = report
        .duplicates
        .iter()
        .map(|card| {
            (
                card.name.as_str(),
                card.players.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(duplicates.len(), 4);
    assert_eq!(duplicates[0], ("Dodonna's Pride", vec!["Alice", "Bob"]));

    let ships = report.totals[0];
    assert_eq!(
        (ships.kind, ships.used, ships.stock),
        (FigureKind::Ship, 10, 39)
    );
}

#[test]
fn report_names_unstocked_models_from_catalogue() {
    let roster = FleetRoster::from_page_html(FLEET_PAGE).unwrap();
    let fleets = vec![FleetResult {
        fleet: fleet(1, "Alice"),
        roster: Ok(roster),
    }];
    let report = GameReport::build(&Stock::default(), fleets, Some(&catalogue()));
    let pelta = report
        .models
        .iter()
        .find(|usage| usage.model == "pelta")
        .unwrap();
    assert_eq!(pelta.name, "Pelta Assault Ship / Pelta Command Ship");
    assert!(report.duplicates.is_empty());
}

#[test]
fn replacing_a_fleet_keeps_its_place() {
    let mut game = Game::new("Campagne".to_owned());
    let first = game.add_fleet("Alice".to_owned(), "a".to_owned());
    let second = game.add_fleet("Bob".to_owned(), "b".to_owned());

    assert!(game.replace_fleet(first, "Carol".to_owned(), "c".to_owned()));
    assert_eq!(
        (
            game.fleets[0].id,
            game.fleets[0].player.as_str(),
            game.fleets[0].url.as_str()
        ),
        (first, "Carol", "c")
    );
    assert_eq!(game.fleets[1].id, second);
    assert!(!game.replace_fleet(99, "Dan".to_owned(), "d".to_owned()));
}

#[tokio::test]
async fn store_saves_and_updates_games() {
    let dir = temp_dir();
    let store = GameStore::new(dir.clone());
    let id = store
        .create(&Game::new("Campagne".to_owned()))
        .await
        .unwrap();

    let fleet_id = store
        .update(&id, |game| {
            game.add_fleet(
                "Alice".to_owned(),
                "https://armada.ryankingston.com/fleet/1/".to_owned(),
            )
        })
        .await
        .unwrap();
    let game = store.load(&id).await.unwrap();
    assert_eq!(game.name, "Campagne");
    assert_eq!(game.fleets[0].id, fleet_id);

    let other = Game::new("Autre".to_owned());
    let other_id = store.create(&other).await.unwrap();
    assert_ne!(other_id, id);
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(matches!(
        store.load(&id).await,
        Err(GameStoreError::NotFound)
    ));
}
