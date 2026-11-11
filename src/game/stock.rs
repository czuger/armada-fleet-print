//! Miniatures the players own, pasted from a spreadsheet.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::game::faction::Faction;
use crate::game::figure_kind::FigureKind;
use crate::game::model_catalogue::ModelCatalogue;

/// Columns of a stock sheet, in order: Owner, Faction, Figurine Name, Size, Type, Qty.
const COLUMNS: usize = 6;
/// Most lines accepted in one paste.
pub const MAX_STOCK_LINES: usize = 1000;

/// Cell separator when a paste lost its tabs: two spaces or more.
// safe: the pattern is a compile-time constant.
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());

/// The miniatures of every player of a game.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock {
    /// Sheet lines, in pasted order.
    pub lines: Vec<StockLine>,
}

/// One line of the stock sheet: a quantity of one miniature owned by one player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockLine {
    /// Player owning the miniatures.
    pub owner: String,
    /// Faction as written in the sheet.
    pub faction: String,
    /// Miniature name as written in the sheet.
    pub name: String,
    /// Size as written in the sheet, kept only to write the sheet back.
    pub size: String,
    /// Ship or squadron.
    pub kind: FigureKind,
    /// Number of miniatures.
    pub quantity: u32,
    /// Builder model the name was matched to, `None` if it matched none.
    pub model: Option<String>,
}

/// Error returned when a pasted stock cannot be read. Line numbers start at 1.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StockParseError {
    /// The paste has no miniature line.
    #[error("stock is empty")]
    Empty,
    /// The paste has more than [`MAX_STOCK_LINES`] lines.
    #[error("stock has more than {MAX_STOCK_LINES} lines")]
    TooManyLines,
    /// A line has fewer than six cells.
    #[error("line {line}: expected {COLUMNS} columns, found {found}")]
    Columns {
        /// Line number.
        line: usize,
        /// Number of cells found.
        found: usize,
    },
    /// The Type cell is neither ship nor squadron.
    #[error("line {line}: unknown type {value:?}")]
    Kind {
        /// Line number.
        line: usize,
        /// Cell content.
        value: String,
    },
    /// The Qty cell is not a whole number.
    #[error("line {line}: invalid quantity {value:?}")]
    Quantity {
        /// Line number.
        line: usize,
        /// Cell content.
        value: String,
    },
}

impl Stock {
    /// Reads a stock pasted from a spreadsheet and matches each name to a builder model.
    ///
    /// Cells are separated by tabs, as spreadsheets copy them, or by two spaces or more. A
    /// first line with neither a known type nor a quantity is taken as the header and skipped. Blank
    /// lines are ignored.
    ///
    /// # Arguments
    /// * `text` - Pasted rows with the columns Owner, Faction, Figurine Name, Size, Type, Qty.
    /// * `catalogue` - Builder models the names are matched against.
    ///
    /// # Errors
    /// Returns [`StockParseError`] for the first unreadable line, or if no line is left.
    pub fn parse(text: &str, catalogue: &ModelCatalogue) -> Result<Self, StockParseError> {
        let rows: Vec<(usize, Vec<&str>)> = text
            .lines()
            .enumerate()
            .map(|(index, line)| (index + 1, split_cells(line)))
            .filter(|(_, cells)| !cells.is_empty())
            .collect();
        if rows.len() > MAX_STOCK_LINES {
            return Err(StockParseError::TooManyLines);
        }
        let data_rows = match rows.split_first() {
            Some(((_, header), rest)) if is_header(header) => rest,
            _ => &rows[..],
        };
        let lines = data_rows
            .iter()
            .map(|(number, cells)| parse_line(*number, cells, catalogue))
            .collect::<Result<Vec<_>, _>>()?;
        if lines.is_empty() {
            return Err(StockParseError::Empty);
        }
        Ok(Self { lines })
    }

    /// Writes the stock back as tab-separated rows with a header, ready to edit and paste again.
    pub fn to_sheet(&self) -> String {
        let mut sheet = String::from("Owner\tFaction\tFigurine Name\tSize\tType\tQty\n");
        for line in &self.lines {
            sheet.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                line.owner,
                line.faction,
                line.name,
                line.size,
                line.kind.sheet_label(),
                line.quantity
            ));
        }
        sheet
    }

    /// Lines whose name matched no builder model.
    pub fn unmatched(&self) -> impl Iterator<Item = &StockLine> {
        self.lines.iter().filter(|line| line.model.is_none())
    }
}

/// Trimmed cells of `line`, without trailing empty cells.
fn split_cells(line: &str) -> Vec<&str> {
    let mut cells: Vec<&str> = if line.contains('\t') {
        line.split('\t').map(str::trim).collect()
    } else {
        SPACES.split(line.trim()).collect()
    };
    while cells.last().is_some_and(|cell| cell.is_empty()) {
        cells.pop();
    }
    cells
}

/// A header row has neither a known type nor a number where data rows have them.
fn is_header(cells: &[&str]) -> bool {
    let not_a_kind = cells
        .get(COLUMNS - 2)
        .is_some_and(|kind| FigureKind::from_sheet(kind).is_none());
    let not_a_quantity = cells
        .get(COLUMNS - 1)
        .is_some_and(|quantity| quantity.parse::<u32>().is_err());
    not_a_kind && not_a_quantity
}

fn parse_line(
    number: usize,
    cells: &[&str],
    catalogue: &ModelCatalogue,
) -> Result<StockLine, StockParseError> {
    let [owner, faction, name, size, kind, quantity, ..] = cells else {
        return Err(StockParseError::Columns {
            line: number,
            found: cells.len(),
        });
    };
    let kind = FigureKind::from_sheet(kind).ok_or_else(|| StockParseError::Kind {
        line: number,
        value: (*kind).to_owned(),
    })?;
    let quantity = quantity.parse().map_err(|_| StockParseError::Quantity {
        line: number,
        value: (*quantity).to_owned(),
    })?;
    let model = catalogue
        .resolve(name, Faction::from_sheet(faction), kind)
        .map(|model| model.id.clone());
    Ok(StockLine {
        owner: (*owner).to_owned(),
        faction: (*faction).to_owned(),
        name: (*name).to_owned(),
        size: (*size).to_owned(),
        kind,
        quantity,
        model,
    })
}
