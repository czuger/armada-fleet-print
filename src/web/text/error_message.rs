//! Fixed error messages shown to users, in each language.

use crate::lang::Lang;

/// Error message without arguments, shown on the error page or the game page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorMessage {
    /// The link is not a fleet builder link.
    InvalidFleetLink,
    /// The builder answered that the fleet does not exist.
    FleetNotFound,
    /// The builder could not be reached.
    FleetUnreachable,
    /// The fleet to replace was removed from the game meanwhile.
    FleetRemoved,
    /// The page has no fleet, usually because it was never saved.
    NoFleetData,
    /// The fleet data could not be read.
    UnreadableFleet,
    /// Card images could not be downloaded.
    CardsUnavailable,
    /// Any other printing failure.
    PdfFailed,
    /// The game link names no game.
    GameNotFound,
    /// The game was created without a usable name; states `MAX_NAME_CHARS`.
    GameName,
    /// Any other game failure.
    GameFailed,
    /// The builder card list could not be downloaded.
    CatalogueUnreachable,
    /// Any other failure, such as a page that could not be rendered.
    Internal,
}

impl ErrorMessage {
    /// The message in `lang`, plain text.
    pub(crate) fn text(self, lang: Lang) -> &'static str {
        match lang {
            Lang::Fr => self.french(),
            Lang::En => self.english(),
        }
    }

    fn french(self) -> &'static str {
        match self {
            Self::InvalidFleetLink => {
                "Ce lien n'est pas une flotte de https://armada.ryankingston.com/. Copiez le lien \
                 complet de la flotte depuis la barre d'adresse."
            }
            Self::FleetNotFound => "Cette flotte n'existe pas. Vérifiez le lien.",
            Self::FleetRemoved => {
                "Cette flotte a été retirée de la partie entre-temps. Vous pouvez l'ajouter à nouveau."
            }
            Self::FleetUnreachable => {
                "Impossible de récupérer la flotte sur https://armada.ryankingston.com/. Réessayez \
                 dans quelques instants."
            }
            Self::NoFleetData => {
                "Aucune flotte trouvée à cette adresse. Vérifiez que la flotte a bien été sauvegardée."
            }
            Self::UnreadableFleet => "Les données de cette flotte sont illisibles.",
            Self::CardsUnavailable => {
                "Impossible de télécharger les images des cartes. Réessayez dans quelques instants."
            }
            Self::PdfFailed => "Erreur interne lors de la création du PDF.",
            Self::GameNotFound => "Cette partie n'existe pas. Vérifiez le lien.",
            Self::GameName => "Donnez un nom à la partie (100 caractères au plus).",
            Self::GameFailed => "Erreur interne lors du traitement de la partie.",
            Self::CatalogueUnreachable => {
                "Impossible de récupérer la liste des cartes sur https://armada.ryankingston.com/. \
                 Réessayez dans quelques instants."
            }
            Self::Internal => "Erreur interne.",
        }
    }

    fn english(self) -> &'static str {
        match self {
            Self::InvalidFleetLink => {
                "This link is not a fleet from https://armada.ryankingston.com/. Copy the full \
                 link of the fleet from the address bar."
            }
            Self::FleetNotFound => "This fleet does not exist. Check the link.",
            Self::FleetRemoved => {
                "This fleet was removed from the game in the meantime. You can add it again."
            }
            Self::FleetUnreachable => {
                "Could not get the fleet from https://armada.ryankingston.com/. Try again in a \
                 few moments."
            }
            Self::NoFleetData => {
                "No fleet found at this address. Check that the fleet has been saved."
            }
            Self::UnreadableFleet => "The data of this fleet cannot be read.",
            Self::CardsUnavailable => {
                "Could not download the card images. Try again in a few moments."
            }
            Self::PdfFailed => "Internal error while creating the PDF.",
            Self::GameNotFound => "This game does not exist. Check the link.",
            Self::GameName => "Give the game a name (100 characters at most).",
            Self::GameFailed => "Internal error while processing the game.",
            Self::CatalogueUnreachable => {
                "Could not get the card list from https://armada.ryankingston.com/. Try again in \
                 a few moments."
            }
            Self::Internal => "Internal error.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MAX_NAME_CHARS;

    #[test]
    fn game_name_message_states_the_limit() {
        for lang in Lang::ALL {
            assert!(
                ErrorMessage::GameName
                    .text(lang)
                    .contains(&MAX_NAME_CHARS.to_string())
            );
        }
    }
}
