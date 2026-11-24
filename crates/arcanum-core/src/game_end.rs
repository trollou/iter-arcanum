//! Fin de partie et décompte final (spec 063) : dernier tour, raison de la fin, Hauts Faits.

use crate::{DeedCard, Feature, FinalScoring, GameState, PlayerId, ScenarioEnd, Site};

/// Où en est le dernier tour annoncé par la révélation d'une cité (MKUE p. 18, *Scenario End*).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LastTurn {
    /// Aucune cité révélée (ou un scénario sans dernier tour).
    #[default]
    NotAnnounced,
    /// Annoncé pendant ce tour : le tour qui suit est le dernier.
    Next,
    /// Ce tour est le dernier : sa fin termine la partie.
    Current,
}

/// Pourquoi la partie s'est arrêtée (spec 063).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GameEndReason {
    /// Une cité a été révélée et le dernier tour est joué (MKUE p. 18).
    CityRevealed,
    /// La dernière manche est finie (MKUE p. 4, 5.a).
    LastRound,
    /// Toutes les cités sont conquises et le dernier tour est joué (MKUE p. 19, *Solo Conquest* ;
    /// spec 064).
    AllCitiesConquered,
}

/// La raison de la fin quand le dernier tour (ou la manche qui finit pendant lui) s'achève : la
/// fin du scénario qui l'a annoncé (specs 063 et 064).
pub(crate) fn last_turn_reason(state: &GameState) -> GameEndReason {
    match state.scenario().map(|scenario| scenario.end()) {
        Some(ScenarioEnd::AllCitiesConquered) => GameEndReason::AllCitiesConquered,
        _ => GameEndReason::CityRevealed,
    }
}

/// Spec 064 : le dernier tour annoncé par la conquête de la dernière cité du scénario (MKUE p. 19,
/// *Scenario End*), à mettre juste après `pending`, les événements produits mais pas encore
/// appliqués. « Toutes » : les cités du scénario, révélées ou non (Compendium, *Final Scoring* S3).
pub(crate) fn conquest_last_turn(
    state: &GameState,
    player: PlayerId,
    pending: &[crate::Event],
) -> Option<crate::Event> {
    let scenario = state.scenario()?;
    if scenario.end() != ScenarioEnd::AllCitiesConquered
        || state.last_turn != LastTurn::NotAnnounced
    {
        return None;
    }
    let newly: Vec<crate::Hex> = pending
        .iter()
        .filter_map(|event| match event {
            crate::Event::CityConquered { hex, .. } => Some(*hex),
            _ => None,
        })
        .filter(|hex| !state.city(*hex).is_some_and(|city| city.is_conquered()))
        .collect();
    if newly.is_empty() {
        return None;
    }
    let before = state
        .cities()
        .values()
        .filter(|city| city.is_conquered())
        .count();
    (before + newly.len() >= usize::from(scenario.tile_deck().core_city))
        .then_some(crate::Event::LastTurnAnnounced { player })
}

/// Les six Hauts Faits du décompte final, dans l'ordre de la carte de score (MKUE p. 15 ; noms
/// de la prise en main FR p. 19).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Achievement {
    /// Le Savant : sorts et actions avancées (specs 050, 054).
    Knowledge,
    /// Le Collectionneur : artefacts et cristaux.
    Loot,
    /// Le Commandant : unités (spec 042).
    Leader,
    /// Le Conquérant : boucliers sur forteresse, tour de mage, monastère (specs 041, 043).
    Conqueror,
    /// L'Aventurier : boucliers sur un site d'aventure (spec 044).
    Adventurer,
    /// La Victime : Blessures du deck.
    Beating,
}

impl Achievement {
    pub const ALL: [Achievement; 6] = [
        Achievement::Knowledge,
        Achievement::Loot,
        Achievement::Leader,
        Achievement::Conqueror,
        Achievement::Adventurer,
        Achievement::Beating,
    ];
}

/// Le décompte final d'un joueur : ce qu'il compte, pas les points, qui s'en déduisent. Ce que le
/// jeu ne connaît pas encore (bannières) vaut 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalScore {
    pub fame: u32,
    /// Cristaux de l'inventaire, toutes couleurs confondues.
    pub crystals: u32,
    /// Blessures de toutes les cartes Acte du joueur : deck, main, défausse, zone de jeu.
    pub wounds: u32,
    /// Actions avancées et sorts du joueur, comptés comme les Blessures (spec 050).
    pub advanced_actions: u32,
    pub spells: u32,
    /// Le Commandant : la somme des niveaux des unités, une unité blessée comptant la moitié de
    /// son niveau, arrondie à l'inférieur (MKUE p. 15 ; spec 042).
    pub unit_levels: u32,
    /// Le Collectionneur : artefacts du joueur, comptés comme les sorts (spec 041).
    pub artifacts: u32,
    /// Le Conquérant : jetons Bouclier du joueur sur une forteresse, une tour de mage ou un
    /// monastère (spec 041).
    pub shields: u32,
    /// L'Aventurier : jetons Bouclier du joueur sur un site d'aventure (spec 044).
    pub adventure_shields: u32,
    /// Spec 064 : les points des cités et du temps, seulement pour le décompte de Conquête en
    /// solitaire.
    pub conquest: Option<ConquestScore>,
}

/// La partie « conquête » du décompte (MKUE p. 19, *Scoring* ; carte de score, *Co-op and Solo
/// Scenarios* ; spec 064) : ce qu'elle compte, les points s'en déduisent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConquestScore {
    /// Cités conquises.
    pub conquered: u32,
    /// Cités du scénario, révélées ou non.
    pub cities: u32,
    /// Manches après celle où la partie a fini.
    pub unplayed_rounds: u32,
    /// Cartes encore dans le deck du joueur fantôme.
    pub dummy_deck: u32,
    /// La fin de la manche a été annoncée pendant la dernière manche (par le joueur ou le fantôme).
    pub round_announced: bool,
    /// Mission accomplie : la partie a fini par la conquête de toutes les cités.
    pub all_conquered: bool,
}

/// Une ligne de la partie « conquête » du décompte, dans l'ordre de la spec 064.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConquestLine {
    Cities,
    AllCities,
    UnplayedRounds,
    DummyDeck,
    RoundNotAnnounced,
}

impl ConquestLine {
    pub const ALL: [ConquestLine; 5] = [
        ConquestLine::Cities,
        ConquestLine::AllCities,
        ConquestLine::UnplayedRounds,
        ConquestLine::DummyDeck,
        ConquestLine::RoundNotAnnounced,
    ];
}

impl ConquestScore {
    /// Points d'une ligne.
    pub fn points(&self, line: ConquestLine) -> i64 {
        // Q2 : le temps gagné ne compte que si la mission est accomplie.
        let success = |points: i64| if self.all_conquered { points } else { 0 };
        match line {
            ConquestLine::Cities => 10 * i64::from(self.conquered),
            ConquestLine::AllCities => success(15),
            ConquestLine::UnplayedRounds => success(30 * i64::from(self.unplayed_rounds)),
            ConquestLine::DummyDeck => success(i64::from(self.dummy_deck)),
            ConquestLine::RoundNotAnnounced => success(if self.round_announced { 0 } else { 5 }),
        }
    }

    /// La somme des cinq lignes.
    pub fn total(&self) -> i64 {
        ConquestLine::ALL
            .into_iter()
            .map(|line| self.points(line))
            .sum()
    }
}

/// La progression de Conquête en solitaire (spec 064, C22), publique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConquestProgress {
    pub conquered: u32,
    pub cities: u32,
}

/// La progression, seulement pour un scénario qui finit par la conquête de toutes les cités.
pub fn conquest_progress(state: &GameState) -> Option<ConquestProgress> {
    let scenario = state.scenario()?;
    (scenario.end() == ScenarioEnd::AllCitiesConquered).then(|| ConquestProgress {
        conquered: u32::try_from(
            state
                .cities()
                .values()
                .filter(|city| city.is_conquered())
                .count(),
        )
        .unwrap_or(u32::MAX),
        cities: u32::from(scenario.tile_deck().core_city),
    })
}

/// La partie « conquête » du décompte, sur l'état complet (L8) : rien de neuf n'est gardé pour
/// elle. L'annonce de la fin de la dernière manche se lit dans `round_over` : vrai après
/// `RoundEnded`, que plus aucune manche ne suit une fois la partie finie.
fn conquest_score(state: &GameState) -> Option<ConquestScore> {
    let scenario = state.scenario()?;
    if scenario.scoring() != FinalScoring::SoloConquest {
        return None;
    }
    let progress = conquest_progress(state)?;
    let all_conquered = state.game_end() == Some(GameEndReason::AllCitiesConquered);
    Some(ConquestScore {
        conquered: progress.conquered,
        cities: progress.cities,
        unplayed_rounds: if all_conquered {
            u32::from(scenario.rounds()).saturating_sub(state.round())
        } else {
            0
        },
        dummy_deck: state.dummy().map_or(0, |dummy| {
            u32::try_from(dummy.deck().len()).unwrap_or(u32::MAX)
        }),
        round_announced: state.round_over,
        all_conquered,
    })
}

impl FinalScore {
    /// Points d'un Haut Fait, sans bonus de titre (MKUE p. 15 et p. 18, *Solo Variant*).
    pub fn points(&self, achievement: Achievement) -> i64 {
        match achievement {
            // 1 par action avancée, 2 par sort (spec 050).
            Achievement::Knowledge => i64::from(self.advanced_actions) + 2 * i64::from(self.spells),
            // 2 par Bouclier sur forteresse, tour de mage ou monastère (spec 041).
            Achievement::Conqueror => 2 * i64::from(self.shields),
            // 2 par Bouclier sur un site d'aventure (spec 044).
            Achievement::Adventurer => 2 * i64::from(self.adventure_shields),
            Achievement::Leader => i64::from(self.unit_levels),
            // 2 par artefact (spec 041), 1 par deux cristaux, arrondi à l'inférieur.
            Achievement::Loot => 2 * i64::from(self.artifacts) + i64::from(self.crystals / 2),
            Achievement::Beating => -2 * i64::from(self.wounds),
        }
    }

    /// La gloire plus les points des six Hauts Faits, et ceux de la conquête ; peut être négatif.
    pub fn total(&self) -> i64 {
        self.conquest.map_or(0, |conquest| conquest.total())
            + i64::from(self.fame)
            + Achievement::ALL
                .into_iter()
                .map(|achievement| self.points(achievement))
                .sum::<i64>()
    }
}

/// Le décompte final calculé sur l'état complet (MKUE p. 15 ; Compendium, *Final Scoring* S4,
/// S5) : une carte jetée hors du jeu n'est plus dans les cartes du joueur, elle ne compte pas.
pub fn final_score(state: &GameState, player: PlayerId) -> FinalScore {
    let crystals = state.inventory(player).total();
    let count = |wanted: fn(DeedCard) -> bool| {
        let count = state
            .deck(player)
            .iter()
            .chain(state.hand(player))
            .chain(state.discard_pile(player))
            .chain(state.play_area(player).iter().map(|played| &played.card))
            .filter(|card| wanted(card.card()))
            .count();
        u32::try_from(count).unwrap_or(u32::MAX)
    };
    // Les jetons Bouclier du joueur sur les sites qui répondent à `wanted`.
    let shields_on = |wanted: fn(Site) -> bool| {
        let count = state
            .shields()
            .iter()
            .filter(|&(&hex, &owner)| {
                owner == player
                    && matches!(
                        state.map().space(hex).and_then(|space| space.feature),
                        Some(Feature::Site(site)) if wanted(site)
                    )
            })
            .count();
        u32::try_from(count).unwrap_or(u32::MAX)
    };
    FinalScore {
        fame: state.fame(player),
        crystals,
        wounds: count(DeedCard::is_wound),
        advanced_actions: count(DeedCard::is_advanced_action),
        spells: count(DeedCard::is_spell),
        unit_levels: state
            .units(player)
            .iter()
            .map(|owned| {
                let level = u32::from(owned.card.unit.definition().level);
                if owned.is_wounded() { level / 2 } else { level }
            })
            .sum(),
        // Spec 055 : une bannière sur une unité compte comme un artefact du joueur.
        artifacts: count(DeedCard::is_artifact).saturating_add(
            u32::try_from(
                state
                    .units(player)
                    .iter()
                    .filter(|owned| owned.banner.is_some())
                    .count(),
            )
            .unwrap_or(u32::MAX),
        ),
        // MKUE p. 15, *The Greatest Conqueror* : forteresse, tour de mage, monastère.
        shields: shields_on(|site| matches!(site, Site::Keep | Site::MageTower | Site::Monastery)),
        // MKUE p. 15, *The Greatest Adventurer* : donjon, tombeau, antre, Sombre Terrier, ruines.
        adventure_shields: shields_on(Site::is_adventure),
        conquest: conquest_score(state),
    }
}
