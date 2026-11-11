//! Text of the game page.

use crate::lang::Lang;

/// Text of the game page.
#[derive(Debug)]
pub(crate) struct GameText {
    /// Link back to the home page.
    pub home_link: &'static str,
    /// Notice that the link is the only key to the game.
    pub link_notice: &'static str,
    /// Separator between a label and its value: " : " in French, ": " in English.
    pub colon: &'static str,
    /// Heading of the results.
    pub results: &'static str,
    /// Shown instead of the results when there is nothing to compare.
    pub results_empty: &'static str,
    /// Alert when some miniatures are missing.
    pub shortfall: &'static str,
    /// Alert when the stock is enough.
    pub enough: &'static str,
    /// Alert when no unique card is taken twice.
    pub no_duplicates: &'static str,
    /// Title of the list of unique cards taken twice.
    pub duplicates: &'static str,
    /// Heading of the fleets.
    pub fleets: &'static str,
    /// Shown when there is no fleet.
    pub no_fleets: &'static str,
    /// Player column and field.
    pub player: &'static str,
    /// Fleet column.
    pub fleet: &'static str,
    /// Points column.
    pub points: &'static str,
    /// Ships column and total.
    pub ships: &'static str,
    /// Squadrons column and total.
    pub squadrons: &'static str,
    /// Name shown for a fleet without one.
    pub unnamed: &'static str,
    /// Button removing a fleet.
    pub remove: &'static str,
    /// Confirmation asked before removing a fleet.
    pub remove_confirm: &'static str,
    /// Button adding a fleet.
    pub add: &'static str,
    /// Button opening the form to replace a fleet, and saving the replacement.
    pub replace: &'static str,
    /// Introduces the fleet being replaced, above the form.
    pub replacing: &'static str,
    /// Link leaving the replacement form.
    pub cancel: &'static str,
    /// Miniature column.
    pub miniature: &'static str,
    /// Used miniatures column.
    pub used: &'static str,
    /// Stock heading and column.
    pub stock: &'static str,
    /// Missing miniatures column.
    pub missing: &'static str,
    /// Introduces the players using a miniature.
    pub used_by: &'static str,
    /// Shown when there is no stock.
    pub no_stock: &'static str,
    /// Follows the number of stock lines.
    pub lines: &'static str,
    /// Title of the list of unrecognized miniatures.
    pub unmatched: &'static str,
    /// How to paste the stock.
    pub stock_help: &'static str,
    /// Button saving the stock.
    pub save_stock: &'static str,
}

const FR: GameText = GameText {
    home_link: "← Impression de flotte",
    link_notice: "Cette partie n'est accessible que par son lien. Toute personne qui l'a peut la \
                  consulter et la modifier.",
    colon: " : ",
    results: "Résultat",
    results_empty: "Ajoutez un stock et des flottes pour voir le résultat.",
    shortfall: "Il manque des figurines dans le stock (lignes en rouge).",
    enough: "Le stock suffit pour toutes les flottes.",
    no_duplicates: "Aucune carte unique en double.",
    duplicates: "Cartes uniques en double :",
    fleets: "Flottes",
    no_fleets: "Aucune flotte pour l'instant.",
    player: "Joueur",
    fleet: "Flotte",
    points: "Points",
    ships: "Vaisseaux",
    squadrons: "Escadrons",
    unnamed: "(sans nom)",
    remove: "Retirer",
    remove_confirm: "Retirer cette flotte de la partie ?",
    add: "Ajouter",
    replace: "Remplacer",
    replacing: "Flotte à remplacer",
    cancel: "Annuler",
    miniature: "Figurine",
    used: "Utilisées",
    stock: "Stock",
    missing: "Manque",
    used_by: "Utilisées par",
    no_stock: "Aucun stock pour l'instant.",
    lines: "lignes",
    unmatched: "Figurines non reconnues, ignorées dans le résultat (corrigez leur nom ou leur \
                type) :",
    stock_help: "Copiez les lignes du tableur (Google Sheets) avec les colonnes Owner, Faction, \
                 Figurine Name, Size, Type et Qty, puis collez-les ici. La ligne d'en-tête est \
                 facultative. Le nouveau stock remplace l'ancien.",
    save_stock: "Enregistrer le stock",
};

const EN: GameText = GameText {
    home_link: "← Fleet printing",
    link_notice: "This game can only be reached through its link. Anyone who has it can view and \
                  change it.",
    colon: ": ",
    results: "Result",
    results_empty: "Add a stock and fleets to see the result.",
    shortfall: "The stock is missing miniatures (rows in red).",
    enough: "The stock is enough for all the fleets.",
    no_duplicates: "No unique card is taken twice.",
    duplicates: "Unique cards taken more than once:",
    fleets: "Fleets",
    no_fleets: "No fleet yet.",
    player: "Player",
    fleet: "Fleet",
    points: "Points",
    ships: "Ships",
    squadrons: "Squadrons",
    unnamed: "(no name)",
    remove: "Remove",
    remove_confirm: "Remove this fleet from the game?",
    add: "Add",
    replace: "Replace",
    replacing: "Fleet to replace",
    cancel: "Cancel",
    miniature: "Miniature",
    used: "Used",
    stock: "Stock",
    missing: "Missing",
    used_by: "Used by",
    no_stock: "No stock yet.",
    lines: "lines",
    unmatched: "Unrecognized miniatures, left out of the result (fix their name or type):",
    stock_help: "Copy the spreadsheet rows (Google Sheets) with the columns Owner, Faction, \
                 Figurine Name, Size, Type and Qty, then paste them here. The header row is \
                 optional. The new stock replaces the old one.",
    save_stock: "Save the stock",
};

impl GameText {
    /// The text in `lang`.
    pub(crate) fn of(lang: Lang) -> &'static Self {
        match lang {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }
}
