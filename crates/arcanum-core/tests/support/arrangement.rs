//! Mise en place arrangée des parcours (issue privée 318, ADR 0006) : la vraie mise en place de
//! `new_game`, où le test choisit l'ordre de certaines piles au lieu de l'attendre du hasard.
//!
//! Un arrangement échange deux éléments d'une même pile (deux cartes du deck du héros, deux jetons
//! d'une pile d'ennemis…) dans **tous** les événements de la mise en place : le mélange, et ce qui
//! a été tiré de la pile (main de départ, ennemis posés, offres). La mise en place reste donc
//! cohérente, comme si le mélange avait donné cet ordre. Les générateurs reprennent là où la vraie
//! mise en place les a laissés : la suite tire le même hasard qu'avec la graine seule.
//!
//! Après la mise en place, rien n'est posé : le moteur joue tout ce que le parcours vérifie. Seul un
//! **épisode posé** (ADR 0006) ajoute ensuite les événements qui posent sa situation
//! (`gain_advanced_action`) : à appeler en dernier, après les échanges.

use arcanum_core::{
    CardDestination, CardId, CommonCard, CommonDeck, DeedCard, Enemy, EnemyColor, EnemyToken,
    Event, EventLog, GameConfig, GameRngs, Hex, ManaColor, Offer, PlayerId, RngPositions, Skill,
    TileId, TileKind, UnitCard, UnitRank, new_game,
};

/// La vraie mise en place d'une partie, à arranger avant de la jouer.
#[derive(Clone, Debug)]
pub struct Setup {
    events: Vec<Event>,
    positions: RngPositions,
}

impl Setup {
    pub fn new(config: GameConfig) -> Self {
        let mut rngs: GameRngs = config.rngs();
        let events = new_game(&config, &mut rngs);
        for event in &events {
            assert!(
                known_setup_event(event),
                "événement de mise en place inconnu de l'arrangement : {event:?}"
            );
        }
        Self {
            events,
            positions: rngs.positions(),
        }
    }

    /// Les événements de la mise en place, arrangée.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Ce que reprend `Parcours::resume` : le journal de la mise en place et les générateurs.
    pub fn save(&self) -> (EventLog, RngPositions) {
        let mut log = EventLog::default();
        for event in &self.events {
            // La mise en place est la décision 0, comme dans `GameRunner::new`.
            log.push(0, event.clone());
        }
        (log, self.positions.clone())
    }

    /// `cards` en tête du deck du héros, dans l'ordre : la main de départ, puis la pioche.
    pub fn deck(mut self, cards: &[CardId]) -> Self {
        let order = self.hero_order();
        for (position, &card) in cards.iter().enumerate() {
            let current = order_at(&self.hero_order(), position, "deck du héros");
            assert!(order.contains(&card), "{card:?} hors du deck du héros");
            self.swap_everywhere(|event| swap_hero_card(event, current, card));
        }
        self
    }

    /// Comme `deck`, par carte : le premier exemplaire pas encore placé de chacune.
    pub fn deck_cards(self, cards: &[DeedCard]) -> Self {
        let order = self.hero_order();
        let mut chosen: Vec<CardId> = Vec::new();
        for &wanted in cards {
            let card = order
                .iter()
                .copied()
                .find(|card| card.card() == wanted && !chosen.contains(card))
                .unwrap_or_else(|| panic!("plus d'exemplaire de {wanted:?} dans le deck"));
            chosen.push(card);
        }
        self.deck(&chosen)
    }

    /// `tokens` sur le dessus de la pile `color`, dans l'ordre : ce que la mise en place en tire
    /// d'abord (ennemis errants et défenseurs des tuiles déjà posées), puis la suite.
    pub fn enemies(mut self, color: EnemyColor, tokens: &[EnemyToken]) -> Self {
        for (position, &token) in tokens.iter().enumerate() {
            let order = self.enemy_order(color);
            assert!(
                order.contains(&token),
                "{token:?} hors de la pile {color:?}"
            );
            let current = order_at(&order, position, "pile d'ennemis");
            self.swap_everywhere(|event| swap_enemy(event, current, token));
        }
        self
    }

    /// `tokens`, de la couleur `color`, les prochains tirés de leur pile après la mise en place.
    pub fn next_enemies(mut self, color: EnemyColor, tokens: &[EnemyToken]) -> Self {
        let first = self.first_undrawn_enemy(color);
        for (offset, &token) in tokens.iter().enumerate() {
            let order = self.enemy_order(color);
            assert!(
                order.contains(&token),
                "{token:?} hors de la pile {color:?}"
            );
            let current = order_at(&order, first + offset, "pile d'ennemis");
            self.swap_everywhere(|event| swap_enemy(event, current, token));
        }
        self
    }

    /// `cards` dans l'offre de départ tirée de `deck`, en tête : les dernières distribuées, qui
    /// restent le plus longtemps dans l'offre (la dernière place repart sous la pioche à chaque
    /// nouvelle manche, MKUE p. 4, 2.d).
    pub fn offer(mut self, deck: CommonDeck, cards: &[CommonCard]) -> Self {
        let dealt = self.first_undealt(deck);
        assert!(
            cards.len() <= dealt,
            "l'offre de {deck:?} n'a que {dealt} cartes"
        );
        for (rank, &card) in cards.iter().enumerate() {
            self.put_common(deck, dealt - 1 - rank, card);
        }
        self
    }

    /// `cards` les prochaines tirées de `deck` après la mise en place (offre complétée).
    pub fn next_cards(mut self, deck: CommonDeck, cards: &[CommonCard]) -> Self {
        let first = self.first_undealt(deck);
        for (offset, &card) in cards.iter().enumerate() {
            self.put_common(deck, first + offset, card);
        }
        self
    }

    /// `skills` sur le dessus de la pile de compétences du héros.
    pub fn skills(mut self, skills: &[Skill]) -> Self {
        for (position, &skill) in skills.iter().enumerate() {
            let order = self.skill_order();
            assert!(order.contains(&skill), "{skill:?} hors de la pile du héros");
            let current = order_at(&order, position, "pile de compétences");
            self.swap_everywhere(|event| swap_skill(event, current, skill));
        }
        self
    }

    /// Spec 061 : `cards` en tête du deck du joueur fantôme, dans l'ordre, le premier exemplaire pas
    /// encore placé de chacune.
    pub fn dummy_deck(mut self, cards: &[DeedCard]) -> Self {
        let mut chosen: Vec<CardId> = Vec::new();
        for (position, &wanted) in cards.iter().enumerate() {
            let order = self.dummy_order();
            let card = order
                .iter()
                .copied()
                .find(|card| card.card() == wanted && !chosen.contains(card))
                .unwrap_or_else(|| {
                    panic!("plus d'exemplaire de {wanted:?} dans le deck du fantôme")
                });
            chosen.push(card);
            let current = order_at(&order, position, "deck du fantôme");
            self.swap_everywhere(|event| swap_dummy_card(event, current, card));
        }
        self
    }

    /// Spec 069 : `tiles` en tête des tuiles royaume de la pioche de tuiles, dans l'ordre (les
    /// tuiles de campagne gardent l'ordre de la graine). Une tuile royaume hors de la pioche prend
    /// la place d'une tuile tirée de même sorte (cité ou non), comme si le tirage des tuiles royaume
    /// au hasard l'avait prise.
    pub fn core_tiles(self, tiles: &[TileId]) -> Self {
        self.tiles(TileKind::Core, tiles)
    }

    /// Spec 069 : `tiles` en tête des tuiles de campagne, dans l'ordre : les deux premières sont
    /// celles que la mise en place révèle.
    pub fn countryside_tiles(self, tiles: &[TileId]) -> Self {
        self.tiles(TileKind::Countryside, tiles)
    }

    fn tiles(mut self, kind: TileKind, tiles: &[TileId]) -> Self {
        for (offset, &tile) in tiles.iter().enumerate() {
            assert_eq!(tile.kind(), kind, "{tile:?} n'est pas de la sorte {kind:?}");
            let order = self.tile_order();
            let first = order
                .iter()
                .position(|tile| tile.kind() == kind)
                .expect("la pioche a des tuiles de cette sorte");
            let position = first + offset;
            if !order.contains(&tile) {
                // La tuile tirée qu'elle remplace : la première de même sorte, mise à sa place.
                let same = order[position..]
                    .iter()
                    .copied()
                    .find(|drawn| drawn.is_city() == tile.is_city())
                    .unwrap_or_else(|| panic!("aucune tuile royaume à remplacer par {tile:?}"));
                let current = order_at(&order, position, "pioche de tuiles");
                self.swap_everywhere(|event| swap_tile(event, current, same));
                self.swap_everywhere(|event| swap_tile(event, same, tile));
            } else {
                let current = order_at(&order, position, "pioche de tuiles");
                self.swap_everywhere(|event| swap_tile(event, current, tile));
            }
        }
        self
    }

    /// Spec 069 : un exemplaire pas encore tiré de chaque ennemi d'`enemies`, les prochains tirés
    /// de la pile `color` après la mise en place, dans l'ordre.
    pub fn next_enemy_kinds(self, color: EnemyColor, enemies: &[Enemy]) -> Self {
        let drawn = self.drawn_enemies();
        let order = self.enemy_order(color);
        let mut chosen: Vec<EnemyToken> = Vec::new();
        for &enemy in enemies {
            let token = order
                .iter()
                .copied()
                .find(|token| {
                    token.enemy() == enemy && !drawn.contains(token) && !chosen.contains(token)
                })
                .unwrap_or_else(|| panic!("plus d'exemplaire de {enemy:?} dans la pile"));
            chosen.push(token);
        }
        self.next_enemies(color, &chosen)
    }

    /// Épisode posé (spec 069) : `card` distribuée dans l'offre de `deck`, puis gagnée sur le
    /// dessus du deck du héros (comme une récompense), sans être piochée. La carte est d'abord mise
    /// la prochaine de la pioche commune.
    pub fn gain_on_deck(mut self, deck: CommonDeck, card: CommonCard) -> Self {
        self = self.next_cards(deck, &[card]);
        let offer = match deck {
            CommonDeck::AdvancedActions => Offer::AdvancedActions,
            CommonDeck::Spells => Offer::Spells,
            CommonDeck::Artifacts => panic!("pas d'offre d'artefacts"),
            CommonDeck::RegularUnits | CommonDeck::EliteUnits => Offer::Units,
        };
        self.events.push(Event::OfferCardDealt { offer, card });
        self.events.push(Event::OfferCardGained {
            player: PlayerId::SOLO,
            offer,
            card,
            to: CardDestination::DeckTop,
        });
        self
    }

    /// Épisode posé (spec 069) : `unit` distribuée dans l'offre d'unités, puis recrutée.
    pub fn recruit(mut self, unit: UnitCard) -> Self {
        let card = CommonCard::Unit(unit);
        let deck = match unit.unit.definition().rank {
            UnitRank::Regular => CommonDeck::RegularUnits,
            UnitRank::Elite => CommonDeck::EliteUnits,
        };
        self = self.next_cards(deck, &[card]);
        self.events.push(Event::OfferCardDealt {
            offer: Offer::Units,
            card,
        });
        self.events.push(Event::UnitRecruited {
            player: PlayerId::SOLO,
            unit,
            cost: 0,
        });
        self
    }

    /// Épisode posé : des événements de la partie, posés après la mise en place.
    pub fn pose(mut self, events: impl IntoIterator<Item = Event>) -> Self {
        self.events.extend(events);
        self
    }

    /// Les tuiles de la pioche, dans l'ordre (mise en place comprise).
    pub fn tile_deck(&self) -> Vec<TileId> {
        self.tile_order()
    }

    /// La figurine posée sur `hex` au lieu de sa case de départ.
    pub fn figure(mut self, hex: Hex) -> Self {
        for event in &mut self.events {
            if let Event::HeroFigurePlaced { hex: placed, .. } = event {
                *placed = hex;
            }
        }
        self
    }

    /// Les dés de la Source à la mise en place (autant que le vrai lancer).
    pub fn source(mut self, dice: &[ManaColor]) -> Self {
        for event in &mut self.events {
            if let Event::SourceRolled { dice: rolled } = event {
                assert_eq!(rolled.len(), dice.len(), "nombre de dés de la Source");
                *rolled = dice.to_vec();
            }
        }
        self
    }

    /// Épisode posé (spec 059, Q9) : `card` gagnée de l'offre d'actions avancées comme à la
    /// récompense de niveau (sur le dessus du deck, l'offre complétée), puis piochée en main.
    pub fn gain_advanced_action(mut self, card: CardId) -> Self {
        let offer = Offer::AdvancedActions;
        let next = self.first_undealt(CommonDeck::AdvancedActions);
        let dealt = self
            .common_order(CommonDeck::AdvancedActions)
            .get(next)
            .copied();
        let player = PlayerId::SOLO;
        self.events.push(Event::OfferCardGained {
            player,
            offer,
            card: CommonCard::Deed(card),
            to: CardDestination::DeckTop,
        });
        self.events
            .extend(dealt.map(|card| Event::OfferCardDealt { offer, card }));
        self.events.push(Event::CardDrawn { player, card });
        self
    }

    fn drawn_enemies(&self) -> Vec<EnemyToken> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Event::EnemyTokenPlaced { token, .. } | Event::DefenderPlaced { token, .. } => {
                    Some(*token)
                }
                _ => None,
            })
            .collect()
    }

    fn swap_everywhere(&mut self, mut swap: impl FnMut(&mut Event)) {
        for event in &mut self.events {
            swap(event);
        }
    }

    fn put_common(&mut self, deck: CommonDeck, position: usize, card: CommonCard) {
        let order = self.common_order(deck);
        assert!(order.contains(&card), "{card:?} hors de la pile {deck:?}");
        let current = order_at(&order, position, "pile commune");
        self.swap_everywhere(|event| swap_common(event, deck, current, card));
    }

    fn hero_order(&self) -> Vec<CardId> {
        self.find(|event| match event {
            Event::DeckShuffled { order, .. } => Some(order.clone()),
            _ => None,
        })
    }

    fn tile_order(&self) -> Vec<TileId> {
        self.find(|event| match event {
            Event::TileDeckFormed { order } => Some(order.clone()),
            _ => None,
        })
    }

    fn dummy_order(&self) -> Vec<CardId> {
        self.find(|event| match event {
            Event::DummySetUp { deck, .. } => Some(deck.clone()),
            _ => None,
        })
    }

    fn enemy_order(&self, wanted: EnemyColor) -> Vec<EnemyToken> {
        self.find(|event| match event {
            Event::EnemyPileShuffled { color, order } if *color == wanted => Some(order.clone()),
            _ => None,
        })
    }

    fn common_order(&self, wanted: CommonDeck) -> Vec<CommonCard> {
        self.find(|event| match event {
            Event::CommonDeckShuffled { deck, order } if *deck == wanted => Some(order.clone()),
            _ => None,
        })
    }

    fn skill_order(&self) -> Vec<Skill> {
        self.find(|event| match event {
            Event::SkillPileShuffled { order, .. } => Some(order.clone()),
            _ => None,
        })
    }

    fn find<T>(&self, pick: impl Fn(&Event) -> Option<T>) -> T {
        self.events
            .iter()
            .find_map(pick)
            .expect("la mise en place mélange cette pile")
    }

    /// La position du premier jeton de la pile qu'aucun événement de la mise en place n'a posé.
    fn first_undrawn_enemy(&self, color: EnemyColor) -> usize {
        let order = self.enemy_order(color);
        let drawn: Vec<EnemyToken> = self
            .events
            .iter()
            .filter_map(|event| match event {
                Event::EnemyTokenPlaced { token, .. } | Event::DefenderPlaced { token, .. } => {
                    Some(*token)
                }
                _ => None,
            })
            .collect();
        order
            .iter()
            .position(|token| !drawn.contains(token))
            .unwrap_or(order.len())
    }

    /// Le nombre de cartes de la pile que la mise en place a déjà distribuées.
    fn first_undealt(&self, deck: CommonDeck) -> usize {
        let order = self.common_order(deck);
        let dealt: Vec<CommonCard> = self
            .events
            .iter()
            .filter_map(|event| match event {
                Event::OfferCardDealt { card, .. } => Some(*card),
                _ => None,
            })
            .collect();
        order
            .iter()
            .position(|card| !dealt.contains(card))
            .unwrap_or(order.len())
    }
}

fn order_at<T: Copy>(order: &[T], position: usize, pile: &str) -> T {
    *order
        .get(position)
        .unwrap_or_else(|| panic!("{pile} trop courte pour la position {position}"))
}

fn swap<T: PartialEq + Copy>(item: &mut T, a: T, b: T) {
    if *item == a {
        *item = b;
    } else if *item == b {
        *item = a;
    }
}

fn swap_hero_card(event: &mut Event, a: CardId, b: CardId) {
    match event {
        Event::DeckShuffled { order, .. } => order.iter_mut().for_each(|card| swap(card, a, b)),
        Event::CardDrawn { card, .. } => swap(card, a, b),
        _ => {}
    }
}

fn swap_tile(event: &mut Event, a: TileId, b: TileId) {
    match event {
        Event::TileDeckFormed { order } => order.iter_mut().for_each(|tile| swap(tile, a, b)),
        Event::TileRevealed { tile, .. } => swap(tile, a, b),
        _ => {}
    }
}

fn swap_dummy_card(event: &mut Event, a: CardId, b: CardId) {
    if let Event::DummySetUp { deck, .. } = event {
        deck.iter_mut().for_each(|card| swap(card, a, b));
    }
}

fn swap_enemy(event: &mut Event, a: EnemyToken, b: EnemyToken) {
    match event {
        Event::EnemyPileShuffled { order, .. } => {
            order.iter_mut().for_each(|token| swap(token, a, b))
        }
        Event::EnemyTokenPlaced { token, .. } | Event::DefenderPlaced { token, .. } => {
            swap(token, a, b)
        }
        _ => {}
    }
}

fn swap_common(event: &mut Event, wanted: CommonDeck, a: CommonCard, b: CommonCard) {
    match event {
        Event::CommonDeckShuffled { deck, order } if *deck == wanted => {
            order.iter_mut().for_each(|card| swap(card, a, b))
        }
        Event::OfferCardDealt { card, .. } => swap(card, a, b),
        _ => {}
    }
}

fn swap_skill(event: &mut Event, a: Skill, b: Skill) {
    if let Event::SkillPileShuffled { order, .. } = event {
        order.iter_mut().for_each(|skill| swap(skill, a, b));
    }
}

/// Les événements que l'arrangement sait garder cohérents. Un nouvel événement de mise en place
/// qui porte une carte ou un jeton doit être ajouté aux échanges ci-dessus, puis ici.
fn known_setup_event(event: &Event) -> bool {
    matches!(
        event,
        Event::GameStarted { .. }
            | Event::TilePlaced { .. }
            | Event::TileDeckFormed { .. }
            | Event::TileRevealed { .. }
            | Event::HeroFigurePlaced { .. }
            | Event::DeckShuffled { .. }
            | Event::CardDrawn { .. }
            | Event::SourceRolled { .. }
            | Event::EnemyPileShuffled { .. }
            | Event::RuinPileShuffled { .. }
            | Event::RuinTokenPlaced { .. }
            | Event::EnemyTokenPlaced { .. }
            | Event::DefenderPlaced { .. }
            | Event::CommonDeckShuffled { .. }
            | Event::OfferCardDealt { .. }
            | Event::SkillPileShuffled { .. }
            | Event::DummySetUp { .. }
    )
}
