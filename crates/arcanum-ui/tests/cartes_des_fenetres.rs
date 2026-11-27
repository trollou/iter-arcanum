//! Scénarios de la spec #32 (cartes des fenêtres) : la carte d'une fenêtre (Offres, choix de
//! l'artefact, choix du sort) se lit comme la carte de la main ; nombre de cartes par rangée.
//! Aucune règle du livret : c'est de l'affichage (mana du renforcé : MKUE p. 4, 3.d ; p. 5, 2.a).

use arcanum_core::{
    AdvancedAction, BasicColor, CardId, CommonCard, CommonDeck, DeedCard, Event, GameState,
    ManaColor, Offer, Spell, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    GameScreen, HandCard, OfferCardView, ScreenMode, ScreenPhase, cards_per_row, game_screen,
    offers_window, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use arcanum_core::BasicAction::*;

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        catalog,
    )
}

fn pure_magic() -> CardId {
    advanced(AdvancedAction::PureMagic)
}

fn ice_shield() -> CardId {
    advanced(AdvancedAction::IceShield)
}

fn fireball() -> CardId {
    CardId::new(DeedCard::Spell(Spell::Fireball), 1)
}

fn blue() -> ManaColor {
    ManaColor::Basic(BasicColor::Blue)
}

fn red() -> ManaColor {
    ManaColor::Basic(BasicColor::Red)
}

/// Le bac à sable avec Magie pure dans l'offre d'actions avancées, Éclair de feu dans l'offre de
/// sorts et des Paysans dans l'offre d'unités.
fn with_offers() -> GameState {
    let mut events = Sandbox {
        figure: Some(arcanum_core::Hex::new(0, 0)),
        hand: vec![card(March, 1)],
        ..Sandbox::default()
    }
    .events();
    // Une carte n'entre dans l'offre que tirée de sa pioche.
    for (deck, offer, card) in [
        (
            CommonDeck::AdvancedActions,
            Offer::AdvancedActions,
            pure_magic(),
        ),
        (CommonDeck::Spells, Offer::Spells, fireball()),
    ] {
        events.push(Event::CommonDeckShuffled {
            deck,
            order: vec![CommonCard::Deed(card)],
        });
        events.push(Event::OfferCardDealt {
            offer,
            card: CommonCard::Deed(card),
        });
    }
    events.extend(unit_events(&[], &[UnitCard::new(Unit::Peasants, 1)]));
    events.iter().fold(GameState::default(), apply)
}

#[track_caller]
fn offer_card(state: &GameState, column: usize, name: &str, catalog: &Catalog) -> OfferCardView {
    let window = offers_window(&view(state, SOLO), &legal_actions(state, SOLO), catalog);
    window.columns[column]
        .cards
        .iter()
        .find(|card| card.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("{name} absente de l'offre : {:?}", window.columns[column]))
}

/// La carte telle que la main la montre, seule en main avec une Marche (spec #11).
fn held_alone(card: CardId, catalog: &Catalog) -> HandCard {
    let state = state_with(&[card, self::card(March, 1)]);
    screen_of(&state, catalog)
        .hand
        .into_iter()
        .find(|held| held.card == card)
        .expect("carte en main")
}

// --- Fenêtre Offres ---

// Spec #32 C1 : une action avancée de l'offre, le texte du « i » et la séparation de la main.
#[test]
fn offer_advanced_action_reads_like_hand_card() {
    let catalog = french();
    let card = offer_card(&with_offers(), 0, "Magie pure", &catalog);
    assert_eq!(card.color, Some(BasicColor::Blue));
    assert_eq!(
        card.lines,
        [
            "Payer un mana : vert Mouvement 4, blanc Influence 4, bleu Parade 4, rouge Attaque 4",
            "Payer un mana : vert Mouvement 7, blanc Influence 7, bleu Parade 7, rouge Attaque 7",
        ]
    );
    assert_eq!(card.strong_heading, "Renforcé");
    assert_eq!(card.strong_colors, [blue()]);
}

// Spec #32 C2 : un sort de l'offre, son nom de base ; le nom renforcé dans la séparation.
#[test]
fn offer_spell_names_its_strong_effect_apart() {
    let catalog = french();
    let card = offer_card(&with_offers(), 1, "Éclair de feu", &catalog);
    assert_eq!(
        card.lines,
        [
            "Attaque à distance de feu 5",
            "Blessure ; Attaque de siège de feu 8"
        ]
    );
    assert_eq!(card.strong_heading, "Tempête de feu");
    assert_eq!(card.strong_colors, [red(), ManaColor::Black]);
}

// Spec #32 C3 : une unité, sans séparation.
#[test]
fn offer_unit_has_no_strong_block() {
    let catalog = french();
    let card = offer_card(&with_offers(), 2, "Paysans", &catalog);
    assert_eq!(card.strong_heading, "");
    assert!(card.strong_colors.is_empty());
    assert!(!card.lines.is_empty());
}

// Spec #32 C4 : l'action avancée d'un monastère garde sa marque et se lit comme en main.
#[test]
fn monastery_advanced_action_reads_like_hand_card() {
    let catalog = french();
    let state = Monastery::default().state();
    let card = offer_card(&state, 2, "Bouclier de glace", &catalog);
    assert_eq!(card.mark.as_deref(), Some("monastère"));
    assert_eq!(card.color, Some(BasicColor::Blue));
    assert_eq!(card.strong_heading, "Renforcé");
    assert_eq!(card.strong_colors, [blue()]);
    assert!(
        !card.lines.iter().any(|line| line.starts_with("Renforcé")),
        "{:?}",
        card.lines
    );
}

// --- Fenêtres de choix ---

/// Sur la tour de mage conquise ce tour, l'offre Éclair de feu, Tempête de neige, Exposition ;
/// le tour fini, le choix du sort attend (spec 043, C43).
fn choosing_spell() -> GameState {
    let spell = |spell| CardId::new(DeedCard::Spell(spell), 1);
    let mut events = Fortified {
        figure: MAGE_TOWER,
        tower: None,
        spells: vec![fireball(), spell(Spell::Snowstorm), spell(Spell::Expose)],
        ..Fortified::default()
    }
    .events();
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: MAGE_TOWER,
    });
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, basic(card(March, 1)));
    done(
        &state,
        arcanum_core::Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    )
}

// Spec #32 C5 : la carte du choix du sort est celle de l'offre, à « Prendre » près.
#[test]
fn spell_choice_card_matches_offer_card() {
    let catalog = french();
    let state = choosing_spell();
    let mut chosen = screen_of(&state, &catalog)
        .spell_choice
        .expect("choix du sort")
        .cards
        .into_iter()
        .find(|card| card.name == "Éclair de feu")
        .expect("Éclair de feu");
    let mut offered = offer_card(&with_offers(), 1, "Éclair de feu", &catalog);
    assert_eq!(chosen.color, Some(BasicColor::Red));
    chosen.take = None;
    offered.take = None;
    assert_eq!(chosen, offered);
}

// Spec #32 C6 : un artefact, son texte écrit et « si détruit », sans pastille.
#[test]
fn artifact_choice_card_reads_like_hand_artifact() {
    let catalog = french();
    let ring = artifact(arcanum_core::Artifact::RubyRing);
    let amulet = artifact(arcanum_core::Artifact::AmuletOfSun);
    let mut events = Monastery {
        artifacts: vec![ring, amulet],
        ..Monastery::default()
    }
    .events();
    events.push(Event::ArtifactsDrawn {
        player: SOLO,
        cards: vec![ring, amulet],
    });
    let state = events.iter().fold(GameState::default(), apply);
    let ring = screen_of(&state, &catalog)
        .artifact_choice
        .expect("choix de l'artefact")
        .cards
        .into_iter()
        .find(|card| card.name == "Anneau de rubis")
        .expect("Anneau de rubis");
    assert_eq!(ring.color, None);
    assert_eq!(
        ring.lines,
        [
            "Jeton et cristal rouges ; Gloire +1",
            "Mana rouge et noir illimité ce tour ; Gloire +1 par sort rouge lancé",
        ]
    );
    assert_eq!(ring.strong_heading, "si détruit");
    assert!(ring.strong_colors.is_empty());
}

// --- Même texte qu'en main ---

// Spec #32 C7 : les lignes de l'offre valent le texte du « i » de la carte en main.
#[test]
fn offer_text_matches_hand_info() {
    let catalog = french();
    let cards = [
        (
            pure_magic(),
            offer_card(&with_offers(), 0, "Magie pure", &catalog),
        ),
        (
            fireball(),
            offer_card(&with_offers(), 1, "Éclair de feu", &catalog),
        ),
        (
            ice_shield(),
            offer_card(
                &Monastery::default().state(),
                2,
                "Bouclier de glace",
                &catalog,
            ),
        ),
    ];
    for (card, offered) in cards {
        let held = held_alone(card, &catalog);
        let info = held.info.expect("texte du « i »");
        assert_eq!(offered.lines, [info.basic, info.strong], "{card:?}");
        assert_eq!(offered.strong_heading, held.strong_heading, "{card:?}");
    }
}

// --- Rangées ---

// Spec #32 C8 : autant de cartes que la largeur en tient, une au moins.
#[test]
fn cards_per_row_fills_the_width() {
    let counts: Vec<usize> = [900.0, 888.0, 887.0, 700.0, 216.0, 100.0]
        .into_iter()
        .map(cards_per_row)
        .collect();
    assert_eq!(counts, [4, 4, 3, 3, 1, 1]);
}

// Spec #68 C4 : les fenêtres de cartes montrent les textes complets, comme le « i » (Agilité dans
// l'offre d'actions avancées).
#[test]
fn offer_window_shows_full_texts() {
    let catalog = french();
    let agility = advanced(AdvancedAction::Agility);
    let mut events = Sandbox {
        figure: Some(arcanum_core::Hex::new(0, 0)),
        hand: vec![card(March, 1)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::CommonDeckShuffled {
        deck: CommonDeck::AdvancedActions,
        order: vec![CommonCard::Deed(agility)],
    });
    events.push(Event::OfferCardDealt {
        offer: Offer::AdvancedActions,
        card: CommonCard::Deed(agility),
    });
    let state = events.iter().fold(GameState::default(), apply);
    let shown = offer_card(&state, 0, "Agilité", &catalog);
    assert_eq!(
        shown.lines,
        [
            catalog.text("card.agility.basic_text"),
            catalog.text("card.agility.strong_text"),
        ]
    );
}
