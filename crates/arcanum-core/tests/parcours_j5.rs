//! Parcours de bout en bout du jalon 5 (spec 059, ADR 0006) : une partie de Première
//! reconnaissance en solo où le héros monte de niveau, choisit compétence et action avancée, les
//! joue, gagne puis achète des sorts et les lance, traverse jusqu'à un lac, y rate un défi et subit
//! la retraite forcée, jusqu'au décompte final.

mod support;

use arcanum_core::{
    Action, AdvancedAction, BasicAction, BasicColor, CardId, CardPlay, CombatPhase, CommonCard,
    CommonDeck, DamageTarget, DeedCard, Effect, EnemyAbility, EnemyAttack, EnemyColor, EnemyToken,
    Event, EventLog, ForcedWithdrawalPreview, GameConfig, HealingTarget, Hero, Hex, ManaColor,
    Offer, PlayerView, Resistance, RngPositions, Scenario, Site, Skill, SkillState, Spell, Terrain,
    TileId, TilePosition, TimeOfDay, routes,
};
use support::arrangement::Setup;
use support::{
    Parcours, SOLO, all_events, card_plays, combat, expect_terrain, has, influence_amount,
    move_amount, route_to, single_card, step_cost,
};

/// Une graine quelconque : la mise en place arrangée ne dépend plus d'elle (spec 059, « Départ et
/// préconditions ») ; `balayer_les_graines` mesure combien la suite en supporte.
const SEED: u64 = 0;
/// Spec 059, Q7 : le parcours principal passe au moins 90 graines sur 100.
const SWEEP_THRESHOLD: usize = 90;

const PORTAL: Hex = Hex::new(0, 0);
/// La clairière magique de la tuile de campagne 2, d'où se révèlent 2A puis A + B.
const GLADE: Hex = Hex::new(4, -3);
/// La plaine de la campagne 3 par où passer hors de portée des orques de `(3, -3)`.
const PASSAGE: Hex = Hex::new(5, -4);
/// Le village de la campagne 4, voisin de la tour.
const NEXT_TO_TOWER: Hex = Hex::new(4, -4);
const TOWER: Hex = Hex::new(4, -5);
/// Le village de la campagne 1, voisin du lac.
const LAKE_VILLAGE: Hex = Hex::new(2, -3);
const LAKE: Hex = Hex::new(2, -4);
/// Les orques de la campagne 1 défiés depuis le lac.
const LAKE_ORCS: Hex = Hex::new(1, -4);

/// Les emplacements 2A et A + B (`docs/reference/bac-a-sable.md`).
fn position_2a() -> TilePosition {
    TilePosition::from_lattice(2, 0)
}

fn position_ab() -> TilePosition {
    TilePosition::from_lattice(1, 1)
}

/// Un tour ne suffit pas toujours pour marcher : borne des tours de marche.
const MAX_WALKING_TURNS: u32 = 6;
/// Borne des tours d'exploration : au-delà, l'explorateur est bloqué.
const MAX_EXPLORER_TURNS: u32 = 60;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Spec 059, L1 : un départ qui ne remplit plus une précondition se revoit, pas la graine.
fn precondition(holds: bool, what: &str) {
    assert!(holds, "départ à revoir : {what}");
}

fn deed(card: DeedCard) -> CommonCard {
    CommonCard::Deed(CardId::new(card, 1))
}

fn token(enemy: arcanum_core::Enemy) -> EnemyToken {
    EnemyToken::new(enemy, 1)
}

/// Spec 059, « Départ et préconditions » : ce que les deux tests arrangent de la même façon.
/// Rôdeurs puis Creuseurs : les errants posés sur `(1, -4)` puis `(3, -3)` à la mise en place.
fn common_setup(seed: u64) -> Setup {
    Setup::new(config(seed)).enemies(
        EnemyColor::Green,
        &[
            token(arcanum_core::Enemy::Prowlers),
            token(arcanum_core::Enemy::Diggers),
        ],
    )
}

/// Le parcours principal : figurine sur la clairière, Moines pour la tour, offres et compétences
/// des choix scriptés, et le deck et la Source qui font tenir C2 à C13 dans la première manche, de
/// jour.
fn main_setup(seed: u64) -> Setup {
    common_setup(seed)
        .figure(GLADE)
        .enemies(EnemyColor::Violet, &[token(arcanum_core::Enemy::Monks)])
        // Restauration en tête, prise à la récompense : elle soigne les Blessures de l'assaut.
        .offer(
            CommonDeck::Spells,
            &[
                deed(DeedCard::Spell(Spell::Restoration)),
                deed(DeedCard::Spell(Spell::SpaceBending)),
            ],
        )
        // Lecture d'esprit, distribuée à la place de Restauration : achetée.
        .next_cards(
            CommonDeck::Spells,
            &[deed(DeedCard::Spell(Spell::MindRead))],
        )
        .offer(
            CommonDeck::AdvancedActions,
            &[deed(DeedCard::Advanced(AdvancedAction::FrostBridge))],
        )
        .skills(&[Skill::DarkPaths, Skill::DarkNegotiation])
        .deck_cards(&MAIN_DECK)
        .source(&MAIN_SOURCE)
}

/// Le deck d'Arythea, tour par tour (main de 5, pioche de fin de tour) :
/// 1. Cristallisation (le dé blanc en cristal blanc), puis 8 points de Mouvement : les deux
///    explorations et la marche jusqu'à `(4, -4)` ;
/// 2. Marche renforcée (dé vert) et Marche : le désert de jour (5) ; Rage, Polyvalence et
///    Détermination : l'attaque 5 contre les Moines (leurs dégâts ne sont pas parés) ;
/// 3. (l'action avancée et le sort gagnés, sur les Blessures) ;
/// 4. Promesse, Menace, Improvisation et deux cartes de côté : 7 Influence pour l'achat ;
/// 5. (le sort acheté) et Concentration : le jeton de sa couleur s'il en manque.
const MAIN_DECK: [DeedCard; 16] = [
    DeedCard::Basic(BasicAction::Crystallize),
    DeedCard::Basic(BasicAction::Stamina),
    DeedCard::Basic(BasicAction::Stamina),
    DeedCard::Basic(BasicAction::Swiftness),
    DeedCard::Basic(BasicAction::Swiftness),
    DeedCard::Basic(BasicAction::March),
    DeedCard::Basic(BasicAction::March),
    DeedCard::Basic(BasicAction::Rage),
    DeedCard::Basic(BasicAction::BattleVersatility),
    DeedCard::Basic(BasicAction::Determination),
    DeedCard::Basic(BasicAction::Promise),
    DeedCard::Basic(BasicAction::Threaten),
    DeedCard::Basic(BasicAction::Improvisation),
    DeedCard::Basic(BasicAction::Tranquility),
    DeedCard::Basic(BasicAction::ManaPull),
    DeedCard::Basic(BasicAction::Concentration),
];

/// Blanc pour le cristal du tour 1 ; vert pour Marche renforcée (tour 2) et pour Restauration (tour
/// 3). Ni bleu ni or : le Mouvement du tour 1 ne prend aucun dé.
const MAIN_SOURCE: [ManaColor; 3] = [
    ManaColor::Basic(BasicColor::White),
    ManaColor::Basic(BasicColor::Green),
    ManaColor::Basic(BasicColor::Green),
];

/// L'épisode du lac (spec 059, Q9) : figurine sur le village, Rôdeurs sur `(1, -4)`, un dé bleu ;
/// puis Pont de glace gagné de l'offre d'actions avancées, et pioché.
fn lake_setup(seed: u64) -> Setup {
    let frost_bridge = CardId::new(DeedCard::Advanced(AdvancedAction::FrostBridge), 1);
    common_setup(seed)
        .figure(LAKE_VILLAGE)
        .offer(
            CommonDeck::AdvancedActions,
            &[CommonCard::Deed(frost_bridge)],
        )
        .source(&LAKE_SOURCE)
        .gain_advanced_action(frost_bridge)
}

const LAKE_SOURCE: [ManaColor; 3] = [
    ManaColor::Basic(BasicColor::Blue),
    ManaColor::Basic(BasicColor::Green),
    ManaColor::Basic(BasicColor::Red),
];

/// Où en est le script : ce qui décide des cartes gardées en main.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Assault,
    Purchase,
    Lake,
    Explore,
}

/// Ce que le script retient d'une décision à l'autre : rejoué tel quel par la partie reprise (C22).
#[derive(Clone, Debug)]
struct Script {
    phase: Phase,
    /// Le sort de la récompense, l'action avancée et la compétence gagnés à la conquête.
    spell: Option<CardId>,
    advanced_action: Option<CardId>,
    skill: Option<Skill>,
    bought: Option<CardId>,
    /// Les cartes gagnées pas encore jouées : jamais jouées de côté ni défaussées.
    keep: Vec<CardId>,
    /// La manche où la compétence a servi, et C18 vérifié.
    skill_round: Option<u32>,
    new_round_checked: bool,
}

impl Script {
    fn new() -> Self {
        Self {
            phase: Phase::Assault,
            spell: None,
            advanced_action: None,
            skill: None,
            bought: None,
            keep: Vec::new(),
            skill_round: None,
            new_round_checked: false,
        }
    }

    /// Les cartes que le script ne joue pas pour se déplacer et ne défausse pas.
    fn kept(&self, card: CardId) -> bool {
        if self.keep.contains(&card) {
            return true;
        }
        match self.phase {
            Phase::Assault => matches!(
                card.card(),
                DeedCard::Basic(
                    BasicAction::BattleVersatility
                        | BasicAction::Determination
                        | BasicAction::Improvisation
                )
            ),
            Phase::Purchase => matches!(
                card.card(),
                DeedCard::Basic(
                    BasicAction::Promise | BasicAction::Threaten | BasicAction::Improvisation
                )
            ),
            Phase::Lake => Some(card) == self.advanced_action,
            Phase::Explore => false,
        }
    }
}

/// Spec 059, « Les choix scriptés » : un sort dont l'effet de base sert hors combat, sans déplacer
/// la figurine (elle reste sur la tour jusqu'à l'achat).
fn useful_spell(spell: Spell, view: &PlayerView) -> bool {
    match spell {
        Spell::SpaceBending
        | Spell::Meditation
        | Spell::ManaClaim
        | Spell::MindRead
        | Spell::CallToArms => true,
        Spell::Restoration => view.hand.iter().any(|card| card.card().is_wound()),
        Spell::EnergyFlow => view.units.iter().any(|unit| !unit.ready),
        _ => false,
    }
}

fn spell_of(card: CardId) -> Option<Spell> {
    match card.card() {
        DeedCard::Spell(spell) => Some(spell),
        _ => None,
    }
}

/// Spec 059 : une compétence d'Arythea qui a une option utilisable hors combat (`UseSkill`).
fn useful_skill(skill: Skill) -> bool {
    matches!(
        skill,
        Skill::DarkPaths | Skill::DarkNegotiation | Skill::DarkFireMagic | Skill::MotivationArythea
    )
}

/// Spec 059 : les deux actions avancées des cartes 1 à 16 qui rendent un lac franchissable.
fn lake_action(card: CardId) -> bool {
    matches!(
        card.card(),
        DeedCard::Advanced(AdvancedAction::FrostBridge | AdvancedAction::SongOfWind)
    )
}

fn is_song_of_wind(card: CardId) -> bool {
    card.card() == DeedCard::Advanced(AdvancedAction::SongOfWind)
}

/// Un mana de cette couleur payé parmi ces événements (de jour, un dé doré la remplace).
fn paid(events: &[Event], color: BasicColor) -> bool {
    has(events, |e| match e {
        Event::DieTaken { color: c, .. } => *c == ManaColor::Basic(color) || *c == ManaColor::Gold,
        Event::CrystalUsed { color: c, .. } => *c == color,
        Event::ManaTokenUsed { color: c, .. } => *c == ManaColor::Basic(color),
        _ => false,
    })
}

fn wounds_in(cards: &[CardId]) -> usize {
    cards.iter().filter(|card| card.card().is_wound()).count()
}

/// Un défenseur que les cartes d'Arythea au corps à corps peuvent vaincre (spec 059, préconditions).
fn beatable_defender(view: &PlayerView, enemy: arcanum_core::Enemy) -> bool {
    let definition = enemy.definition();
    !definition.resistances.contains(&Resistance::Physical)
        && definition.attack != EnemyAttack::Summon
        && wounds_from(view, enemy) < view.hand_limit
}

/// Les Blessures que l'attaque non parée de cet ennemi met en main (MKUE p. 9, *Assign Damage* ;
/// Brutal double les dégâts).
fn wounds_from(view: &PlayerView, enemy: arcanum_core::Enemy) -> usize {
    let definition = enemy.definition();
    let attack = match definition.attack {
        EnemyAttack::Physical(n)
        | EnemyAttack::Fire(n)
        | EnemyAttack::Ice(n)
        | EnemyAttack::ColdFire(n) => u32::from(n),
        EnemyAttack::Summon => u32::MAX / 4,
    };
    let damage = if definition.abilities.contains(&EnemyAbility::Brutal) {
        attack * 2
    } else {
        attack
    };
    damage.div_ceil(u32::from(view.armor.max(1))) as usize
}

struct J5 {
    p: Parcours,
    s: Script,
}

impl J5 {
    fn view(&self) -> PlayerView {
        self.p.view()
    }

    fn legal(&self) -> Vec<Action> {
        self.p.legal()
    }

    fn play(&mut self, action: Action) -> Vec<Event> {
        self.p.play(action)
    }

    /// Joue des cartes jusqu'à avoir `points` points de mouvement : à chaque fois celle qui en donne
    /// le plus, sans les cartes gardées. Rend faux s'il en manque.
    fn gather_move(&mut self, points: u32) -> bool {
        while self.view().move_points < points {
            let best = card_plays(&self.p)
                .into_iter()
                .filter(|(_, card, play, _)| !self.s.kept(*card) && single_card(play))
                .filter_map(|(action, _, _, effect)| move_amount(effect).map(|n| (n, action)))
                .max_by_key(|(n, _)| *n);
            let Some((_, action)) = best else {
                return false;
            };
            self.play(action);
        }
        true
    }

    /// Joue des cartes jusqu'à ce que l'attaque en cours atteigne `needed`.
    fn gather_combat(&mut self, needed: u32, what: &str) {
        loop {
            let current = combat(&self.view());
            if current.in_progress_total().unwrap_or(0) >= needed {
                return;
            }
            let best = card_plays(&self.p)
                .into_iter()
                .filter(|(_, _, _, effect)| current.efficiency(*effect).is_some())
                .filter_map(|(action, _, play, effect)| {
                    let points = current.counted_points(effect)?;
                    let cards = match play {
                        CardPlay::Basic {
                            discard: Some(_), ..
                        }
                        | CardPlay::Strong {
                            discard: Some(_), ..
                        } => 2,
                        _ => 1,
                    };
                    Some((points * 10 / cards, action))
                })
                .max_by_key(|(score, _)| *score);
            precondition(best.is_some(), &format!("assez de cartes pour {what}"));
            let Some((_, action)) = best else { return };
            self.play(action);
        }
    }

    /// Fin du tour (MKUE p. 11) : annoncer la fin de la manche si le deck est vide et rien n'est
    /// joué ; sinon défausser toutes les Blessures si le repos le permet, et les cartes non gardées,
    /// pour renouveler la main. Puis la récompense d'un niveau pair, et les vérifications du début
    /// d'un tour.
    fn finish_turn(&mut self) -> Vec<Event> {
        let s = self.s.clone();
        self.finish_turn_keeping(|card| s.kept(card))
    }

    fn finish_turn_keeping(&mut self, kept: impl Fn(CardId) -> bool) -> Vec<Event> {
        let view = self.view();
        let mut events =
            if view.play_area.is_empty() && self.legal().contains(&Action::AnnounceEndOfRound) {
                self.play(Action::AnnounceEndOfRound)
            } else {
                let action = self
                    .legal()
                    .into_iter()
                    .filter_map(|action| match &action {
                        Action::EndTurn {
                            discard,
                            reclaim: None,
                            glade: None,
                            steady_tempo: false,
                        } => {
                            let wounds = wounds_in(discard);
                            let kept_out = discard.iter().filter(|c| kept(**c)).count();
                            let others = discard.len() - wounds - kept_out;
                            Some(((wounds, usize::MAX - kept_out, others), action))
                        }
                        _ => None,
                    })
                    .max_by_key(|(key, _)| *key)
                    .map(|(_, action)| action)
                    .unwrap_or_else(|| panic!("une fin de tour légale : {:?}", self.legal()));
                self.play(action)
            };
        events.extend(support::take_pending_reward(&mut self.p));
        self.at_turn_start(&events);
        events
    }

    /// C18 et C19, vérifiés au début de chaque tour.
    fn at_turn_start(&mut self, events: &[Event]) {
        let view = self.view();
        if view.game_end.is_some() {
            return;
        }
        // C19 — spec 051 : la compétence reste dans la vue.
        if let Some(skill) = self.s.skill {
            assert!(view.skills.iter().any(|s| s.skill == skill));
        }
        // C18 — MKUE p. 4, 2.g ; p. 5, *Using Skills* 4 ; specs 023, 051.
        if let Some(round) = self.s.skill_round
            && !self.s.new_round_checked
            && view.round > round
        {
            self.s.new_round_checked = true;
            assert!(has(events, |e| matches!(e, Event::RoundStarted { .. })));
            let skill = self.s.skill.expect("la compétence gagnée");
            let state = view
                .skills
                .iter()
                .find(|s| s.skill == skill)
                .map(|s| s.state);
            assert_eq!(state, Some(SkillState::Ready));
            assert_eq!(view.shields.get(&TOWER), Some(&SOLO));
            assert!(view.level >= 2);
            assert!(view.discard.is_empty(), "le deck reformé avec la défausse");
            for card in [self.s.spell, self.s.advanced_action, self.s.bought]
                .into_iter()
                .flatten()
            {
                assert!(!view.discard.contains(&card));
                assert!(!view.play_area.iter().any(|played| played.card == card));
            }
        }
    }

    /// Marche case par case, sur un ou plusieurs tours, le long de `steps`, en gardant `extra`
    /// points de mouvement à l'arrivée ; le tour n'est pas fini à l'arrivée.
    fn walk(&mut self, steps: &[Hex], extra: u32) -> Vec<Event> {
        let mut events = Vec::new();
        let mut next = 0;
        for _ in 0..MAX_WALKING_TURNS {
            let view = self.view();
            let remaining: u32 = steps[next..].iter().map(|hex| step_cost(&view, *hex)).sum();
            self.gather_move(remaining + extra);
            while let Some(&hex) = steps.get(next) {
                if !self.legal().contains(&Action::MoveHero { to: hex }) {
                    break;
                }
                events.extend(self.play(Action::MoveHero { to: hex }));
                next += 1;
            }
            if next == steps.len() && self.gather_move(extra) {
                return events;
            }
            self.finish_turn();
        }
        precondition(false, "assez de cartes de Mouvement pour marcher");
        events
    }

    /// Les décisions en attente après un sort (Méditation : cartes à remettre sur le deck ;
    /// Restauration : les points de soin dépensés sur les Blessures de la main).
    fn resolve_pending(&mut self) {
        while let Some(action) = self.legal().into_iter().find(|action| {
            matches!(
                action,
                Action::PlaceOnDeck { .. }
                    | Action::SpendHealing {
                        on: HealingTarget::Wound
                    }
            )
        }) {
            self.play(action);
        }
    }

    /// Concentration jouée pour un jeton de la couleur du sort, s'il n'a pas son mana (spec 059,
    /// « Départ et préconditions » : Concentration comprise).
    fn token_for(&mut self, spell: CardId) -> bool {
        let color = spell
            .card()
            .definition()
            .expect("un sort a une définition")
            .color;
        let Some((action, ..)) = card_plays(&self.p)
            .into_iter()
            .find(|(_, card, _, effect)| {
                card.card() == DeedCard::Basic(BasicAction::Concentration)
                    && *effect == Effect::GainToken(color)
            })
        else {
            return false;
        };
        self.play(action);
        true
    }

    /// C11 et C13 : le sort lancé pour son effet de base, payé d'un mana de sa couleur, s'il le peut.
    fn try_cast(&mut self, card: CardId) -> bool {
        let casts: Vec<Action> = self
            .legal()
            .into_iter()
            .filter(|action| {
                matches!(
                    action,
                    Action::PlayCard {
                        card: c,
                        play: CardPlay::Cast { .. },
                    } if *c == card
                )
            })
            .collect();
        let basic = casts.iter().find(|action| {
            matches!(
                action,
                Action::PlayCard {
                    play: CardPlay::Cast {
                        black: None,
                        from_offer: false,
                        ..
                    },
                    ..
                }
            )
        });
        let Some(action) = basic.cloned() else {
            return self.token_for(card) && self.try_cast(card);
        };
        // MKUE p. 4, 3.b : de jour, l'effet renforcé ne se lance pas.
        if self.view().time_of_day == TimeOfDay::Day {
            assert!(!casts.iter().any(|action| matches!(
                action,
                Action::PlayCard {
                    play: CardPlay::Cast { black: Some(_), .. },
                    ..
                }
            )));
        }
        let color = card
            .card()
            .definition()
            .expect("un sort a une définition")
            .color;
        let events = self.play(action);
        assert!(has(&events, |e| matches!(
            e,
            Event::CardPlayed {
                card: c,
                play: CardPlay::Cast { black: None, .. },
                ..
            } if *c == card
        )));
        assert!(paid(&events, color), "payé d'un mana de sa couleur");
        self.resolve_pending();
        self.s.keep.retain(|c| *c != card);
        true
    }
}

/// C1 — MKUE p. 5, *Using Skills* 2 ; specs 051 (C2, C3), 050.
fn setup(j: &J5) {
    let view = j.view();
    assert_eq!(
        view.scenario,
        Some(Scenario::first_reconnaissance().without_dummy())
    );
    assert_eq!((view.fame, view.level), (0, 1));
    assert!(view.skills.is_empty());
    assert!(view.skill_choice.is_empty() && view.common_skills.is_empty());
    assert_eq!(view.skill_pile_count, 10);
    assert_eq!(view.offers[&Offer::Spells].len(), 3);
    assert_eq!((view.round, view.time_of_day), (1, TimeOfDay::Day));
    assert_eq!(view.figure, Some(GLADE));
    for (hex, terrain) in [
        (PORTAL, Terrain::Plains),
        (GLADE, Terrain::Forest),
        (LAKE_VILLAGE, Terrain::Plains),
        (LAKE, Terrain::Lake),
        (LAKE_ORCS, Terrain::Forest),
    ] {
        expect_terrain(&j.p, hex, terrain);
    }
}

/// Spec 059, « Départ et préconditions » : C2 à C13 tiennent dans la première manche, de jour.
fn first_round_by_day(j: &J5, what: &str) {
    let view = j.view();
    assert_eq!(
        (view.round, view.time_of_day),
        (1, TimeOfDay::Day),
        "{what} hors de la première manche, de jour"
    );
}

/// C2 et C3 : de la clairière, exploration de 2A puis de A + B ; la case voisine de la tour.
fn explore_to_the_tower(j: &mut J5) {
    // Le cristal blanc de l'achat (C12), avant que le Mouvement ne prenne des cartes.
    let crystal = card_plays(&j.p)
        .into_iter()
        .find(|(_, _, _, effect)| *effect == Effect::PayForCrystal(BasicColor::White));
    precondition(
        crystal.is_some(),
        "Cristallisation et un dé blanc au tour 1",
    );
    j.play(crystal.expect("vérifié").0);
    let to_glade = route_to(&j.p, GLADE);
    j.walk(&to_glade, 2);
    assert_eq!(j.view().figure, Some(GLADE));

    // C2 — carte Mage Tower ; specs 015, 043 (C1), 063 (C3).
    let fame = j.view().fame;
    let events = j.play(Action::Explore {
        position: position_2a(),
    });
    assert!(events.contains(&Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: position_2a(),
    }));
    assert!(events.contains(&Event::FameGained {
        player: SOLO,
        fame: 1,
    }));
    assert_eq!(j.view().fame, fame + 1);
    let mut ready = j.gather_move(2);
    for _ in 0..MAX_WALKING_TURNS {
        if ready {
            break;
        }
        j.finish_turn();
        ready = j.gather_move(2);
    }
    precondition(ready, "de quoi explorer A + B");
    let events = j.play(Action::Explore {
        position: position_ab(),
    });
    assert!(events.contains(&Event::TileRevealed {
        tile: TileId::Countryside(4),
        position: position_ab(),
    }));
    assert!(has(&events, |e| matches!(
        e,
        Event::DefenderPlaced { hex, token } if *hex == TOWER && token.color() == EnemyColor::Violet
    )));
    assert!(events.contains(&Event::FameGained {
        player: SOLO,
        fame: 1,
    }));
    let view = j.view();
    assert_eq!(view.fame, fame + 2);
    expect_terrain(&j.p, NEXT_TO_TOWER, Terrain::Plains);
    expect_terrain(&j.p, TOWER, Terrain::Desert);
    expect_terrain(&j.p, PASSAGE, Terrain::Plains);
    assert_eq!(
        view.face_down_defenders.get(&TOWER),
        Some(&vec![EnemyColor::Violet])
    );
    assert!(!view.enemy_tokens.contains_key(&TOWER));

    // C3 — carte Mage Tower ; specs 043 (C4), 030 : de jour, le pas qui arrive à côté retourne le
    // défenseur.
    let steps = route_to(&j.p, NEXT_TO_TOWER);
    assert_eq!(steps, [PASSAGE, NEXT_TO_TOWER]);
    let events = j.walk(&steps, 0);
    assert_eq!(j.view().figure, Some(NEXT_TO_TOWER));
    first_round_by_day(j, "l'arrivée à côté de la tour");
    assert!(has(&events, |e| matches!(
        e,
        Event::EnemyTokenRevealed { hex, .. } if *hex == TOWER
    )));
    let view = j.view();
    assert!(!view.face_down_defenders.contains_key(&TOWER));
    let defender = view.enemy_tokens[&TOWER][0];
    assert_eq!(defender.color(), EnemyColor::Violet);
    precondition(
        beatable_defender(&view, defender.enemy()),
        "un défenseur battable au corps à corps",
    );
    assert!(!has(&all_events(&j.p), |e| matches!(
        e,
        Event::EnemiesProvoked { .. }
    )));
    j.finish_turn();
}

/// C4 : l'assaut, le combat et la conquête.
fn conquer(j: &mut J5) {
    let mut ready = false;
    for _ in 0..MAX_WALKING_TURNS {
        if j.gather_move(step_cost(&j.view(), TOWER)) {
            ready = true;
            break;
        }
        j.finish_turn();
    }
    precondition(ready, "de quoi payer le pas de l'assaut");

    // C4 — MKUE p. 7, 4.a ; p. 8, 1.a ; spec 043 (C11).
    first_round_by_day(j, "l'assaut");
    let events = j.play(Action::MoveHero { to: TOWER });
    assert!(events.contains(&Event::AssaultStarted {
        player: SOLO,
        site: TOWER,
        from: NEXT_TO_TOWER,
    }));
    assert!(events.contains(&Event::ReputationChanged {
        player: SOLO,
        change: -1,
    }));
    j.play(Action::StartCombat);
    let defender = combat(&j.view()).enemies[0].clone();
    assert_eq!(defender.hex, TOWER);
    assert!(defender.site_fortified);
    let enemy = defender.token.enemy();
    precondition(
        beatable_defender(&j.view(), enemy),
        "un défenseur battable au corps à corps",
    );

    // Sans attaque de siège, le défenseur fortifié ne se combat qu'au corps à corps ; ses dégâts
    // vont au héros.
    let mut fight = Vec::new();
    fight.extend(j.play(Action::NextCombatPhase));
    fight.extend(j.play(Action::NextCombatPhase));
    while let Some(action) = j.legal().into_iter().find(|action| {
        matches!(
            action,
            Action::AssignDamage {
                to: DamageTarget::Hero,
                ..
            }
        )
    }) {
        fight.extend(j.play(action));
    }
    if combat(&j.view()).phase != CombatPhase::Attack {
        fight.extend(j.play(Action::NextCombatPhase));
    }
    assert_eq!(combat(&j.view()).phase, CombatPhase::Attack);
    j.play(Action::DeclareAttack {
        targets: vec![defender.token],
    });
    j.gather_combat(u32::from(enemy.definition().armor), "vaincre le défenseur");
    fight.extend(j.play(Action::ResolveAttack));
    assert!(has(&fight, |e| matches!(
        e,
        Event::EnemiesDefeated { enemies, .. } if enemies == &vec![defender.token]
    )));

    // C4 — MKUE p. 10, *Combat Outcomes* 2.c ; spec 043 (C20).
    let events = j.play(Action::EndCombat);
    assert!(events.contains(&Event::SiteConquered {
        player: SOLO,
        hex: TOWER,
    }));
    let view = j.view();
    assert_eq!(view.shields.get(&TOWER), Some(&SOLO));
    assert_eq!(view.spell_rewards, 1);

    // L2 — spec 034 : jamais assommé pendant l'assaut.
    assert!(!has(&fight, |e| matches!(e, Event::KnockedOut { .. })));
}

/// C5 à C8 : la fin du tour de la conquête, ses trois choix et la pioche.
fn end_of_conquest_turn(j: &mut J5) {
    // C8 : au plus trois cartes en main, pour que la pioche donne les deux cartes gagnées.
    let hand = j.view().hand;
    precondition(
        wounds_in(&hand) <= 3,
        "au plus trois Blessures en main après l'assaut",
    );
    let others: Vec<CardId> = hand
        .iter()
        .copied()
        .filter(|card| !card.card().is_wound())
        .collect();
    let mut events = j.play(Action::EndTurn {
        discard: others,
        reclaim: None,
        glade: None,
        steady_tempo: false,
    });

    // C5 — MKUE p. 11, 5.c ; p. 5, *"Gain" Effects* 3.a ; spec 043.
    let view = j.view();
    let offer = view.offers[&Offer::Spells].clone();
    let legal = j.legal();
    let takes: Vec<CommonCard> = legal
        .iter()
        .filter_map(|action| match action {
            Action::TakeSpell { card } => Some(CommonCard::Deed(*card)),
            _ => None,
        })
        .collect();
    assert_eq!(takes.len(), legal.len(), "seuls les sorts de l'offre");
    assert_eq!(takes, offer);
    let spell = offer
        .iter()
        .filter_map(|card| card.deed())
        .find(|card| spell_of(*card).is_some_and(|s| useful_spell(s, &view)));
    precondition(spell.is_some(), "un sort hors combat à la récompense");
    let spell = spell.expect("vérifié");
    let taken = j.play(Action::TakeSpell { card: spell });
    let gained = taken
        .iter()
        .position(|e| {
            *e == Event::OfferCardGained {
                player: SOLO,
                offer: Offer::Spells,
                card: CommonCard::Deed(spell),
                to: arcanum_core::CardDestination::DeckTop,
            }
        })
        .expect("le sort gagné");
    assert!(taken[gained..].iter().any(|e| matches!(
        e,
        Event::OfferCardDealt {
            offer: Offer::Spells,
            ..
        }
    )));
    let view = j.view();
    assert_eq!(view.offers[&Offer::Spells].len(), 3);
    assert!(!view.offers[&Offer::Spells].contains(&CommonCard::Deed(spell)));
    events.extend(taken.iter().cloned());

    // C6 — MKUE p. 11, 6.b ; spec 051 (C4, C5). Les compétences sont révélées après le choix du
    // sort (issue privée 317).
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 2,
    }));
    let revealed = taken
        .iter()
        .find_map(|e| match e {
            Event::SkillsRevealed { skills, .. } => Some(skills.clone()),
            _ => None,
        })
        .expect("deux compétences révélées après le sort");
    assert_eq!(revealed.len(), 2);
    assert!(revealed.iter().all(|s| s.hero() == Hero::Arythea));
    assert_eq!(view.skill_choice, revealed);
    let chooses: Vec<Skill> = j
        .legal()
        .iter()
        .filter_map(|action| match action {
            Action::ChooseSkill { skill } => Some(*skill),
            _ => None,
        })
        .collect();
    assert_eq!(chooses, revealed);
    assert_eq!(j.legal().len(), 2);
    let skill = revealed.iter().copied().find(|s| useful_skill(*s));
    precondition(skill.is_some(), "une compétence utilisable hors combat");
    let skill = skill.expect("vérifié");
    j.play(Action::ChooseSkill { skill });
    let view = j.view();
    let owned: Vec<(Skill, SkillState)> = view.skills.iter().map(|s| (s.skill, s.state)).collect();
    assert_eq!(owned, [(skill, SkillState::Ready)]);
    let other = revealed.iter().copied().find(|s| *s != skill);
    let common: Vec<Skill> = view.common_skills.iter().map(|s| s.skill).collect();
    assert_eq!(common, other.into_iter().collect::<Vec<_>>());
    assert_eq!(view.skill_pile_count, 8);

    // C7 — MKUE p. 11, 6.b ; specs 050, 051 (C5, C6).
    let offer = view.offers[&Offer::AdvancedActions].clone();
    let gains: Vec<CommonCard> = j
        .legal()
        .iter()
        .filter_map(|action| match action {
            Action::GainAdvancedAction { card } => Some(CommonCard::Deed(*card)),
            _ => None,
        })
        .collect();
    assert_eq!(gains, offer);
    let card = offer
        .iter()
        .filter_map(|card| card.deed())
        .find(|card| lake_action(*card) && !is_song_of_wind(*card))
        .or_else(|| {
            offer
                .iter()
                .filter_map(|card| card.deed())
                .find(|card| is_song_of_wind(*card))
        });
    precondition(
        card.is_some(),
        "Pont de glace ou Mélodie du vent dans l'offre",
    );
    let card = card.expect("vérifié");
    let events = j.play(Action::GainAdvancedAction { card });
    assert!(events.contains(&Event::OfferCardGained {
        to: arcanum_core::CardDestination::DeckTop,
        player: SOLO,
        offer: Offer::AdvancedActions,
        card: CommonCard::Deed(card),
    }));
    let view = j.view();
    assert_eq!(view.offers[&Offer::AdvancedActions].len(), offer.len());
    assert!(!view.offers[&Offer::AdvancedActions].contains(&CommonCard::Deed(card)));
    assert!(!j.legal().iter().any(|action| matches!(
        action,
        Action::TakeSpell { .. } | Action::ChooseSkill { .. } | Action::GainAdvancedAction { .. }
    )));

    // C8 — MKUE p. 5, 3 ; p. 11, 7.b ; specs 043, 050 : l'action avancée, au-dessus, piochée la
    // première, puis le sort.
    let drawn: Vec<CardId> = events
        .iter()
        .filter_map(|e| match e {
            Event::CardDrawn { card, .. } => Some(*card),
            _ => None,
        })
        .collect();
    assert_eq!(drawn.get(..2), Some(&[card, spell][..]));
    assert!(view.hand.contains(&card) && view.hand.contains(&spell));

    j.s.spell = Some(spell);
    j.s.advanced_action = Some(card);
    j.s.skill = Some(skill);
    j.s.keep = vec![card, spell];
    j.s.phase = Phase::Purchase;
    j.at_turn_start(&events);
}

/// C9 : la compétence gagnée utilisée, son effet visible dans la vue.
fn use_skill(j: &mut J5) {
    let skill = j.s.skill.expect("la compétence gagnée");
    let usable = |j: &J5| {
        j.legal()
            .into_iter()
            .find(|action| matches!(action, Action::UseSkill { skill: s, .. } if *s == skill))
    };
    // Sombre négociation : son Influence ne sert qu'en interaction, à la tour conquise.
    if usable(j).is_none() && j.legal().contains(&Action::Interact) {
        j.play(Action::Interact);
    }
    let action = usable(j);
    precondition(action.is_some(), "la compétence utilisable au tour suivant");
    let before = j.view();
    let events = j.play(action.expect("vérifié"));
    let effect = events
        .iter()
        .find_map(|e| match e {
            Event::SkillUsed {
                skill: s, effect, ..
            } if *s == skill => Some(*effect),
            _ => None,
        })
        .expect("SkillUsed");
    let view = j.view();
    match effect {
        Effect::Move(n) => assert_eq!(view.move_points, before.move_points + u32::from(n)),
        Effect::Influence(n) => {
            assert_eq!(
                view.influence_points,
                before.influence_points + i32::from(n)
            )
        }
        Effect::CrystalAndToken { .. } => {
            assert_eq!(view.inventory.total(), before.inventory.total() + 1)
        }
        Effect::DrawAndToken { cards, .. } => {
            assert_eq!(view.hand.len(), before.hand.len() + usize::from(cards))
        }
        other => panic!("effet inattendu : {other:?}"),
    }
    let state = view
        .skills
        .iter()
        .find(|s| s.skill == skill)
        .map(|s| s.state);
    assert!(matches!(
        state,
        Some(SkillState::UsedThisTurn | SkillState::Flipped)
    ));
    assert!(usable(j).is_none());
    j.s.skill_round = Some(view.round);
}

/// C10 : l'action avancée gagnée jouée pour son effet de base.
fn play_advanced_action(j: &mut J5) {
    let card = j.s.advanced_action.expect("l'action avancée gagnée");
    let action = j
        .legal()
        .into_iter()
        .find(|action| {
            matches!(
                action,
                Action::PlayCard {
                    card: c,
                    play: CardPlay::Basic { .. },
                } if *c == card
            )
        })
        .expect("l'action avancée se joue pour son effet");
    let before = j.view().move_points;
    let events = j.play(action);
    assert!(has(&events, |e| matches!(
        e,
        Event::CardPlayed {
            card: c,
            play: CardPlay::Basic { .. },
            ..
        } if *c == card
    )));
    // Pont de glace et Mélodie du vent : Mouvement 2 de base.
    assert_eq!(j.view().move_points, before + 2);
    j.s.keep.retain(|c| *c != card);
}

/// C9 à C11 : le tour qui suit la conquête.
fn play_gained_cards(j: &mut J5) {
    use_skill(j);
    play_advanced_action(j);
    let spell = j.s.spell.expect("le sort gagné");
    if !j.try_cast(spell) {
        j.finish_turn();
        precondition(j.try_cast(spell), "le mana du sort gagné");
    }
    j.finish_turn();
}

/// C12 : sur la tour conquise, l'achat d'un sort ; rend vrai s'il est fait ce tour.
fn try_purchase(j: &mut J5) -> bool {
    let view = j.view();
    if !view.offers[&Offer::Spells]
        .iter()
        .filter_map(|card| card.deed())
        .any(|card| spell_of(card).is_some_and(|s| useful_spell(s, &view)))
    {
        return false;
    }
    // Un essai, repris si les 7 points et le mana ne sont pas là (MKUE p. 5, *Reverting*).
    let mut decisions = 0;
    let undo = |j: &mut J5, decisions: usize| {
        for _ in 0..decisions {
            j.p.undo().expect("l'essai d'achat se reprend");
        }
    };
    if j.view().interaction.is_none() {
        if !j.legal().contains(&Action::Interact) {
            return false;
        }
        let events = j.play(Action::Interact);
        decisions += 1;
        assert!(has(&events, |e| matches!(
            e,
            Event::InteractionStarted {
                site: Site::MageTower,
                ..
            }
        )));
    }
    let buys = |j: &J5| -> Vec<(CardId, Action)> {
        let view = j.view();
        j.legal()
            .into_iter()
            .filter_map(|action| match &action {
                Action::BuySpell { card, .. }
                    if spell_of(*card).is_some_and(|s| useful_spell(s, &view)) =>
                {
                    Some((*card, action))
                }
                _ => None,
            })
            .collect()
    };
    while j.view().influence_points < 7 {
        // C12 : avant les 7 points, pas d'achat.
        assert!(
            !j.legal()
                .iter()
                .any(|action| matches!(action, Action::BuySpell { .. }))
        );
        let skill = j.s.skill;
        let mut best = card_plays(&j.p)
            .into_iter()
            .filter(|(_, card, play, _)| {
                single_card(play)
                    || matches!(
                        (card.card(), play),
                        (
                            DeedCard::Basic(BasicAction::Improvisation),
                            CardPlay::Basic { .. } | CardPlay::Strong { .. }
                        )
                    )
            })
            .filter(|(_, card, _, _)| !j.s.keep.contains(card))
            // Le mana est gardé pour le sort : l'Influence vient des effets de base et de côté.
            .filter(|(_, _, play, _)| !matches!(play, CardPlay::Strong { .. }))
            .filter_map(|(action, _, _, effect)| {
                let amount = influence_amount(effect).or(match effect {
                    Effect::InfluenceReputation { amount, .. } => Some(amount),
                    _ => None,
                })?;
                Some((amount, action))
            })
            .max_by_key(|(n, _)| *n)
            .map(|(_, action)| action);
        if best.is_none() {
            best = j.legal().into_iter().find(|action| {
                matches!(action, Action::UseSkill { skill: s, .. }
                    if Some(*s) == skill && *s == Skill::DarkNegotiation)
            });
        }
        let Some(action) = best else {
            undo(j, decisions);
            return false;
        };
        j.play(action);
        decisions += 1;
    }
    let Some((card, action)) = buys(j).into_iter().next() else {
        undo(j, decisions);
        return false;
    };
    let before = j.view().influence_points;
    let events = j.play(action);
    assert!(events.contains(&Event::SpellBought {
        player: SOLO,
        card,
        cost: 7,
    }));
    let view = j.view();
    assert_eq!(view.offers[&Offer::Spells].len(), 3);
    assert!(!view.offers[&Offer::Spells].contains(&CommonCard::Deed(card)));
    assert_eq!(view.influence_points, before - 7);
    j.s.bought = Some(card);
    j.s.keep.push(card);
    true
}

/// C12 et C13 : l'achat au tour qui suit les cartes gagnées, puis le sort acheté lancé, dans la
/// première manche, de jour.
fn purchase(j: &mut J5) {
    assert_eq!(j.view().figure, Some(TOWER));
    first_round_by_day(j, "l'achat");
    precondition(try_purchase(j), "7 Influence et le mana du sort à l'achat");
    j.s.phase = Phase::Explore;
    j.finish_turn();

    // C13 — MKUE p. 8, 4.d ; p. 4, 3.b ; spec 054 : le sort acheté, sur le dessus du deck, pioché.
    let card = j.s.bought.expect("le sort acheté");
    assert!(j.view().hand.contains(&card));
    first_round_by_day(j, "le lancer du sort acheté");
    precondition(j.try_cast(card), "le mana du sort acheté");
}

/// C14 à C17 : depuis le village, le lac, le défi raté et la retraite forcée.
fn lake(j: &mut J5) {
    let card = j.s.advanced_action.expect("l'action avancée gagnée");
    assert_eq!(j.view().figure, Some(LAKE_VILLAGE));
    let song = is_song_of_wind(card);
    let strong = |j: &J5| {
        j.legal().into_iter().find(|action| match action {
            Action::PlayCard {
                card: c,
                play:
                    CardPlay::Strong {
                        option, extra_mana, ..
                    },
            } if *c == card => {
                if song {
                    *option == 1 && extra_mana.is_some()
                } else {
                    *option == 0
                }
            }
            _ => false,
        })
    };
    precondition(
        strong(j).is_some(),
        "l'action avancée et son mana sur le village",
    );

    // C14 — MKUE p. 7, 7.b et 7.d ; carte Frost Bridge ; spec 052 (C14, C27).
    let before = j.view().move_points;
    let events = j.play(strong(j).expect("vérifié"));
    assert!(has(&events, |e| matches!(
        e,
        Event::CardPlayed { card: c, .. } if *c == card
    )));
    assert!(paid(&events, BasicColor::Blue), "payé d'un mana bleu");
    let view = j.view();
    let (gain, lake_cost) = if song { (2, 0) } else { (4, 1) };
    assert_eq!(view.move_points, before + gain);
    let to_lake = routes(&view.map, LAKE_VILLAGE, view.move_rules.clone())
        .into_iter()
        .find(|route| route.to == LAKE)
        .expect("le lac atteignable");
    assert_eq!(to_lake.cost, lake_cost);
    let events = j.play(Action::MoveHero { to: LAKE });
    assert!(has(&events, |e| matches!(
        e,
        Event::HeroMoved { to, .. } if *to == LAKE
    )));
    assert!(!has(&events, |e| matches!(
        e,
        Event::EnemiesProvoked { .. }
    )));
    let view = j.view();
    assert_eq!(
        view.forced_withdrawal,
        Some(ForcedWithdrawalPreview {
            to: LAKE_VILLAGE,
            wounds: 1,
        })
    );

    // C15 — MKUE p. 8, *Combat* 1 ; p. 10, *Combat Outcomes* ; specs 031, 034, 052.
    let orcs = view.enemy_tokens[&LAKE_ORCS][0];
    assert_eq!(orcs.enemy(), arcanum_core::Enemy::Prowlers);
    precondition(
        orcs.enemy().definition().attack != EnemyAttack::Summon
            && wounds_from(&view, orcs.enemy()) < view.hand_limit,
        "des orques qui n'assomment pas le héros",
    );
    let mut fight = j.play(Action::Challenge { enemy: LAKE_ORCS });
    assert!(fight.contains(&Event::EnemyChallenged {
        player: SOLO,
        enemy: LAKE_ORCS,
    }));
    fight.extend(j.play(Action::StartCombat));
    fight.extend(j.play(Action::NextCombatPhase));
    fight.extend(j.play(Action::NextCombatPhase));
    while let Some(action) = j.legal().into_iter().find(|action| {
        matches!(
            action,
            Action::AssignDamage {
                to: DamageTarget::Hero,
                ..
            }
        )
    }) {
        fight.extend(j.play(action));
    }
    if combat(&j.view()).phase != CombatPhase::Attack {
        fight.extend(j.play(Action::NextCombatPhase));
    }
    fight.extend(j.play(Action::EndCombat));
    assert!(has(&fight, |e| matches!(e, Event::CombatEnded { .. })));
    assert!(!has(&fight, |e| matches!(e, Event::EnemiesDefeated { .. })));
    assert!(has(&fight, |e| matches!(e, Event::WoundsTaken { .. })));
    // L2 — spec 034.
    assert!(!has(&fight, |e| matches!(e, Event::KnockedOut { .. })));
    let view = j.view();
    assert_eq!(view.enemy_tokens.get(&LAKE_ORCS), Some(&vec![orcs]));
    assert_eq!(view.figure, Some(LAKE));

    // C16 — MKUE p. 11, 2 ; FAQ p3 ; Compendium, *Forced Withdrawal* S3 ; spec 052 (C19, C22, C23).
    let wounds_before = wounds_in(&view.hand);
    let after = view
        .after_withdrawal
        .clone()
        .expect("la vue d'après la retraite");
    assert_eq!(wounds_in(&after.hand), wounds_before + 1);
    let events = j.finish_turn();
    // MKUE p. 11, 1 : les dés rendus à la Source d'abord, puis la retraite (2).
    match events
        .iter()
        .find(|e| !matches!(e, Event::DieReturned { .. }))
    {
        Some(Event::ForcedWithdrawal { path, wounds, .. }) => {
            assert_eq!(path, &vec![LAKE_VILLAGE]);
            assert_eq!(wounds.len(), 1);
        }
        other => panic!("la retraite forcée d'abord, pas {other:?}"),
    }
    assert!(!has(&events, |e| matches!(
        e,
        Event::EnemiesProvoked { .. } | Event::KnockedOut { .. }
    )));

    // C17 — MKUE p. 11, 2 et 4 ; spec 052 (C21, C41).
    assert!(has(&events, |e| matches!(e, Event::TurnStarted { .. })));
    let view = j.view();
    assert_eq!(view.figure, Some(LAKE_VILLAGE));
    assert_eq!(view.enemy_tokens.get(&LAKE_ORCS), Some(&vec![orcs]));
    assert!(
        !routes(&view.map, LAKE_VILLAGE, view.move_rules.clone())
            .iter()
            .any(|route| route.to == LAKE)
    );
    j.s.phase = Phase::Explore;
}

/// C19 : tours d'exploration jusqu'à la fin de la partie.
fn explore_to_the_end(j: &mut J5) {
    let mut turns = 0;
    while !j.p.is_finished() {
        turns += 1;
        precondition(
            turns <= MAX_EXPLORER_TURNS,
            "l'explorateur n'est pas bloqué",
        );
        let events = j.p.explorer_turn();
        j.at_turn_start(&events);
    }
    // C19 — MKUE p. 18, *Scenario End* ; spec 063 (C9, C10, C12).
    let view = j.view();
    assert!(view.game_end.is_some());
    assert!(j.legal().is_empty());
    let events = all_events(&j.p);
    assert!(matches!(events.last(), Some(Event::GameEnded { .. })));
    assert!(j.s.new_round_checked, "C18 vérifié");

    // L3 — spec 059 : aucun ennemi provoqué, un seul assaut.
    assert!(!has(&events, |e| matches!(
        e,
        Event::EnemiesProvoked { .. }
    )));
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::AssaultStarted { .. }))
            .count(),
        1
    );
}

/// C20 et C21 — MKUE p. 15 ; p. 18, *Scoring* ; specs 063 (C14), 043, 054.
fn check_score(j: &J5) {
    let view = j.view();
    let score = view.final_score.expect("le décompte de la partie finie");
    let events = all_events(&j.p);
    let gained = |offer: Offer| {
        events
            .iter()
            .filter(|e| matches!(e, Event::OfferCardGained { offer: o, .. } if *o == offer))
            .count() as u32
    };
    let bought = events
        .iter()
        .filter(|e| matches!(e, Event::SpellBought { .. }))
        .count() as u32;
    let spells = gained(Offer::Spells) + bought;
    let advanced_actions = gained(Offer::AdvancedActions);
    assert_eq!(spells, 2);
    assert_eq!(score.spells, spells);
    assert_eq!(score.advanced_actions, advanced_actions);
    use arcanum_core::Achievement::*;
    assert_eq!(
        score.points(Knowledge),
        2 * i64::from(spells) + i64::from(advanced_actions)
    );
    assert_eq!(score.shields, 1);
    assert_eq!(score.points(Conqueror), 2);
    assert_eq!(score.fame, view.fame);
    assert_eq!(
        score.total(),
        i64::from(view.fame)
            + [Knowledge, Loot, Leader, Conqueror, Adventurer, Beating]
                .into_iter()
                .map(|a| score.points(a))
                .sum::<i64>()
    );
}

/// Du début de la partie à la sauvegarde, après le tour de C9 à C11.
fn up_to_the_save_point(seed: u64) -> J5 {
    let mut j = J5 {
        p: Parcours::arranged(&main_setup(seed)),
        s: Script::new(),
    };
    setup(&j);
    explore_to_the_tower(&mut j);
    conquer(&mut j);
    end_of_conquest_turn(&mut j);
    play_gained_cards(&mut j);
    j
}

fn after_the_save_point(j: &mut J5) {
    purchase(j);
    explore_to_the_end(j);
    check_score(j);
}

fn parcours(seed: u64) -> J5 {
    let mut j = up_to_the_save_point(seed);
    after_the_save_point(&mut j);
    j
}

// C1 à C13 et C18 à C21 : le parcours principal, de la mise en place arrangée au décompte final.
#[test]
fn parcours_sorts_et_competences() {
    parcours(SEED);
}

// C14 à C17 : l'épisode du lac, depuis l'état posé (spec 059, Q9).
#[test]
fn parcours_lac_et_retraite_forcee() {
    let p = Parcours::arranged(&lake_setup(SEED));
    let frost_bridge = CardId::new(DeedCard::Advanced(AdvancedAction::FrostBridge), 1);
    let view = p.view();
    precondition(view.hand.contains(&frost_bridge), "Pont de glace en main");
    assert!(
        !view.offers[&Offer::AdvancedActions].contains(&CommonCard::Deed(frost_bridge)),
        "Pont de glace gagné de l'offre"
    );
    assert_eq!(view.offers[&Offer::AdvancedActions].len(), 3);
    let mut s = Script::new();
    s.phase = Phase::Lake;
    s.advanced_action = Some(frost_bridge);
    let mut j = J5 { p, s };
    lake(&mut j);
}

// C22 — ADR 0001 ; specs 024, 038 : reprise depuis la sauvegarde faite après les cartes gagnées.
#[test]
fn parcours_reprise_apres_les_cartes_gagnees() {
    let mut played = up_to_the_save_point(SEED);
    let saved: (EventLog, RngPositions) = played.p.save();
    let script = played.s.clone();
    after_the_save_point(&mut played);

    let mut resumed = J5 {
        p: Parcours::resume(&saved),
        s: script,
    };
    after_the_save_point(&mut resumed);
    assert_eq!(resumed.p.runner.log(), played.p.runner.log());
    assert_eq!(resumed.p.runner.state(), played.p.runner.state());
    assert_eq!(resumed.view().final_score, played.view().final_score);
}

/// Spec 059, Q7 : `cargo test -p arcanum-core --test parcours_j5 -- --ignored --nocapture` joue le
/// parcours principal sur les graines 0 à 99 et dit pourquoi les autres échouent.
#[test]
#[ignore]
fn balayer_les_graines() {
    let passed = support::sweep_seeds(0..100, |seed| {
        parcours(seed);
    });
    assert!(
        passed.len() >= SWEEP_THRESHOLD,
        "{} graines sur 100, moins que {SWEEP_THRESHOLD}",
        passed.len()
    );
}
