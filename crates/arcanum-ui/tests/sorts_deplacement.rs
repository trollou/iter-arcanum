//! Scénarios de la spec 057 côté affichage : les choix d'un sort dans le bandeau (arrivée, dé),
//! la fenêtre de Méditation, l'offre d'unités avec Appel aux armes et Appel à la gloire, le tour
//! supplémentaire, le dé gardé et les cartes écartées.

use arcanum_core::{
    Action, BasicColor, CardId, CardPlay, CastTargets, DeedCard, Event, GameState, ManaColor,
    ManaPayment, Spell, TimeOfDay, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardInput, CastChoice, CastDestination, CastPicks, Command, DestinationChoice,
    EndTurnChoice, GameScreen, ScreenMode, ScreenPhase, board_screen, game_screen, hex_center,
    offers_window_in, repository_assets_dir,
};
use eframe::egui::{Rect, pos2, vec2};

#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{bar_of, board_input, session_of};

use Spell::*;
use arcanum_core::BasicAction::{Concentration, March, Promise, Stamina, Swiftness};

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

/// Le bac à sable, figurine sur la plaine à l'ouest du village, la main `hand`, les dés `dice`,
/// puis `extra`.
fn state_of(
    time: TimeOfDay,
    hand: &[CardId],
    dice: Vec<ManaColor>,
    extra: Vec<Event>,
) -> GameState {
    decor_events(time, hand, dice, extra)
        .iter()
        .fold(GameState::default(), apply)
}

fn decor_events(
    time: TimeOfDay,
    hand: &[CardId],
    dice: Vec<ManaColor>,
    extra: Vec<Event>,
) -> Vec<Event> {
    let mut events = Sandbox {
        time,
        figure: Some(PLAIN_W),
        hand: hand.to_vec(),
        dice,
        ..Sandbox::default()
    }
    .events();
    events.extend(extra);
    events
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

fn cast_labels(state: &GameState, choice: CastChoice, catalog: &Catalog) -> Vec<String> {
    screen_in(state, &ScreenMode::Casting(choice), catalog)
        .mana_prompt
        .expect("bandeau du choix du sort")
        .payments
        .into_iter()
        .map(|button| button.label)
        .collect()
}

fn cast(card: CardId, option: u8, black: bool, targets: CastTargets) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana: ManaPayment::Crystal,
            black: black.then_some(ManaPayment::Token),
            extra_mana: None,
            enemy: None,
            from_offer: false,
            targets,
        },
    }
}

fn index_of(state: &GameState, action: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|legal| legal == action)
        .unwrap_or_else(|| panic!("{action:?} absente des actions légales"))
}

// Spec 057 C23 (écran) : le dé de Puits de mana se choisit dans le bandeau du sort, un bouton par
// dé.
#[test]
fn cast_prompt_names_the_die() {
    let catalog = french();
    let claim = spell(ManaClaim);
    let state = state_of(
        TimeOfDay::Day,
        &[claim],
        vec![
            ManaColor::Basic(BasicColor::Blue),
            ManaColor::Basic(BasicColor::Red),
        ],
        vec![crystal(BasicColor::Blue)],
    );
    let labels = cast_labels(
        &state,
        CastChoice {
            card: claim,
            option: Some((false, 0)),
            from_offer: false,
            picks: CastPicks::default(),
        },
        &catalog,
    );
    let red = catalog.format("mana_claim.die", &[("color", "<red>rouge</red>")]);
    assert!(
        labels.iter().any(|label| label.contains(&red)),
        "{labels:?}"
    );
}

fn underground_choice() -> CastChoice {
    CastChoice {
        card: spell(UndergroundTravel),
        option: Some((false, 0)),
        from_offer: false,
        picks: CastPicks::default(),
    }
}

/// Voyage souterrain en main, un cristal vert, et le dé vert de la Source du bac à sable : deux
/// paiements.
fn underground_two_payments() -> GameState {
    state_of(
        TimeOfDay::Day,
        &[spell(UndergroundTravel)],
        Sandbox::default().dice,
        vec![crystal(BasicColor::Green)],
    )
}

/// Le lancer de base de Voyage souterrain payé du cristal vert, sans son arrivée.
fn crystal_payment() -> CardPlay {
    let Action::PlayCard { play, .. } =
        cast(spell(UndergroundTravel), 0, false, CastTargets::default())
    else {
        unreachable!("cast() fait un PlayCard")
    };
    play
}

fn arrival(destination: arcanum_core::Hex) -> Action {
    cast(
        spell(UndergroundTravel),
        0,
        false,
        CastTargets {
            destination: Some(destination),
            ..CastTargets::default()
        },
    )
}

// Spec 057 C3 (écran), issue privée 257 : avec deux paiements, le bandeau ne propose que les paiements,
// sans coordonnée ; rien n'est encore en évidence sur le plateau.
#[test]
fn destination_prompt_offers_payments_first() {
    let catalog = french();
    let state = underground_two_payments();
    let screen = screen_in(&state, &ScreenMode::Casting(underground_choice()), &catalog);
    let payments = screen.mana_prompt.expect("bandeau du sort").payments;
    assert_eq!(payments.len(), 2, "{payments:?}");
    for button in &payments {
        assert!(!button.label.contains("Village"), "{}", button.label);
        assert!(
            matches!(button.command, Command::ChooseDestination(_)),
            "{button:?}"
        );
    }
    assert!(screen.cast_destinations.is_empty());
}

// Spec 057 C3 (écran), issue privée 257 : le paiement choisi, les arrivées légales sont sur le plateau,
// chacune joue le sort ainsi payé ; le bandeau dit de cliquer l'arrivée, sans bouton de paiement.
#[test]
fn chosen_payment_puts_destinations_on_the_board() {
    let catalog = french();
    let state = underground_two_payments();
    let mode = ScreenMode::ChoosingDestination(DestinationChoice {
        choice: underground_choice(),
        payment: crystal_payment(),
    });
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.mana_prompt.expect("bandeau du sort");
    assert!(prompt.payments.is_empty(), "{:?}", prompt.payments);
    assert_eq!(
        prompt.text,
        catalog.format(
            "spell.destination_prompt",
            &[("card", &catalog.text("card.underground_travel.name"))]
        )
    );
    assert!(
        screen.cast_destinations.contains(&CastDestination {
            hex: VILLAGE,
            command: Command::Play(index_of(&state, &arrival(VILLAGE))),
        }),
        "{:?}",
        screen.cast_destinations
    );
    let legal = legal_actions(&state, SOLO);
    for destination in &screen.cast_destinations {
        let Command::Play(index) = destination.command else {
            panic!("{destination:?}");
        };
        assert_eq!(legal[index], arrival(destination.hex));
    }
}

// Spec 057 C3 (écran), issue privée 257 : un seul paiement possible, les arrivées sont sur le plateau dès
// le clic sur le sort.
#[test]
fn single_payment_goes_straight_to_the_board() {
    let catalog = french();
    let state = state_of(
        TimeOfDay::Day,
        &[spell(UndergroundTravel)],
        vec![
            ManaColor::Basic(BasicColor::Blue),
            ManaColor::Basic(BasicColor::Red),
        ],
        vec![crystal(BasicColor::Green)],
    );
    let screen = screen_in(&state, &ScreenMode::Casting(underground_choice()), &catalog);
    assert!(screen.mana_prompt.expect("bandeau").payments.is_empty());
    assert!(
        screen
            .cast_destinations
            .iter()
            .any(|destination| destination.hex == VILLAGE),
        "{:?}",
        screen.cast_destinations
    );
}

fn board_area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn board_camera() -> BoardCamera {
    BoardCamera {
        center: hex_center(VILLAGE),
        zoom: 30.0,
    }
}

// Spec 057 C3 (écran), issue privée 257 : l'arrivée est mise en évidence sur le plateau, et un clic sur
// sa case joue le sort, avant tout déplacement.
#[test]
fn board_click_on_a_destination_plays_the_spell() {
    let catalog = french();
    let state = underground_two_payments();
    let seen = view(&state, SOLO);
    let legal = legal_actions(&state, SOLO);
    let destinations = [CastDestination {
        hex: VILLAGE,
        command: Command::Play(7),
    }];
    let cursor = board_camera().to_screen(board_area(), hex_center(VILLAGE));
    let screen = board_screen(
        &BoardInput {
            destinations: &destinations,
            ..board_input(&seen, &legal, board_camera(), board_area(), Some(cursor))
        },
        &catalog,
    );
    assert_eq!(screen.click, Some(Command::Play(7)));
    assert!(
        screen
            .destination_marks
            .iter()
            .any(|mark| mark.hex == VILLAGE)
    );
    assert!(screen.move_marks.iter().all(|mark| !mark.reachable));
}

// Spec 057 C3 (écran), issue privée 257 : de bout en bout dans la session, le sort, le paiement, puis
// le clic sur l'arrivée déplacent la figurine.
#[test]
fn session_casts_underground_travel_to_the_clicked_space() {
    let catalog = french();
    let mut session = session_of(decor_events(
        TimeOfDay::Day,
        &[spell(UndergroundTravel)],
        Sandbox::default().dice,
        vec![crystal(BasicColor::Green)],
    ));
    session.command(Command::ChooseCast(underground_choice()));
    let payment = session
        .screen(&catalog)
        .mana_prompt
        .expect("bandeau du sort")
        .payments[0]
        .command;
    session.command(payment);
    let destination = session
        .screen(&catalog)
        .cast_destinations
        .into_iter()
        .find(|destination| destination.hex == VILLAGE)
        .expect("le village est une arrivée");
    session.command(destination.command);
    session.run_frame();
    assert_eq!(session.view().figure, Some(VILLAGE));
}

fn discarded() -> Vec<CardId> {
    vec![
        card(Stamina, 1),
        card(Swiftness, 1),
        card(Concentration, 1),
        card(Promise, 1),
    ]
}

/// Méditation (ou Transe si `strong`, de nuit) lancée, avec `discarded()` dans la défausse.
fn meditating(strong: bool) -> GameState {
    meditating_events(strong)
        .iter()
        .fold(GameState::default(), apply)
}

fn meditating_events(strong: bool) -> Vec<Event> {
    let meditation = spell(Meditation);
    let mut hand = vec![meditation, card(March, 1)];
    hand.extend(discarded());
    let mut extra: Vec<Event> = discarded()
        .into_iter()
        .map(|card| Event::CardDiscarded { player: SOLO, card })
        .collect();
    extra.extend([crystal(BasicColor::Green), black_token()]);
    let time = if strong {
        TimeOfDay::Night
    } else {
        TimeOfDay::Day
    };
    let mut events = decor_events(time, &hand, Sandbox::default().dice, extra);
    let state = events.iter().fold(GameState::default(), apply);
    events.extend(events_of(
        &state,
        &cast(meditation, 0, strong, CastTargets::default()),
    ));
    events
}

// Spec 057 C19, C20 (écran) : la fenêtre de Méditation, « Sur le deck » et « Sous le deck ».
#[test]
fn meditation_window_places_cards() {
    let catalog = french();
    let state = meditating(false);
    let pending = view(&state, SOLO).meditation.expect("cartes en attente");
    let window = screen_in(&state, &ScreenMode::Normal, &catalog)
        .meditation
        .expect("fenêtre de Méditation");
    assert_eq!(window.title, "Méditation : où remettre ces cartes ?");
    assert_eq!(window.cards.len(), 2);
    let top = window.top.expect("Sur le deck");
    assert_eq!(top.label, "Sur le deck");
    let place = Action::PlaceOnDeck {
        top: true,
        order: pending.cards.clone(),
    };
    assert_eq!(top.command, Command::Play(index_of(&state, &place)));
    // L'ordre se choisit en cliquant les cartes.
    let reversed = vec![pending.cards[1], pending.cards[0]];
    let window = screen_in(&state, &ScreenMode::Meditating(reversed.clone()), &catalog)
        .meditation
        .expect("fenêtre de Méditation");
    let bottom = window.bottom.expect("Sous le deck");
    let place = Action::PlaceOnDeck {
        top: false,
        order: reversed,
    };
    assert_eq!(bottom.command, Command::Play(index_of(&state, &place)));

    // Transe : deux cartes à choisir parmi la défausse ; rien tant qu'elles ne le sont pas.
    let state = meditating(true);
    let window = screen_in(&state, &ScreenMode::Normal, &catalog)
        .meditation
        .expect("fenêtre de Transe");
    assert_eq!(window.cards.len(), discarded().len());
    assert!(window.top.is_none() && window.bottom.is_none());
    assert_eq!(
        window.cards[0].pick,
        Some(Command::PickMeditationCard(discarded()[0]))
    );
    let chosen = vec![discarded()[2], discarded()[0]];
    let window = screen_in(&state, &ScreenMode::Meditating(chosen.clone()), &catalog)
        .meditation
        .expect("fenêtre de Transe");
    assert_eq!(window.cards[2].place, Some(1));
    let place = Action::PlaceOnDeck {
        top: true,
        order: chosen,
    };
    assert_eq!(
        window.top.map(|button| button.command),
        Some(Command::Play(index_of(&state, &place)))
    );
    // Dans la session : un clic choisit, un second clic retire.
    let mut session = support::session_of(meditating_events(true));
    for card in [
        discarded()[2],
        discarded()[1],
        discarded()[1],
        discarded()[0],
    ] {
        session.command(Command::PickMeditationCard(card));
    }
    let window = session
        .screen(&catalog)
        .meditation
        .expect("fenêtre de Transe");
    assert_eq!(
        window
            .cards
            .iter()
            .map(|card| card.place)
            .collect::<Vec<_>>(),
        [Some(2), None, Some(1), None]
    );
}

// Spec 057 C17, C18 (écran) : « Utiliser » une unité de l'offre, « Recruter (gratuit) ».
#[test]
fn offer_units_use_and_free_recruit() {
    let catalog = french();
    let call = spell(CallToArms);
    // Des Paysans : Influence 2 et Mouvement 2 servent hors combat.
    let peasants = UnitCard::new(Unit::Peasants, 1);
    let mut extra = unit_events(&[], &[peasants]);
    extra.extend([
        crystal(BasicColor::White),
        crystal(BasicColor::White),
        black_token(),
    ]);
    let state = state_of(
        TimeOfDay::Night,
        &[call, card(March, 1)],
        Sandbox::default().dice,
        extra,
    );
    let using = done(&state, cast(call, 0, false, CastTargets::default()));
    let window = offers_window_in(
        &view(&using, SOLO),
        &legal_actions(&using, SOLO),
        &ScreenMode::Normal,
        &catalog,
    );
    let shown = &window.columns[2].cards[0];
    assert!(
        shown
            .uses
            .iter()
            .any(|button| button.label.starts_with("Utiliser")),
        "{shown:?}"
    );

    let recruiting = done(&state, cast(call, 0, true, CastTargets::default()));
    let window = offers_window_in(
        &view(&recruiting, SOLO),
        &legal_actions(&recruiting, SOLO),
        &ScreenMode::Normal,
        &catalog,
    );
    let take = window.columns[2].cards[0].take.clone().expect("Recruter");
    assert_eq!(take.label, "Recruter (gratuit)");
    let recruit = Action::RecruitUnit {
        unit: peasants,
        disband: None,
        bonds: false,
    };
    assert_eq!(take.command, Command::Play(index_of(&recruiting, &recruit)));
}

// Spec 057 C15, C27 (écran) : « Tour supplémentaire » dans la barre de menu, la carte écartée
// avec la défausse, le dé gardé avec les dés pris.
#[test]
fn extra_turn_kept_die_and_set_aside_shown() {
    let catalog = french();
    let (claim, bending) = (spell(ManaClaim), spell(SpaceBending));
    let state = state_of(
        TimeOfDay::Night,
        &[claim, bending, card(March, 1)],
        vec![
            ManaColor::Basic(BasicColor::Blue),
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Black,
        ],
        vec![
            crystal(BasicColor::Blue),
            crystal(BasicColor::Blue),
            black_token(),
        ],
    );
    let state = done(
        &state,
        cast(
            claim,
            0,
            false,
            CastTargets {
                die: Some(1),
                ..CastTargets::default()
            },
        ),
    );
    let state = done(&state, cast(bending, 0, true, CastTargets::default()));
    let end_turn = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour");
    let state = done(&state, end_turn);
    let seen = view(&state, SOLO);
    let bar = bar_of(&seen, &catalog);
    assert!(bar.round.contains("Tour supplémentaire"), "{}", bar.round);
    let screen = screen_in(&state, &ScreenMode::Normal, &catalog);
    // Sous le bouton de la défausse, que le panneau affiche (vérification à l'œil).
    assert_eq!(
        screen.set_aside.as_deref(),
        Some("Écartées jusqu'à la fin de la manche : Distorsion spatiale")
    );
    // Pas le bandeau d'une nouvelle manche : la Source relancée est celle de la fin du tour.
    assert_eq!(screen.new_round, None);
    assert!(
        screen
            .source
            .taken
            .iter()
            .any(|die| die.tooltip.contains("gardé jusqu'à la fin de la manche")),
        "{:?}",
        screen.source.taken
    );
}

fn ending() -> ScreenMode {
    ScreenMode::EndingTurn(EndTurnChoice {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: None,
    })
}

// Spec 057 C15, C21 (écran) : le bandeau de fin du tour annonce la pioche réelle : aucune carte
// avant un tour supplémentaire, deux de plus après Méditation.
#[test]
fn end_turn_prompt_counts_time_bending_and_draw_bonus() {
    let catalog = french();
    let bending = spell(SpaceBending);
    let state = state_of(
        TimeOfDay::Night,
        &[bending, card(March, 1)],
        vec![ManaColor::Basic(BasicColor::Blue), ManaColor::Black],
        vec![crystal(BasicColor::Blue), black_token()],
    );
    let state = done(&state, cast(bending, 0, true, CastTargets::default()));
    let prompt = screen_in(&state, &ending(), &catalog)
        .end_turn_prompt
        .expect("bandeau de fin du tour");
    assert_eq!(prompt.draw, "Vous ne piocherez aucune carte.");

    // Méditation : la main garde March et les quatre cartes de la défausse moins celles remises
    // sur le deck ; la limite de main 5 plus 2.
    let state = meditating(false);
    let place = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::PlaceOnDeck { top: true, .. }))
        .expect("placer sur le deck");
    let state = done(&state, place);
    let kept = view(&state, SOLO).hand.len();
    let prompt = screen_in(&state, &ending(), &catalog)
        .end_turn_prompt
        .expect("bandeau de fin du tour");
    assert_eq!(
        prompt.draw,
        format!("Vous piocherez {} carte(s).", 7 - kept)
    );
}

// Issue privée 260 : l'effet en cours posé par un sort renforcé se nomme par l'effet joué, comme la zone
// de jeu (Distorsion spatiale renforcée : Distorsion temporelle) ; lancé de base, par le sort.
#[test]
fn modifier_line_names_the_strong_spell_effect() {
    let catalog = french();
    let bending = spell(SpaceBending);
    let modifier_lines = |black: bool| {
        let state = state_of(
            TimeOfDay::Night,
            &[bending, card(March, 1)],
            vec![ManaColor::Basic(BasicColor::Blue), ManaColor::Black],
            vec![crystal(BasicColor::Blue), black_token()],
        );
        let state = done(&state, cast(bending, 0, black, CastTargets::default()));
        screen_in(&state, &ScreenMode::Normal, &catalog)
            .modifiers
            .expect("effets en cours")
            .lines
    };
    assert_eq!(
        modifier_lines(true),
        ["Distorsion temporelle : tour supplémentaire à la fin du tour"]
    );
    assert!(
        modifier_lines(false)
            .iter()
            .all(|line| line.starts_with("Distorsion spatiale : ")),
        "{:?}",
        modifier_lines(false)
    );
}

// --- Cibles cochées (issue privée 258) ---

fn swordsmen() -> arcanum_core::EnemyToken {
    token(arcanum_core::Enemy::Swordsmen, 1)
}

/// De nuit, Ailes du vent et Marche en main, un jeton noir, combat engagé contre trois ennemis
/// des orques ②, puis Marche jouée : 2 points de Mouvement.
fn wings_of_night_session() -> arcanum_ui::GameSession {
    let mut events = Sandbox {
        time: TimeOfDay::Night,
        figure: Some(PLAIN_W),
        hand: vec![spell(WingsOfWind), card(March, 1)],
        placed: vec![
            (prowlers(), ORCS_2),
            (swordsmen(), ORCS_2),
            (wolf_riders(), ORCS_2),
        ],
        ..Sandbox::default()
    }
    .events();
    events.push(black_token());
    let mut session = session_of(events);
    support::play(&mut session, &challenge(ORCS_2));
    support::play(&mut session, &Action::StartCombat);
    support::play(&mut session, &basic(card(March, 1)));
    assert_eq!(session.view().move_points, 2);
    session
}

fn wings_of_night_choice() -> CastChoice {
    CastChoice {
        card: spell(WingsOfWind),
        option: Some((true, 0)),
        from_offer: false,
        picks: CastPicks::default(),
    }
}

/// Les ennemis visés par l'action que joue chaque bouton du bandeau du sort.
fn cast_enemies(
    session: &arcanum_ui::GameSession,
    catalog: &Catalog,
) -> Vec<Vec<arcanum_core::EnemyToken>> {
    let legal = session.legal_actions();
    session
        .screen(catalog)
        .mana_prompt
        .expect("bandeau du sort")
        .payments
        .iter()
        .map(|button| match button.command {
            Command::Play(index) => match legal[index] {
                Action::PlayCard {
                    play: CardPlay::Cast { targets, .. },
                    ..
                } => targets.enemies.to_vec(),
                _ => panic!("{:?} n'est pas un sort", legal[index]),
            },
            other => panic!("{other:?} ne joue pas le sort"),
        })
        .collect()
}

// Spec 057 C8 (écran), issue privée 258 : les ennemis d'Ailes de la nuit se cochent sur les cartes de
// l'écran de combat ; le bandeau ne garde que le paiement, avec le coût en Mouvement des ennemis
// cochés, et rien à valider quand la réserve ne suffit pas.
#[test]
fn wings_of_night_targets_are_ticked_on_enemy_cards() {
    // Carte (LL) ; Compendium, *Wings of Night* S1 : 0 point pour le 1er ennemi, 1 pour le 2e, 2
    // pour le 3e.
    let catalog = french();
    let mut session = wings_of_night_session();
    session.command(Command::ChooseCast(wings_of_night_choice()));
    let toggle_of = |session: &arcanum_ui::GameSession, token| {
        session
            .combat_screen(&catalog)
            .expect("écran de combat")
            .enemies
            .into_iter()
            .find(|enemy| enemy.token == token)
            .and_then(|enemy| enemy.target)
            .expect("case du sort")
    };
    for token in [prowlers(), swordsmen(), wolf_riders()] {
        let toggle = toggle_of(&session, token);
        assert_eq!(toggle.command, Command::ToggleCastEnemy(token));
        assert!(!toggle.checked);
    }
    // Aucun ennemi coché : rien à valider.
    assert!(cast_enemies(&session, &catalog).is_empty());

    session.command(Command::ToggleCastEnemy(prowlers()));
    session.command(Command::ToggleCastEnemy(swordsmen()));
    assert!(toggle_of(&session, prowlers()).checked);
    assert!(!toggle_of(&session, wolf_riders()).checked);
    let prompt = session.screen(&catalog).mana_prompt.expect("bandeau");
    assert!(prompt.text.contains("Mouvement : 1"), "{}", prompt.text);
    assert!(
        prompt
            .payments
            .iter()
            .all(|button| !button.label.contains("Orques maraudeurs")),
        "{:?}",
        prompt.payments
    );
    let sets = cast_enemies(&session, &catalog);
    assert!(!sets.is_empty());
    assert!(
        sets.iter()
            .all(|set| set.len() == 2 && set.contains(&prowlers()) && set.contains(&swordsmen()))
    );

    // Le troisième coûterait 3 Mouvement, la réserve n'en a que 2.
    session.command(Command::ToggleCastEnemy(wolf_riders()));
    let prompt = session.screen(&catalog).mana_prompt.expect("bandeau");
    assert!(prompt.text.contains("Mouvement : 3"), "{}", prompt.text);
    assert!(prompt.payments.is_empty());

    session.command(Command::ToggleCastEnemy(wolf_riders()));
    let payment = cast_enemies(&session, &catalog);
    assert!(!payment.is_empty());
    let first = session
        .screen(&catalog)
        .mana_prompt
        .expect("bandeau")
        .payments[0]
        .command;
    session.command(first);
    session.run_frame();
    assert_eq!(session.view().move_points, 1);
}

/// De nuit, en forêt, Restauration en main, un jeton noir, trois unités épuisées.
fn rebirth_session(units: &[UnitCard]) -> arcanum_ui::GameSession {
    let mut events = Sandbox {
        time: TimeOfDay::Night,
        figure: Some(FOREST),
        hand: vec![spell(Restoration)],
        ..Sandbox::default()
    }
    .events();
    events.extend(unit_events(units, &[]));
    events.extend(units.iter().map(|&unit| Event::UnitActivated {
        player: SOLO,
        unit,
        ability: 0,
        option: 0,
        effect: arcanum_core::Effect::Nothing,
        enemy: None,
    }));
    events.push(black_token());
    session_of(events)
}

const FOREST: arcanum_core::Hex = arcanum_core::Hex::new(1, -1);

// Spec 057 C11 (écran), issue privée 258 : les unités de Renaissance se cochent dans le panneau du bas ;
// sans unité cochée, le bandeau propose le soin seul.
#[test]
fn rebirth_targets_are_ticked_on_unit_lines() {
    // Carte ; Compendium, *Rebirth* S2 : des unités épuisées de 3 niveaux en tout, 5 en forêt.
    let catalog = french();
    let units = [
        UnitCard::new(Unit::UtemCrossbowmen, 1),
        UnitCard::new(Unit::Peasants, 1),
        UnitCard::new(Unit::GuardianGolems, 1),
    ];
    let mut session = rebirth_session(&units);
    assert!(
        session
            .screen(&catalog)
            .units
            .units
            .iter()
            .all(|line| line.spell_target.is_none())
    );
    let rebirth = CastChoice {
        card: spell(Restoration),
        option: Some((true, 0)),
        from_offer: false,
        picks: CastPicks::default(),
    };
    session.command(Command::ChooseCast(rebirth));
    let lines = session.screen(&catalog).units.units;
    for (line, unit) in lines.iter().zip(units) {
        let toggle = line.spell_target.as_ref().expect("case du sort");
        assert_eq!(toggle.command, Command::ToggleCastUnit(unit));
        assert!(!toggle.checked);
    }
    let cast_units = |session: &arcanum_ui::GameSession| -> Vec<Vec<UnitCard>> {
        let legal = session.legal_actions();
        session
            .screen(&catalog)
            .mana_prompt
            .expect("bandeau du sort")
            .payments
            .iter()
            .map(|button| match button.command {
                Command::Play(index) => match legal[index] {
                    Action::PlayCard {
                        play: CardPlay::Cast { targets, .. },
                        ..
                    } => targets.units.to_vec(),
                    _ => panic!("pas un sort"),
                },
                other => panic!("{other:?}"),
            })
            .collect()
    };
    // Sans unité cochée : le soin seul.
    let heal_only = cast_units(&session);
    assert!(!heal_only.is_empty());
    assert!(heal_only.iter().all(Vec::is_empty), "{heal_only:?}");

    for unit in units {
        session.command(Command::ToggleCastUnit(unit));
    }
    assert!(session.screen(&catalog).units.units.iter().all(|line| {
        line.spell_target
            .as_ref()
            .is_some_and(|toggle| toggle.checked)
    }));
    let all = cast_units(&session);
    assert!(!all.is_empty());
    assert!(
        all.iter()
            .all(|set| set.len() == 3 && units.iter().all(|unit| set.contains(unit))),
        "{all:?}"
    );
}
