//! Text of the home page.

use crate::lang::Lang;

/// Text of the home page. Fields marked HTML are trusted markup rendered unescaped.
#[derive(Debug)]
pub(crate) struct IndexText {
    /// Warning that a builder account is needed (HTML).
    pub account_warning: &'static str,
    /// Steps to print a fleet (HTML).
    pub steps: [&'static str; 6],
    /// Print button.
    pub submit: &'static str,
    /// Portrait orientation choice.
    pub portrait: &'static str,
    /// Landscape orientation choice.
    pub landscape: &'static str,
    /// Heading of the game section.
    pub games_heading: &'static str,
    /// What a game is for.
    pub games_intro: &'static str,
    /// Hint of the game name field.
    pub game_name_placeholder: &'static str,
    /// Game creation button.
    pub create_game: &'static str,
}

/// Link to the fleet builder, as a literal so `concat!` can use it.
macro_rules! builder_link {
    () => {
        r#"<a href="https://armada.ryankingston.com/" target="_blank">https://armada.ryankingston.com/</a>"#
    };
}

const FR: IndexText = IndexText {
    account_warning: concat!(
        "Pour utiliser ce site, il faut impérativement avoir créé un compte sur ",
        builder_link!(),
    ),
    steps: [
        concat!("Allez sur le site ", builder_link!(),),
        "Créez une flotte.",
        "Sauvegardez-la (IMPORTANT !).",
        "Copiez le lien dans la barre d'url.",
        "Collez-le en dessous, choisissez l'orientation des pages et cliquez sur « Envoyer » \
         (ça peut prendre du temps : 30-60 secondes au max).",
        "Imprimez ou sauvegardez le pdf généré.",
    ],
    submit: "Envoyer",
    portrait: "Portrait",
    landscape: "Paysage",
    games_heading: "Partie à plusieurs joueurs",
    games_intro: "Rassemblez les flottes de plusieurs joueurs et le stock de figurines de chacun \
                  pour vérifier qu'il y a assez de figurines et qu'aucune carte unique n'est prise \
                  deux fois. La partie créée n'est accessible que par son lien : gardez-le et \
                  partagez-le avec les autres joueurs.",
    game_name_placeholder: "Nom de la partie",
    create_game: "Créer la partie",
};

const EN: IndexText = IndexText {
    account_warning: concat!(
        "To use this site, you must have an account on ",
        builder_link!(),
    ),
    steps: [
        concat!("Go to ", builder_link!(),),
        "Build a fleet.",
        "Save it (IMPORTANT!).",
        "Copy the link from the address bar.",
        "Paste it below, choose the page orientation and click “Send” (this can take a while: \
         30-60 seconds at most).",
        "Print or save the generated PDF.",
    ],
    submit: "Send",
    portrait: "Portrait",
    landscape: "Landscape",
    games_heading: "Multi-player game",
    games_intro: "Gather the fleets of several players and the miniature stock of each to check \
                  that there are enough miniatures and that no unique card is taken twice. The \
                  game can only be reached through its link: keep it and share it with the other \
                  players.",
    game_name_placeholder: "Game name",
    create_game: "Create the game",
};

impl IndexText {
    /// The text in `lang`.
    pub(crate) fn of(lang: Lang) -> &'static Self {
        match lang {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }
}
