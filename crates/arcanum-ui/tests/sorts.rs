//! Scénarios de la spec 054 côté affichage (sorts) : paiement d'un sort, bouton Renforcer grisé de
//! jour, ennemi visé, options, ennemis sans fortification ni résistances ou détruits, attaque
//! différée, sort lancé depuis l'offre, effets en cours et faits du tour.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, CardId, CardPlay, DeedCard, EnemyToken, Event, GameState,
    ManaColor, ManaPayment, Spell, TimeOfDay, UndoStatus, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CastChoice, CastPicks, CombatChoice, CombatScreen, Command, EffectButton, EffectFace,
    GameScreen, GameSession, ScreenMode, ScreenPhase, combat_screen, game_screen, offers_window_in,
    repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{play, session_of};

use Spell::*;
use arcanum_core::BasicAction::Rage;

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn spell(card: Spell) -> CardId {
    CardId::new(DeedCard::Spell(card), 1)
}

fn crystal(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

fn black_token() -> Event {
    Event::ManaTokenGained {
        player: SOLO,
        color: ManaColor::Black,
    }
}

/// Un combat aux orques ② contre `enemies`, la main `hand`, des dés de la Source qui ne paient rien
/// (noirs de jour, or de nuit), puis `extra` (cristaux, jetons, offre de sorts).
fn fight(time: TimeOfDay, hand: &[CardId], enemies: &[EnemyToken], extra: Vec<Event>) -> GameState {
    let events = sandbox_events(time, hand, enemies, extra);
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

/// Le journal du bac à sable de `fight`, avant le défi.
fn sandbox_events(
    time: TimeOfDay,
    hand: &[CardId],
    enemies: &[EnemyToken],
    extra: Vec<Event>,
) -> Vec<Event> {
    let dice = match time {
        TimeOfDay::Day => vec![ManaColor::Black; 3],
        TimeOfDay::Night => vec![ManaColor::Gold; 3],
    };
    let mut events = Sandbox {
        time,
        figure: Some(PLAIN_W),
        hand: hand.to_vec(),
        placed: enemies.iter().map(|&enemy| (enemy, ORCS_2)).collect(),
        dice,
        ..Sandbox::default()
    }
    .events();
    events.extend(extra);
    events
}

/// Le même combat, une attaque déclarée sur `target` (les cartes d'attaque se jouent dans une
/// attaque en cours).
fn attacking(time: TimeOfDay, hand: &[CardId], target: EnemyToken, extra: Vec<Event>) -> GameState {
    done(&fight(time, hand, &[target], extra), declare(&[target]))
}

fn screen_in(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        mode,
        catalog,
    )
}

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    screen_in(state, &ScreenMode::Normal, catalog)
}

fn combat_of(state: &GameState, catalog: &Catalog) -> CombatScreen {
    let legal = legal_actions(state, SOLO);
    combat_screen(
        &view(state, SOLO),
        &legal,
        &CombatChoice::default(),
        UndoStatus::Possible,
        catalog,
    )
    .expect("combat engagé")
}

/// Les boutons d'effet de base et renforcés de `card` dans la main.
fn effects_of(screen: &GameScreen, card: CardId) -> (Vec<EffectButton>, Vec<EffectButton>) {
    let shown = screen
        .hand
        .iter()
        .find(|shown| shown.card == card)
        .expect("carte en main");
    (shown.basic_effects.clone(), shown.strong_effects.clone())
}

/// Les libellés du bandeau du choix d'un sort.
fn cast_labels(state: &GameState, choice: CastChoice, catalog: &Catalog) -> Vec<String> {
    screen_in(state, &ScreenMode::Casting(choice), catalog)
        .mana_prompt
        .expect("bandeau du choix du sort")
        .payments
        .into_iter()
        .map(|button| button.label)
        .collect()
}

fn enemy_name(token: EnemyToken, catalog: &Catalog) -> String {
    catalog.text(token.enemy().label_key())
}

fn status_of(combat: &CombatScreen, token: EnemyToken) -> String {
    combat
        .enemies
        .iter()
        .find(|enemy| enemy.token == token)
        .and_then(|enemy| enemy.status.clone())
        .unwrap_or_default()
}

fn cast_at(card: CardId, option: u8, black: bool, enemy: Option<EnemyToken>) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana: ManaPayment::Crystal,
            black: black.then_some(ManaPayment::Token),
            extra_mana: None,
            enemy,
            from_offer: false,
            targets: Default::default(),
        },
    }
}

fn swordsmen() -> EnemyToken {
    token(arcanum_core::Enemy::Swordsmen, 1)
}

fn golems() -> EnemyToken {
    token(arcanum_core::Enemy::Golems, 1)
}

fn fire_mages() -> EnemyToken {
    token(arcanum_core::Enemy::FireMages, 1)
}

// --- Paiement, ennemi visé, options (étape 11) ---

// Spec 054 C1
#[test]
fn spell_basic_effect_cast_with_its_color() {
    // MKUE p. 4, 3.b : l'effet de base d'un sort se paie du mana de sa couleur.
    let catalog = french();
    let fireball = spell(Fireball);
    let state = attacking(
        TimeOfDay::Day,
        &[fireball],
        prowlers(),
        vec![crystal(BasicColor::Red)],
    );
    let (basic, _) = effects_of(&screen_of(&state, &catalog), fireball);
    let legal = legal_actions(&state, SOLO);
    // Un seul paiement : le bouton lance le sort.
    let Some(Command::Play(index)) = basic[0].command else {
        panic!("{:?}", basic[0].command);
    };
    assert_eq!(legal[index], cast_at(fireball, 0, false, None));
    // Sans mana rouge, grisé avec le mana manquant.
    let state = attacking(TimeOfDay::Day, &[fireball], prowlers(), Vec::new());
    let (basic, _) = effects_of(&screen_of(&state, &catalog), fireball);
    assert_eq!(basic[0].command, None);
    assert!(basic[0].tooltip.contains("rouge"), "{:?}", basic[0].tooltip);
}

// Spec 054 C2
#[test]
fn strong_spell_greyed_by_day() {
    // MKUE p. 5, 2.a : pas de mana noir de jour.
    let catalog = french();
    let fireball = spell(Fireball);
    let state = attacking(
        TimeOfDay::Day,
        &[fireball],
        prowlers(),
        vec![crystal(BasicColor::Red), black_token()],
    );
    let (_, strong) = effects_of(&screen_of(&state, &catalog), fireball);
    assert_eq!(strong[0].command, None);
    assert!(
        strong[0].tooltip.contains("Renforcé : mana noir, de nuit"),
        "{:?}",
        strong[0].tooltip
    );
}

// Spec 054 C3
#[test]
fn strong_spell_at_night_pays_color_then_black() {
    // MKUE p. 4, 3.b ; p. 5, 2.b : la couleur du sort et un mana noir.
    let catalog = french();
    let fireball = spell(Fireball);
    let state = attacking(
        TimeOfDay::Night,
        &[fireball],
        prowlers(),
        vec![
            crystal(BasicColor::Red),
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Basic(BasicColor::Red),
            },
            black_token(),
        ],
    );
    let (_, strong) = effects_of(&screen_of(&state, &catalog), fireball);
    let choice = CastChoice {
        card: fireball,
        option: Some((true, 0)),
        from_offer: false,
        picks: CastPicks::default(),
    };
    // Cristal ou jeton rouge (le dé or ne paie rien de nuit) : le bandeau du choix.
    assert_eq!(strong[0].command, Some(Command::ChooseCast(choice)));
    // Spec #11 C9 : le noir est dans le titre du bloc, plus sur le bouton.
    assert_eq!(strong[0].color, None);
    let labels = cast_labels(&state, choice, &catalog);
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Cristal <red>rouge</red>")
                && label.contains("Jeton <black>noir</black>")),
        "{labels:?}"
    );
}

// Spec 054 C10
#[test]
fn tremor_chooses_its_enemy_and_names_its_options() {
    // Carte ; Compendium, *Tremor* S1 : un ennemi, ou tous.
    let catalog = french();
    let tremor = spell(Tremor);
    let state = fight(
        TimeOfDay::Day,
        &[tremor],
        &[swordsmen(), prowlers()],
        vec![crystal(BasicColor::Green)],
    );
    let (basic, _) = effects_of(&screen_of(&state, &catalog), tremor);
    assert_eq!(basic.len(), 2);
    let target = CastChoice {
        card: tremor,
        option: Some((false, 0)),
        from_offer: false,
        picks: CastPicks::default(),
    };
    assert_eq!(basic[0].command, Some(Command::ChooseCast(target)));
    // Issue #107 : chaque option dit qui elle vise, un ennemi ou tous.
    assert_eq!(basic[0].tooltip, "Jouer : Un ennemi : Armure −3 (min. 1)");
    assert_eq!(
        basic[1].tooltip,
        "Jouer : Tous les ennemis : Armure −2 (min. 1)"
    );
    // L'option « tous » ne vise personne : un seul paiement, le sort se lance.
    assert!(matches!(basic[1].command, Some(Command::Play(_))));
    let labels = cast_labels(&state, target, &catalog);
    assert_eq!(labels.len(), 2, "{labels:?}");
    assert!(labels[0].contains(&enemy_name(swordsmen(), &catalog)));
    assert!(labels[1].contains(&enemy_name(prowlers(), &catalog)));
    assert!(
        labels[0].contains("Cristal <green>vert</green>"),
        "{labels:?}"
    );
}

// Issue #107 : un libellé écrit par option, de base et renforcée, en fr et en en.
#[test]
fn tremor_options_have_their_own_written_labels() {
    // Carte ; Compendium, *Tremor* S1 : un ennemi, ou tous.
    let tremor = spell(Tremor);
    for language in ["fr", "en"] {
        let catalog = Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(language.into()),
        )
        .expect("catalogue du dépôt");
        let state = fight(
            TimeOfDay::Night,
            &[tremor],
            &[swordsmen(), prowlers()],
            vec![crystal(BasicColor::Green), black_token()],
        );
        let (basic, strong) = effects_of(&screen_of(&state, &catalog), tremor);
        let faces: Vec<EffectFace> = basic
            .iter()
            .chain(&strong)
            .map(|b| b.face.clone())
            .collect();
        let expected: Vec<EffectFace> = [
            "card.tremor.basic_label_1",
            "card.tremor.basic_label_2",
            "card.tremor.strong_label_1",
            "card.tremor.strong_label_2",
        ]
        .into_iter()
        .map(|key| EffectFace::Text(catalog.text(key)))
        .collect();
        assert_eq!(faces, expected, "{language}");
    }
}

// Spec 054 C14, issue privée 245
#[test]
fn flame_wave_buttons_show_total_with_enemies_faced() {
    // Carte ; Compendium, *Flame Wave* S2, Q4 : +2 par jeton ennemi affronté, ici deux.
    let catalog = french();
    let flame_wall = spell(FlameWall);
    let state = fight(
        TimeOfDay::Night,
        &[flame_wall],
        &[swordsmen(), prowlers()],
        vec![crystal(BasicColor::Red), black_token()],
    );
    let (basic, strong) = effects_of(&screen_of(&state, &catalog), flame_wall);
    let tooltips = |buttons: &[EffectButton]| -> Vec<String> {
        buttons
            .iter()
            .map(|button| button.tooltip.clone())
            .collect()
    };
    let (basic, strong) = (tooltips(&basic), tooltips(&strong));
    // La base ne compte pas les ennemis.
    assert!(basic[0].contains("Attaque de feu 5"), "{basic:?}");
    assert!(basic[1].contains("Parade de feu 7"), "{basic:?}");
    // Renforcé : 5 + 2 × 2 et 7 + 2 × 2.
    assert!(strong[0].contains("Attaque de feu 9"), "{strong:?}");
    assert!(strong[1].contains("Parade de feu 11"), "{strong:?}");
}

// Spec 054 C15
#[test]
fn mana_bolt_options_greyed_by_phase() {
    // Carte ; Compendium, *Mana Bolt* S1 : en phase à distance, le blanc et le vert seulement.
    let catalog = french();
    let bolt = spell(ManaBolt);
    let colors = [
        BasicColor::Blue,
        BasicColor::Blue,
        BasicColor::Red,
        BasicColor::White,
        BasicColor::Green,
    ];
    let state = attacking(
        TimeOfDay::Day,
        &[bolt],
        prowlers(),
        colors.iter().map(|&color| crystal(color)).collect(),
    );
    let (basic, _) = effects_of(&screen_of(&state, &catalog), bolt);
    assert_eq!(basic.len(), 4);
    let enabled: Vec<bool> = basic
        .iter()
        .map(|button| button.command.is_some())
        .collect();
    assert_eq!(enabled, vec![false, false, true, true]);
    // La pastille : le mana en plus de chaque option.
    assert_eq!(basic[1].color, Some(ManaColor::Basic(BasicColor::Red)));
}

// --- Ennemis du combat, attaque différée, offre, effets en cours (étape 12) ---

// Spec 054 C31, C7, C8
#[test]
fn enemy_card_shows_lost_fortification_and_resistances() {
    // Carte Exposition ; MKUE p. 9, 3.a et 4.a.
    let catalog = french();
    let expose = spell(Expose);
    let state = fight(
        TimeOfDay::Day,
        &[expose],
        &[diggers(), golems()],
        vec![crystal(BasicColor::White), crystal(BasicColor::White)],
    );
    let state = done(&state, cast_at(expose, 0, false, Some(diggers())));
    let combat = combat_of(&state, &catalog);
    let status = status_of(&combat, diggers());
    assert!(status.contains("[[fortified]] perdu"), "{status:?}");
    assert!(!status.contains("siège seulement"), "{status:?}");
    // Les Golems : plus de résistances, ni leurs icônes.
    let state = fight(
        TimeOfDay::Day,
        &[expose],
        &[golems()],
        vec![crystal(BasicColor::White)],
    );
    let state = done(&state, cast_at(expose, 0, false, Some(golems())));
    let combat = combat_of(&state, &catalog);
    assert!(
        status_of(&combat, golems()).contains("plus de résistances"),
        "{:?}",
        status_of(&combat, golems())
    );
    assert!(combat.enemies[0].resistances.is_empty());
}

// Spec 054 C22
#[test]
fn chilled_enemy_loses_fire_resistance_on_its_card() {
    // Carte Frisson ; Compendium, *Chill* S1.
    let catalog = french();
    let chill = spell(Chill);
    let state = fight(
        TimeOfDay::Day,
        &[chill],
        &[fire_mages()],
        vec![crystal(BasicColor::Blue)],
    );
    let state = done(&state, cast_at(chill, 0, false, Some(fire_mages())));
    let combat = combat_of(&state, &catalog);
    let status = status_of(&combat, fire_mages());
    assert!(status.contains("plus de résistance au Feu"), "{status:?}");
    assert!(status.contains("n'attaque pas"), "{status:?}");
}

// Spec 054 C18, C20
#[test]
fn destroyed_enemy_shown_and_in_turn_facts() {
    // Carte Tornade : détruire en phase d'attaque.
    let catalog = french();
    let whirlwind = spell(Whirlwind);
    let state = fight(
        TimeOfDay::Night,
        &[whirlwind],
        &[fire_mages(), prowlers()],
        vec![crystal(BasicColor::White), black_token()],
    );
    let state = skip_to_attack_phase(&state);
    let state = done(&state, cast_at(whirlwind, 0, true, Some(fire_mages())));
    let combat = combat_of(&state, &catalog);
    let status = status_of(&combat, fire_mages());
    assert!(status.contains("détruit"), "{status:?}");
    assert!(!status.contains("vaincu"), "{status:?}");
    let facts = screen_of(&state, &catalog).turn_facts;
    let expected = format!("{} détruit", enemy_name(fire_mages(), &catalog));
    assert!(facts.contains(&expected), "{facts:?}");
}

// Spec 054 C19, C31
#[test]
fn deferred_attack_button_and_modifier_line() {
    // Carte Bouclier ardent ; Compendium, *Burning Shield* S2, S3.
    let catalog = french();
    let shield = spell(BurningShield);
    let state = fight(
        TimeOfDay::Day,
        &[shield],
        &[prowlers(), wolf_riders()],
        vec![crystal(BasicColor::Red)],
    );
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, cast_at(shield, 0, false, None));
    let state = done(&state, Action::ResolveBlock);
    let lines = screen_of(&state, &catalog)
        .modifiers
        .map(|list| list.lines)
        .unwrap_or_default();
    assert!(
        lines
            .iter()
            .any(|line| line.ends_with(": Attaque de feu 4 en phase d'attaque")),
        "{lines:?}"
    );
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[wolf_riders()]));
    let button = combat_of(&state, &catalog)
        .in_progress
        .and_then(|panel| panel.deferred_attack)
        .expect("bouton de l'attaque différée");
    assert_eq!(button.label, "Ajouter l'Attaque de feu 4");
    let legal = legal_actions(&state, SOLO);
    assert_eq!(
        button.command,
        Command::Play(
            legal
                .iter()
                .position(|action| *action == Action::UseDeferredAttack)
                .expect("action légale")
        )
    );
}

// Spec 054 C27, C31
#[test]
fn magic_talent_offers_cast_button_on_spells_of_its_color() {
    // Carte Don pour la magie, version Lost Legion ; Compendium, *Magic Talent* S1.
    let catalog = french();
    let talent = advanced(AdvancedAction::MagicTalent);
    let rage = card(Rage, 1);
    let mut extra = spell_offer(&[spell(Fireball), spell(Snowstorm), spell(Tremor)]);
    extra.push(crystal(BasicColor::Red));
    let state = fight(TimeOfDay::Day, &[talent, rage], &[prowlers()], extra);
    let legal = legal_actions(&state, SOLO);
    let talent_play = legal
        .iter()
        .find(|action| {
            matches!(action, Action::PlayCard { card, play: CardPlay::Basic { discard: Some(d), .. } }
                if *card == talent && *d == rage)
        })
        .cloned()
        .expect("Don pour la magie en défaussant Rage");
    let state = done(&state, talent_play);
    let lines = screen_of(&state, &catalog)
        .modifiers
        .map(|list| list.lines)
        .unwrap_or_default();
    assert!(
        lines
            .iter()
            .any(|line| line.ends_with(": un sort <red>rouge</red> de l'offre")),
        "{lines:?}"
    );
    let state = done(&state, declare(&[prowlers()]));
    let legal = legal_actions(&state, SOLO);
    let window = offers_window_in(&view(&state, SOLO), &legal, &ScreenMode::Normal, &catalog);
    let spells = &window.columns[1].cards;
    let take: Vec<Option<&str>> = spells
        .iter()
        .map(|card| card.take.as_ref().map(|button| button.label.as_str()))
        .collect();
    assert_eq!(take, vec![Some("Lancer"), None, None]);
    // Un seul paiement (le cristal rouge) : le bouton lance le sort depuis l'offre.
    let Some(Command::Play(index)) = spells[0].take.as_ref().map(|button| button.command) else {
        panic!("{:?}", spells[0].take);
    };
    assert!(matches!(
        legal[index],
        Action::PlayCard {
            play: CardPlay::Cast {
                from_offer: true,
                ..
            },
            ..
        }
    ));
}

// Issue privée 247 (spec 054 C27) : avec un seul paiement, « Lancer » joue le sort et referme la fenêtre
// Offres, comme le bandeau du choix quand il y a plusieurs paiements.
#[test]
fn single_payment_cast_from_offer_closes_offers_window() {
    let catalog = french();
    let talent = advanced(AdvancedAction::MagicTalent);
    let rage = card(Rage, 1);
    let mut extra = spell_offer(&[spell(Fireball), spell(Snowstorm), spell(Tremor)]);
    extra.push(crystal(BasicColor::Red));
    let mut session: GameSession = session_of(sandbox_events(
        TimeOfDay::Day,
        &[talent, rage],
        &[prowlers()],
        extra,
    ));
    play(&mut session, &challenge(ORCS_2));
    play(&mut session, &Action::StartCombat);
    let talent_play = session
        .legal_actions()
        .into_iter()
        .find(|action| {
            matches!(action, Action::PlayCard { card, play: CardPlay::Basic { discard: Some(d), .. } }
                if *card == talent && *d == rage)
        })
        .expect("Don pour la magie en défaussant Rage");
    play(&mut session, &talent_play);
    play(&mut session, &declare(&[prowlers()]));
    session.command(Command::OpenOffers);
    let window = session
        .offers_window(&catalog)
        .expect("fenêtre Offres ouverte");
    let cast = window.columns[1].cards[0]
        .take
        .as_ref()
        .map(|button| button.command)
        .expect("« Lancer » sur Éclair de feu");
    assert!(matches!(cast, Command::Play(_)), "{cast:?}");
    session.command(cast);
    session.run_frame();
    assert!(session.offers_window(&catalog).is_none());
}

/// Combat contre les Prowlers, Don pour la magie joué en défaussant Rage, l'offre Éclair de feu,
/// Blizzard, Secousse, puis l'attaque déclarée sur les Prowlers.
fn talent_played() -> GameState {
    let talent = advanced(AdvancedAction::MagicTalent);
    let rage = card(Rage, 1);
    let mut extra = spell_offer(&[spell(Fireball), spell(Snowstorm), spell(Tremor)]);
    extra.push(crystal(BasicColor::Red));
    let state = fight(TimeOfDay::Day, &[talent, rage], &[prowlers()], extra);
    let state = done(
        &state,
        Action::PlayCard {
            card: talent,
            play: CardPlay::Basic {
                option: 0,
                discard: Some(rage),
                mana: None,
                with: None,
                gain: None,
                die: None,
            },
        },
    );
    done(&state, declare(&[prowlers()]))
}

fn offer_cast(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option: 0,
            mana: ManaPayment::Crystal,
            black: None,
            extra_mana: None,
            enemy: None,
            from_offer: true,
            targets: Default::default(),
        },
    }
}

// Spec 054 C33, issue privée 246
#[test]
fn offer_spell_shown_in_play_area() {
    // Compendium, *Magic Talent* S1 : le sort reste dans l'offre, la zone de jeu le montre.
    let catalog = french();
    let state = done(&talent_played(), offer_cast(spell(Fireball)));
    let screen = screen_of(&state, &catalog);
    let lines: Vec<&str> = screen
        .play_area
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    assert_eq!(
        lines.last(),
        Some(&"Éclair de feu (depuis l'offre) : Attaque à distance de feu 5"),
        "{lines:?}"
    );
    // Dit une seule fois : pas dans la liste des faits du tour.
    assert!(
        !screen
            .turn_facts
            .iter()
            .any(|fact| fact.contains("Éclair de feu")),
        "{:?}",
        screen.turn_facts
    );
    // La victoire ne l'efface pas.
    let state = done(&state, Action::ResolveAttack);
    let lines: Vec<String> = screen_of(&state, &catalog)
        .play_area
        .into_iter()
        .map(|line| line.text)
        .collect();
    assert!(
        lines.iter().any(|line| line.contains("(depuis l'offre)")),
        "{lines:?}"
    );
}

// Spec 054 C33, issue privée 246
#[test]
fn no_cast_button_once_enemies_defeated() {
    // Plus d'ennemi à attaquer : aucun sort d'attaque de l'offre n'est lançable.
    let catalog = french();
    let state = apply(
        talent_played(),
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![prowlers()],
            fame: 0,
        },
    );
    let legal = legal_actions(&state, SOLO);
    let window = offers_window_in(&view(&state, SOLO), &legal, &ScreenMode::Normal, &catalog);
    assert!(
        window.columns[1]
            .cards
            .iter()
            .all(|card| card.take.is_none()),
        "{:?}",
        window.columns[1].cards
    );
}

// Spec 054 C14, issue privée 246
#[test]
fn flame_wave_line_shows_total() {
    // Carte Déluge de flammes : la ligne de la zone de jeu dit l'effet obtenu, 5 + 2 × 2.
    let catalog = french();
    let flame_wall = spell(FlameWall);
    let state = fight(
        TimeOfDay::Night,
        &[flame_wall],
        &[swordsmen(), prowlers()],
        vec![crystal(BasicColor::Red), black_token()],
    );
    let state = done(
        &skip_to_attack_phase(&state),
        declare(&[swordsmen(), prowlers()]),
    );
    let state = done(&state, cast_at(flame_wall, 0, true, None));
    let lines: Vec<String> = screen_of(&state, &catalog)
        .play_area
        .into_iter()
        .map(|line| line.text)
        .collect();
    assert_eq!(lines, vec!["Déluge de flammes : Attaque de feu 9"]);
}

// Spec 054 C24
#[test]
fn demolish_shows_ignored_site_fortifications() {
    // Carte Démolition ; Compendium, *Demolish* S1.
    let catalog = french();
    let demolish = spell(Demolish);
    let state = fight(
        TimeOfDay::Day,
        &[demolish],
        &[prowlers()],
        vec![crystal(BasicColor::Red)],
    );
    let state = done(&state, cast_at(demolish, 0, false, None));
    let lines = screen_of(&state, &catalog)
        .modifiers
        .map(|list| list.lines)
        .unwrap_or_default();
    assert!(
        lines
            .iter()
            .any(|line| line.ends_with(": fortifications des sites ignorées")),
        "{lines:?}"
    );
    // Le texte de la carte corrigé (Compendium, *Demolish* S1).
    assert_eq!(
        catalog.text("card.demolish.basic_text"),
        "Ce tour-ci, ignorez les fortifications des sites. Les ennemis perdent 1 d'[[armor]]Armure, \
         sans descendre sous 1."
    );
}
