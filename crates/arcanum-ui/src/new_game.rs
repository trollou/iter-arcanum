//! Écran de nouvelle partie (specs 065 et 078) : assistant en deux étapes, Scénario puis Héros,
//! avec la liste à gauche et la fiche à droite, calculé sans egui.

use arcanum_core::{
    BasicAction, BasicColor, CardId, DeedCard, DummyHeroChoice, DummyRule, GameConfig, Hero,
    Scenario, ScenarioId,
};
use arcanum_i18n::Catalog;
use arcanum_persistance::SaveSummary;
use eframe::egui::Color32;

use crate::controls::summary_without_date;
use crate::screen::{card_effect_texts, card_strong_text};
use crate::{
    Button, Command, Illustration, ImageId, Shortcut, hero_color, hero_illustration, hero_image,
    shortcut_label,
};

/// La plus grande graine, `u64::MAX`, a 20 chiffres (L7).
pub const SEED_MAX_CHARS: usize = 20;

/// Étape de l'écran (spec 078) : une seule montrée à la fois, dans l'ordre du livret (MKUE p. 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NewGameStep {
    Scenario,
    Hero,
    /// Spec 061 : les options du scénario (le joueur fantôme).
    Options,
}

impl NewGameStep {
    pub const ALL: [NewGameStep; 3] = [
        NewGameStep::Scenario,
        NewGameStep::Hero,
        NewGameStep::Options,
    ];
}

/// Ce que le joueur a coché et saisi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewGameForm {
    pub hero: Hero,
    pub scenario: ScenarioId,
    /// Texte du champ de la graine, tel que saisi.
    pub seed: String,
    /// Étape affichée (spec 078).
    pub step: NewGameStep,
    /// Volet « Avancé » ouvert (spec 078, C13).
    pub advanced_open: bool,
    /// Spec 061, Q1 : avec le joueur fantôme, si le scénario en a un.
    pub dummy: bool,
    /// Spec 061, Q2 : le héros du fantôme, au hasard par défaut.
    pub dummy_hero: DummyHeroChoice,
}

impl Default for NewGameForm {
    /// C1 : Arythea, Première reconnaissance, graine vide, étape Scénario, volet fermé.
    fn default() -> Self {
        Self {
            hero: Hero::Arythea,
            scenario: ScenarioId::FirstReconnaissance,
            seed: String::new(),
            step: NewGameStep::Scenario,
            advanced_open: false,
            dummy: true,
            dummy_hero: DummyHeroChoice::Random,
        }
    }
}

/// La graine saisie n'est pas un entier de 0 à `u64::MAX` (C8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidSeed;

/// Graine du champ : `None` s'il est vide (tirée de l'horloge), espaces autour ignorés (C7).
/// Seuls les chiffres sont acceptés : ni signe, ni virgule.
pub fn parse_seed(text: &str) -> Result<Option<u64>, InvalidSeed> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(InvalidSeed);
    }
    text.parse().map(Some).map_err(|_| InvalidSeed)
}

/// D'où s'ouvre l'écran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewGameContext {
    /// Au lancement, sans partie : le résumé de la sauvegarde automatique, si sa partie n'est
    /// pas terminée (C15).
    Launch { autosave: Option<SaveSummary> },
    /// Par-dessus une partie ; `finished` : elle est terminée, rien ne sera sauvegardé (C14).
    InGame { finished: bool },
}

/// Un héros de la liste, avec son illustration et son accroche (spec 078, C8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroChoice {
    pub hero: Hero,
    pub name: String,
    pub tagline: String,
    /// Portrait généré (C9b), à la place de `image` quand il est là.
    pub portrait: Illustration,
    pub image: ImageId,
    pub fallback: Color32,
    pub selected: bool,
    pub command: Command,
}

/// Un scénario de la liste, avec son accroche (spec 078, C2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioChoice {
    pub scenario: ScenarioId,
    pub name: String,
    pub tagline: String,
    pub selected: bool,
    pub command: Command,
}

/// Un onglet de la barre d'étapes (C1, C7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepTab {
    pub step: NewGameStep,
    /// 1 ou 2, dessiné dans la pastille.
    pub number: u8,
    pub label: String,
    pub active: bool,
    pub command: Command,
}

/// Un fait de la fiche du scénario : « Durée », « 4 manches : … » (C3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetFact {
    pub label: String,
    pub value: String,
}

/// Une rubrique de fiche : un titre et ses puces (C3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetSection {
    pub title: String,
    pub bullets: Vec<String>,
}

/// La fiche du scénario coché (C3, C4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioSheet {
    pub name: String,
    pub flavor: String,
    /// Durée, Joueurs, Type.
    pub facts: Vec<SheetFact>,
    /// Objectif, Règles spéciales, Fin de partie, Décompte.
    pub sections: Vec<SheetSection>,
    /// « Pas encore dans le jeu : … », s'il y a lieu.
    pub missing: Option<String>,
}

/// Une carte propre sur la fiche du héros (C10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnCardSheet {
    pub card: BasicAction,
    pub name: String,
    /// « à la place de Rage ».
    pub replaces: String,
    pub color: BasicColor,
    pub basic_title: String,
    pub basic: String,
    /// « Renforcé (mana rouge) ».
    pub strong_title: String,
    pub strong: String,
}

/// La fiche du héros coché (C9, C10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroSheet {
    pub name: String,
    /// Portrait généré (C9b), à la place de `image` quand il est là.
    pub portrait: Illustration,
    pub image: ImageId,
    pub fallback: Color32,
    pub description: String,
    /// « Style de jeu » et ses puces.
    pub style: SheetSection,
    pub own_cards_title: String,
    pub own_cards: Vec<OwnCardSheet>,
}

/// Un héros de la liste déroulante du fantôme : « Au hasard », puis les trois autres (spec 061).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummyHeroItem {
    pub choice: DummyHeroChoice,
    pub label: String,
    pub selected: bool,
    pub command: Command,
}

/// La case du joueur fantôme et la liste de son héros (spec 061, C41).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummyOption {
    /// « Joueur fantôme », titre de la case et de l'aide.
    pub label: String,
    /// Spec 064, Q3 : fantôme obligatoire, « Ce scénario se joue avec le joueur fantôme. » à la
    /// place de la case.
    pub required: Option<String>,
    pub checked: bool,
    pub command: Command,
    /// Fiche de droite : ce qu'il fait, et le conseil pour une toute première partie.
    pub help: String,
    /// « Héros du joueur fantôme ».
    pub hero_label: String,
    /// Le libellé du choix affiché.
    pub hero_selected: String,
    /// Grisée quand la case est décochée.
    pub hero_enabled: bool,
    pub heroes: Vec<DummyHeroItem>,
}

/// L'étape Options (spec 061) : la case du fantôme, ou « Pas d'option pour ce scénario. ».
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionsSheet {
    pub dummy: Option<DummyOption>,
    pub none: Option<String>,
}

/// La fiche de droite, selon l'étape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NewGameSheet {
    Scenario(ScenarioSheet),
    Hero(HeroSheet),
    /// Spec 061 : la liste (case et héros) et l'aide, dessinées ensemble.
    Options(OptionsSheet),
}

/// Le volet « Avancé » (C13, C14).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedPanel {
    pub open: bool,
    /// « Avancé », ou « Avancé (graine saisie) » refermé sur une graine.
    pub label: String,
    /// Refermer est refusé tant que la graine est invalide (C14).
    pub can_close: bool,
    pub command: Command,
}

/// Bouton « Commencer », grisé si la graine est invalide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartButton {
    pub label: String,
    pub enabled: bool,
    pub command: Command,
    /// « Entrée » (spec 071, C23).
    pub shortcut: Option<String>,
}

/// L'écran de nouvelle partie, déjà traduit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewGameScreen {
    pub title: String,
    pub step: NewGameStep,
    /// La barre d'étapes : Scénario puis Héros.
    pub steps: Vec<StepTab>,
    /// « Votre partie : ».
    pub recap_label: String,
    /// « Première reconnaissance · Arythea ».
    pub recap: String,
    /// « Scénarios » ou « Héros », selon l'étape.
    pub list_title: String,
    /// Toujours calculées ; la liste de l'étape est dessinée.
    pub scenarios: Vec<ScenarioChoice>,
    pub heroes: Vec<HeroChoice>,
    pub sheet: NewGameSheet,
    /// « Suivant : choisir le héros », à l'étape Scénario seulement (C6).
    pub next: Option<Button>,
    pub advanced: AdvancedPanel,
    pub seed_label: String,
    pub seed_hint: String,
    /// En rouge sous le champ quand la graine est invalide (C8).
    pub seed_invalid: Option<String>,
    /// En cours de partie non terminée : elle sera sauvegardée, puis remplacée.
    pub abandon_warning: Option<String>,
    /// Échec de la sauvegarde de la partie abandonnée (L2) ; rempli par la session.
    pub notice: Option<String>,
    pub start: StartButton,
    /// En cours de partie seulement (L3).
    pub cancel: Option<Button>,
    /// Au lancement, quand la sauvegarde automatique a une partie non terminée.
    pub continue_game: Option<Button>,
    /// Au lancement seulement.
    pub load: Option<Button>,
    /// « Préférences… », au lancement seulement, à côté de « Charger… » (spec 080, C23).
    pub preferences: Option<Button>,
}

pub fn new_game_screen(
    form: &NewGameForm,
    context: &NewGameContext,
    catalog: &Catalog,
) -> NewGameScreen {
    let heroes = Hero::ALL
        .into_iter()
        .map(|hero| HeroChoice {
            hero,
            name: catalog.text(hero.label_key()),
            tagline: catalog.text(&sheet_key(hero.label_key(), "tagline")),
            portrait: hero_illustration(hero),
            image: hero_image(hero),
            fallback: hero_color(hero),
            selected: form.hero == hero,
            command: Command::ChooseHero(hero),
        })
        .collect();
    let scenarios = ScenarioId::ALL
        .into_iter()
        .map(|scenario| ScenarioChoice {
            scenario,
            name: catalog.text(scenario.label_key()),
            tagline: catalog.text(&sheet_key(scenario.label_key(), "tagline")),
            selected: form.scenario == scenario,
            command: Command::ChooseScenario(scenario),
        })
        .collect();
    let valid = parse_seed(&form.seed).is_ok();
    let button = |key: &str, command| Button {
        label: catalog.text(key),
        command,
        shortcut: None,
    };
    let (abandon_warning, cancel, continue_game, load) = match context {
        NewGameContext::Launch { autosave } => (
            None,
            None,
            autosave.as_ref().map(|summary| Button {
                label: catalog.format(
                    "new_game.continue",
                    &[("summary", &summary_without_date(summary, catalog))],
                ),
                command: Command::ContinueGame,
                shortcut: None,
            }),
            Some(button("new_game.load", Command::OpenLoad)),
        ),
        NewGameContext::InGame { finished } => (
            (!finished).then(|| catalog.text("new_game.abandon_warning")),
            Some(Button {
                shortcut: shortcut_label(Shortcut::Cancel, catalog),
                ..button("new_game.cancel", Command::CancelNewGame)
            }),
            None,
            None,
        ),
    };
    NewGameScreen {
        title: catalog.text("new_game.title"),
        step: form.step,
        steps: step_tabs(form.step, catalog),
        recap_label: catalog.text("new_game.recap"),
        recap: recap(form, catalog),
        list_title: catalog.text(match form.step {
            NewGameStep::Scenario => "new_game.scenarios",
            NewGameStep::Hero => "new_game.heroes",
            NewGameStep::Options => "new_game.step_options",
        }),
        scenarios,
        heroes,
        sheet: match form.step {
            NewGameStep::Scenario => NewGameSheet::Scenario(scenario_sheet(form.scenario, catalog)),
            NewGameStep::Hero => NewGameSheet::Hero(hero_sheet(form.hero, catalog)),
            NewGameStep::Options => NewGameSheet::Options(options_sheet(form, catalog)),
        },
        next: (form.step == NewGameStep::Scenario).then(|| Button {
            label: catalog.text("new_game.next"),
            command: Command::ChooseStep(NewGameStep::Hero),
            shortcut: None,
        }),
        advanced: advanced_panel(form, valid, catalog),
        seed_label: catalog.text("new_game.seed"),
        seed_hint: catalog.text("new_game.seed_hint"),
        seed_invalid: (!valid).then(|| catalog.text("new_game.seed_invalid")),
        abandon_warning,
        notice: None,
        start: StartButton {
            label: catalog.text("new_game.start"),
            enabled: valid,
            command: Command::StartNewGame,
            shortcut: shortcut_label(Shortcut::Confirm, catalog),
        },
        cancel,
        continue_game,
        load,
        // Spec 080, C23 : au lancement, à côté de « Charger… ».
        preferences: matches!(context, NewGameContext::Launch { .. }).then(|| Button {
            shortcut: shortcut_label(Shortcut::Preferences, catalog),
            ..button("preferences.button", Command::OpenPreferences)
        }),
    }
}

/// La barre d'étapes : un onglet par étape, l'affichée active (C1, C7).
fn step_tabs(current: NewGameStep, catalog: &Catalog) -> Vec<StepTab> {
    NewGameStep::ALL
        .into_iter()
        .zip(1..)
        .map(|(step, number)| StepTab {
            step,
            number,
            label: catalog.text(match step {
                NewGameStep::Scenario => "new_game.scenario",
                NewGameStep::Hero => "new_game.hero",
                NewGameStep::Options => "new_game.step_options",
            }),
            active: step == current,
            command: Command::ChooseStep(step),
        })
        .collect()
}

/// « Première reconnaissance · Tovak », suivi au besoin de « · sans joueur fantôme » ou de
/// « · joueur fantôme : Goldyx » (spec 061, Libellés).
fn recap(form: &NewGameForm, catalog: &Catalog) -> String {
    let base = catalog.format(
        "new_game.recap_choice",
        &[
            ("scenario", &catalog.text(form.scenario.label_key())),
            ("hero", &catalog.text(form.hero.label_key())),
        ],
    );
    let extra = match (has_dummy(form.scenario), with_dummy(form), dummy_hero(form)) {
        (true, false, _) => Some(catalog.text("new_game.recap_no_dummy")),
        (true, true, DummyHeroChoice::Hero(hero)) => Some(catalog.format(
            "new_game.recap_dummy_hero",
            &[("hero", &catalog.text(hero.label_key()))],
        )),
        _ => None,
    };
    // Le séparateur « · » est celui du récapitulatif : le même libellé, appliqué une fois de plus.
    match extra {
        Some(extra) => catalog.format(
            "new_game.recap_choice",
            &[("scenario", &base), ("hero", &extra)],
        ),
        None => base,
    }
}

/// Le scénario a un joueur fantôme, que l'étape Options peut retirer (spec 061, Q1).
fn has_dummy(scenario: ScenarioId) -> bool {
    Scenario::of(scenario).dummy() != DummyRule::None
}

/// Le fantôme est de la partie : case cochée, ou fantôme obligatoire (spec 064, Q3).
fn with_dummy(form: &NewGameForm) -> bool {
    form.dummy || !Scenario::of(form.scenario).dummy_optional()
}

/// Le héros du fantôme du formulaire, ramené à « Au hasard » s'il est celui du joueur (L10).
pub fn dummy_hero(form: &NewGameForm) -> DummyHeroChoice {
    match form.dummy_hero {
        DummyHeroChoice::Hero(hero) if hero == form.hero => DummyHeroChoice::Random,
        choice => choice,
    }
}

/// L'étape Options (spec 061, C41) : la case du fantôme et la liste de son héros (« Au hasard »,
/// puis les héros autres que celui du joueur, dans l'ordre de `Hero::ALL`).
fn options_sheet(form: &NewGameForm, catalog: &Catalog) -> OptionsSheet {
    if !has_dummy(form.scenario) {
        return OptionsSheet {
            dummy: None,
            none: Some(catalog.text("new_game.options.none")),
        };
    }
    let current = dummy_hero(form);
    let label = |choice: DummyHeroChoice| match choice {
        DummyHeroChoice::Random => catalog.text("new_game.options.dummy_hero_random"),
        DummyHeroChoice::Hero(hero) => catalog.text(hero.label_key()),
    };
    let heroes = std::iter::once(DummyHeroChoice::Random)
        .chain(
            Hero::ALL
                .into_iter()
                .filter(|hero| *hero != form.hero)
                .map(DummyHeroChoice::Hero),
        )
        .map(|choice| DummyHeroItem {
            choice,
            label: label(choice),
            selected: choice == current,
            command: Command::ChooseDummyHero(choice),
        })
        .collect();
    let required = !Scenario::of(form.scenario).dummy_optional();
    OptionsSheet {
        dummy: Some(DummyOption {
            label: catalog.text("new_game.options.dummy"),
            required: required.then(|| catalog.text("new_game.options.dummy_required")),
            checked: with_dummy(form),
            command: Command::ToggleDummy,
            // L'aide d'habitude conseille de jouer sans lui : fausse quand il est obligatoire.
            help: catalog.text(if required {
                "new_game.options.dummy_required"
            } else {
                "new_game.options.dummy_help"
            }),
            hero_label: catalog.text("new_game.options.dummy_hero"),
            hero_selected: label(current),
            hero_enabled: with_dummy(form),
            heroes,
        }),
        none: None,
    }
}

/// La configuration de la partie du formulaire : scénario sans fantôme si la case est décochée
/// (spec 061, Q1), héros du fantôme choisi (Q2).
pub fn form_config(form: &NewGameForm, seed: u64) -> GameConfig {
    let scenario = Scenario::of(form.scenario);
    GameConfig {
        seed,
        scenario: if with_dummy(form) {
            scenario
        } else {
            scenario.without_dummy()
        },
        hero: form.hero,
        dummy_hero: dummy_hero(form),
    }
}

/// Le formulaire qui reprend le scénario et le héros d'une partie (spec 078, C11, C16) ; les
/// options du fantôme repartent de leur défaut, case cochée et « Au hasard » (spec 061, C41).
pub fn form_of(config: &GameConfig) -> NewGameForm {
    NewGameForm {
        hero: config.hero,
        scenario: config.scenario.id(),
        ..NewGameForm::default()
    }
}

/// Le volet « Avancé » : ouvert d'office tant que la graine est invalide (C14) ; refermé sur une
/// graine, son libellé le dit (C13).
fn advanced_panel(form: &NewGameForm, valid: bool, catalog: &Catalog) -> AdvancedPanel {
    let open = form.advanced_open || !valid;
    let filled = !open && !form.seed.trim().is_empty();
    AdvancedPanel {
        open,
        label: catalog.text(if filled {
            "new_game.advanced_filled"
        } else {
            "new_game.advanced"
        }),
        can_close: valid,
        command: Command::ToggleAdvanced,
    }
}

/// Champs de la fiche d'un scénario, dans l'ordre de l'écran : faits, puis rubriques.
const SCENARIO_FACTS: [(&str, &str); 3] = [
    ("new_game.fact_length", "length"),
    ("new_game.fact_players", "players"),
    ("new_game.fact_kind", "kind"),
];
const SCENARIO_SECTIONS: [(&str, &str); 4] = [
    ("new_game.objective", "objective"),
    ("new_game.special_rules", "special_rules"),
    ("new_game.game_end", "game_end"),
    ("new_game.scoring", "scoring"),
];
const SCENARIO_TEXTS: [&str; 2] = ["tagline", "flavor"];
const HERO_TEXTS: [&str; 3] = ["tagline", "description", "style"];

/// Les scénarios qui ont une ligne « Pas encore dans le jeu » (spec 078, Modèle) : plus aucun
/// depuis le joueur fantôme (spec 061, C42).
fn has_missing_line(scenario: ScenarioId) -> bool {
    match scenario {
        ScenarioId::FirstReconnaissance | ScenarioId::SoloConquest | ScenarioId::Sandbox => false,
    }
}

/// La clé sœur du nom : `scenario.sandbox.name` et `flavor` donnent `scenario.sandbox.flavor`.
fn sheet_key(name_key: &str, field: &str) -> String {
    let group = name_key.strip_suffix(".name").unwrap_or(name_key);
    format!("{group}.{field}")
}

/// Une puce par ligne du libellé (spec 078, Libellés).
fn bullet_lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_owned).collect()
}

/// La fiche d'un scénario (C3, C4).
pub fn scenario_sheet(scenario: ScenarioId, catalog: &Catalog) -> ScenarioSheet {
    let key = |field| sheet_key(scenario.label_key(), field);
    ScenarioSheet {
        name: catalog.text(scenario.label_key()),
        flavor: catalog.text(&key("flavor")),
        facts: SCENARIO_FACTS
            .into_iter()
            .map(|(label, field)| SheetFact {
                label: catalog.text(label),
                value: catalog.text(&key(field)),
            })
            .collect(),
        sections: SCENARIO_SECTIONS
            .into_iter()
            .map(|(title, field)| SheetSection {
                title: catalog.text(title),
                bullets: bullet_lines(&catalog.text(&key(field))),
            })
            .collect(),
        missing: has_missing_line(scenario).then(|| catalog.text(&key("missing"))),
    }
}

/// La fiche d'un héros (C9, C10).
pub fn hero_sheet(hero: Hero, catalog: &Catalog) -> HeroSheet {
    let key = |field| sheet_key(hero.label_key(), field);
    HeroSheet {
        name: catalog.text(hero.label_key()),
        portrait: hero_illustration(hero),
        image: hero_image(hero),
        fallback: hero_color(hero),
        description: catalog.text(&key("description")),
        style: SheetSection {
            title: catalog.text("new_game.style"),
            bullets: bullet_lines(&catalog.text(&key("style"))),
        },
        own_cards_title: catalog.text("new_game.own_cards_title"),
        own_cards: hero
            .own_cards()
            .into_iter()
            .filter_map(|own| own_card_sheet(own.card, own.replaced, catalog))
            .collect(),
    }
}

/// Une carte propre : ses effets au texte de la fenêtre des offres, le renforcé sans son préfixe
/// (C10). Une carte sans définition de couleur n'en est pas une (contenu validé par arcanum-core).
fn own_card_sheet(
    card: BasicAction,
    replaced: BasicAction,
    catalog: &Catalog,
) -> Option<OwnCardSheet> {
    let id = CardId::new(DeedCard::Basic(card), 0);
    let color = id.card().definition()?.color;
    let (basic, _) = card_effect_texts(id, catalog);
    Some(OwnCardSheet {
        card,
        name: catalog.text(card.label_key()),
        replaces: catalog.format(
            "new_game.card_replaces",
            &[("card", &catalog.text(replaced.label_key()))],
        ),
        color,
        basic_title: catalog.text("new_game.card_basic"),
        basic,
        strong_title: catalog.format(
            "new_game.card_strong",
            &[("color", &catalog.text(mana_key(color)))],
        ),
        strong: card_strong_text(id, catalog),
    })
}

fn mana_key(color: BasicColor) -> &'static str {
    match color {
        BasicColor::Red => "mana.red",
        BasicColor::Blue => "mana.blue",
        BasicColor::White => "mana.white",
        BasicColor::Green => "mana.green",
    }
}

/// Les clés de fiche d'un scénario, sœurs de son nom : pour vérifier qu'aucune ne manque (C17).
pub fn sheet_keys_of_scenario(scenario: ScenarioId) -> Vec<String> {
    let fields = SCENARIO_TEXTS
        .into_iter()
        .chain(SCENARIO_FACTS.map(|(_, field)| field))
        .chain(SCENARIO_SECTIONS.map(|(_, field)| field))
        .chain(has_missing_line(scenario).then_some("missing"));
    fields
        .map(|field| sheet_key(scenario.label_key(), field))
        .collect()
}

/// Les clés de fiche d'un héros (C17).
pub fn sheet_keys_of_hero(hero: Hero) -> Vec<String> {
    HERO_TEXTS
        .into_iter()
        .map(|field| sheet_key(hero.label_key(), field))
        .collect()
}
