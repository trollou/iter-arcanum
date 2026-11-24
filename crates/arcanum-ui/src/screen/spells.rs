//! Sorts (spec 054) : boutons d'effet d'un sort, bandeau du choix du mana et de l'ennemi visé,
//! bouton « Lancer » de l'offre de sorts.

use arcanum_core::{CastTargets, CommonCard, DeedCard};

use super::*;

/// Le nom du sort, celui de son effet renforcé si `strong` (« Tempête de feu »).
pub(super) fn spell_name(card: CardId, strong: bool, catalog: &Catalog) -> String {
    match card.card() {
        DeedCard::Spell(spell) if strong => catalog.text(spell.strong_name_key()),
        _ => card_name(card, catalog),
    }
}

/// Le texte d'une option de sort : son effet, suivi de ce qu'elle pose sur les ennemis
/// (« tous les ennemis : Armure −2 (min. 1) ») quand le sort a plusieurs options ; le texte écrit
/// de la carte pour une option seule qui pose des modificateurs ou ne se dit pas en points. Dans
/// `combat`, Déluge de flammes dit son total avec les ennemis affrontés (issue privée 245).
pub(super) fn spell_option_label(
    card: CardId,
    strong: bool,
    option: u8,
    combat: Option<&Combat>,
    catalog: &Catalog,
) -> String {
    let Some(definition) = card.card().definition() else {
        return String::new();
    };
    let effect = if strong {
        definition.strong
    } else {
        definition.basic
    };
    let BasicEffect::Options { options, .. } = effect else {
        return String::new();
    };
    let Some(&points) = options.get(usize::from(option)) else {
        return String::new();
    };
    let points = against_combat(points, combat);
    let kinds = effect.option_modifiers(option);
    if options.len() == 1 && (!kinds.is_empty() || !has_points(points)) {
        let written = card
            .card()
            .unsupported_text_keys()
            .map(|(basic, strong_key)| if strong { strong_key } else { basic });
        if let Some(key) = written {
            return catalog.text(key);
        }
    }
    let mut parts = Vec::new();
    if points != Effect::Nothing {
        parts.push(effect_text(points, catalog));
    }
    parts.extend(
        kinds
            .iter()
            .map(|kind| super::play_area::modifier_text(*kind, catalog)),
    );
    parts.join(", ")
}

/// L'effet obtenu contre les ennemis affrontés de `combat` (Déluge de flammes, spec 054, Q4).
fn against_combat(effect: Effect, combat: Option<&Combat>) -> Effect {
    combat.map_or(effect, |combat| {
        effect.against_enemies(combat.enemies_faced())
    })
}

/// Le combat en cours de `view`, pas encore terminé.
fn ongoing(view: Option<&PlayerView>) -> Option<&Combat> {
    view.and_then(|view| view.combat.as_ref())
        .filter(|combat| !combat.ended)
}

/// Vrai pour un effet que son texte calculé dit en entier.
fn has_points(effect: Effect) -> bool {
    !matches!(
        effect,
        Effect::Nothing
            | Effect::DestroyEnemy
            | Effect::AttackPerEnemy { .. }
            | Effect::BlockPerEnemy { .. }
    )
}

/// Les sorts lancés que vise `choice` parmi les actions légales, avec leur index.
fn matching(legal_actions: &[Action], choice: CastChoice) -> Vec<(usize, CardPlay)> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::PlayCard {
                card,
                play:
                    play @ CardPlay::Cast {
                        option,
                        black,
                        from_offer,
                        ..
                    },
            } if *card == choice.card
                && *from_offer == choice.from_offer
                && choice.option.is_none_or(|(strong, wanted)| {
                    strong == black.is_some() && wanted == *option
                }) =>
            {
                Some((index, *play))
            }
            _ => None,
        })
        .collect()
}

/// Vrai si des lancers légaux de `choice` visent des unités ou des ennemis à cocher (Renaissance,
/// Flux d'énergie, Ailes de la nuit ; issue privée 258).
fn ticks_targets(legal_actions: &[Action], choice: CastChoice) -> bool {
    matching(legal_actions, choice).iter().any(|(_, play)| {
        matches!(play, CardPlay::Cast { targets, .. }
            if !targets.units.is_empty() || !targets.enemies.is_empty())
    })
}

/// Vrai si `a` et `b` ont les mêmes éléments, dans n'importe quel ordre.
fn same_picks<T: Copy + PartialEq>(a: arcanum_core::Picks<T>, b: arcanum_core::Picks<T>) -> bool {
    a.len() == b.len() && a.iter().all(|item| b.contains(item))
}

/// Vrai si le lancer `play` vise exactement les cibles cochées de `picks`.
fn aims_at(play: CardPlay, picks: CastPicks) -> bool {
    matches!(play, CardPlay::Cast { targets, .. }
        if same_picks(targets.units, picks.units) && same_picks(targets.enemies, picks.enemies))
}

/// Les cibles qu'on peut cocher pour le sort en cours : les unités et les ennemis d'au moins un
/// lancer légal (issue privée 258) ; rien hors du choix d'un sort.
pub(super) fn tickable_targets(
    legal_actions: &[Action],
    mode: &ScreenMode,
) -> Option<(CastPicks, Vec<UnitCard>, Vec<EnemyToken>)> {
    let ScreenMode::Casting(choice) = *mode else {
        return None;
    };
    let mut units = Vec::new();
    let mut enemies = Vec::new();
    for (_, play) in matching(legal_actions, choice) {
        if let CardPlay::Cast { targets, .. } = play {
            units.extend(
                targets
                    .units
                    .iter()
                    .filter(|unit| !units.contains(unit))
                    .collect::<Vec<_>>(),
            );
            enemies.extend(
                targets
                    .enemies
                    .iter()
                    .filter(|enemy| !enemies.contains(enemy))
                    .collect::<Vec<_>>(),
            );
        }
    }
    Some((choice.picks, units, enemies))
}

/// La case « cible » d'un ennemi d'Ailes de la nuit sur l'écran de combat, par ennemi qu'un lancer
/// légal peut viser (issue privée 258) ; vide hors du choix d'un sort qui vise des ennemis.
pub fn cast_enemy_toggles(
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Vec<(EnemyToken, DiscardToggle)> {
    let Some((picks, _, enemies)) = tickable_targets(legal_actions, mode) else {
        return Vec::new();
    };
    enemies
        .into_iter()
        .map(|enemy| {
            (
                enemy,
                DiscardToggle {
                    label: catalog.text("combat.target"),
                    checked: picks.enemies.contains(enemy),
                    command: Command::ToggleCastEnemy(enemy),
                },
            )
        })
        .collect()
}

/// Un bouton par façon distincte de lancer le sort : l'option si le choix ne la fixe pas, l'ennemi
/// visé, puis les manas payés (couleur du sort, noir, mana en plus de l'option) : « Swordsmen ·
/// Cristal vert (1 sur 3) ». Avec `ticked`, pour un sort dont les cibles se cochent (issue privée 258),
/// seulement les lancers qui visent les cibles cochées, sans les nommer.
fn cast_buttons(
    view: &PlayerView,
    legal_actions: &[Action],
    choice: CastChoice,
    ticked: bool,
    catalog: &Catalog,
) -> Vec<Button> {
    let Some(definition) = choice.card.card().definition() else {
        return Vec::new();
    };
    let ticked = ticked && ticks_targets(legal_actions, choice);
    let mut buttons: Vec<Button> = Vec::new();
    for (index, play) in matching(legal_actions, choice) {
        if ticked && !aims_at(play, choice.picks) {
            continue;
        }
        let CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            ..
        } = play
        else {
            continue;
        };
        let strong = black.is_some();
        let mut parts = Vec::new();
        if choice.option.is_none() {
            let label =
                spell_option_label(choice.card, strong, option, ongoing(Some(view)), catalog);
            parts.push(catalog.format(
                if strong { "play.strong" } else { "play.basic" },
                &[("effect", &label)],
            ));
        }
        if let Some(enemy) = enemy {
            parts.push(catalog.text(enemy.enemy().label_key()));
        }
        // Issue privée 257 : l'arrivée d'un déplacement par effet se clique ensuite sur le plateau.
        let mut command = Command::Play(index);
        if let CardPlay::Cast { targets, .. } = play {
            if targets.destination.is_some() {
                command = Command::ChooseDestination(DestinationChoice {
                    choice,
                    payment: without_destination(play),
                });
            }
            let (units, enemies) = if ticked {
                Default::default()
            } else {
                (targets.units, targets.enemies)
            };
            parts.extend(target_parts(
                view,
                CastTargets {
                    destination: None,
                    units,
                    enemies,
                    ..targets
                },
                catalog,
            ));
        }
        parts.push(payment_label(
            view,
            mana,
            ManaColor::Basic(definition.color),
            catalog,
        ));
        if let Some(black) = black {
            parts.push(payment_label(view, black, ManaColor::Black, catalog));
        }
        let effect = if strong {
            definition.strong
        } else {
            definition.basic
        };
        if let (Some(extra), Some(color)) = (extra_mana, effect.option_mana(option)) {
            parts.push(payment_label(view, extra, ManaColor::Basic(color), catalog));
        }
        let label = parts.join(" · ");
        // Deux dés de la même couleur paient pareil, et les arrivées viennent après : un bouton
        // suffit.
        if !buttons.iter().any(|button| button.label == label) {
            buttons.push(Button {
                label,
                command,
                shortcut: None,
            });
        }
    }
    buttons
}

/// Le lancer `play` sans son arrivée : ce que fixe le choix du paiement (issue privée 257).
fn without_destination(mut play: CardPlay) -> CardPlay {
    if let CardPlay::Cast { targets, .. } = &mut play {
        targets.destination = None;
    }
    play
}

/// Le déplacement par effet dont l'arrivée se choisit maintenant sur le plateau : le paiement
/// choisi, ou le seul possible (spec 057, issue privée 257).
fn destination_stage(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<DestinationChoice> {
    match *mode {
        ScreenMode::ChoosingDestination(stage) => Some(stage),
        ScreenMode::Casting(choice) => {
            match cast_buttons(view, legal_actions, choice, false, catalog).as_slice() {
                [
                    Button {
                        command: Command::ChooseDestination(stage),
                        ..
                    },
                ] => Some(*stage),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Les arrivées légales du déplacement par effet payé, chacune avec l'action qui y mène (spec
/// 057, C3, issue privée 257).
pub(super) fn cast_destinations(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Vec<CastDestination> {
    let Some(stage) = destination_stage(view, legal_actions, mode, catalog) else {
        return Vec::new();
    };
    matching(legal_actions, stage.choice)
        .into_iter()
        .filter(|(_, play)| without_destination(*play) == stage.payment)
        .filter_map(|(index, play)| match play {
            CardPlay::Cast { targets, .. } => targets.destination.map(|hex| CastDestination {
                hex,
                command: Command::Play(index),
            }),
            _ => None,
        })
        .collect()
}

/// Les choix d'un sort de la spec 057, dans le bandeau : l'arrivée (site ou terrain, et la case),
/// les unités visées, les ennemis d'Ailes de la nuit et leur coût en Mouvement, le dé de Puits de
/// mana.
fn target_parts(view: &PlayerView, targets: CastTargets, catalog: &Catalog) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(hex) = targets.destination {
        let place = view.map.space(hex).map_or_else(String::new, |space| {
            catalog.text(match space.feature {
                Some(feature) => crate::board::feature_key(feature),
                None => crate::board::terrain_key(space.terrain),
            })
        });
        parts.push(catalog.format(
            "destination.space",
            &[
                ("place", &place),
                ("q", &hex.q.to_string()),
                ("r", &hex.r.to_string()),
            ],
        ));
    }
    if !targets.units.is_empty() {
        let names: Vec<String> = targets
            .units
            .iter()
            .map(|unit| catalog.text(unit.unit.label_key()))
            .collect();
        parts.push(names.join(", "));
    }
    if !targets.enemies.is_empty() {
        let names: Vec<String> = targets
            .enemies
            .iter()
            .map(|enemy| catalog.text(enemy.enemy().label_key()))
            .collect();
        parts.push(names.join(", "));
        parts.push(catalog.format(
            "wings_of_night.cost",
            &[(
                "points",
                &arcanum_core::wings_of_night_cost(targets.enemies.len()).to_string(),
            )],
        ));
    }
    if let Some(&color) = targets
        .die
        .and_then(|die| view.source.get(usize::from(die)))
    {
        parts.push(catalog.format("mana_claim.die", &[("color", &mana_name(color, catalog))]));
    }
    parts
}

/// Ce que fait le bouton d'un sort : rien sans action légale, l'action seule s'il n'y a qu'une
/// façon de le lancer, sinon le bandeau du choix.
pub(super) fn cast_command(
    view: &PlayerView,
    legal_actions: &[Action],
    choice: CastChoice,
    catalog: &Catalog,
) -> Option<Command> {
    match cast_buttons(view, legal_actions, choice, false, catalog).as_slice() {
        [] => None,
        [only] if matches!(only.command, Command::Play(_)) => Some(only.command),
        _ => Some(Command::ChooseCast(choice)),
    }
}

/// Le bandeau du choix d'un sort lancé : le mana, le noir, le mana en plus et l'ennemi visé ;
/// pour un déplacement par effet payé, l'invitation à cliquer l'arrivée (issue privée 257).
pub(super) fn cast_prompt(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<ManaPrompt> {
    let cancel = Button {
        label: catalog.text("mana_choice.cancel"),
        command: Command::CancelChoice,
        shortcut: shortcut_label(Shortcut::Cancel, catalog),
    };
    if let Some(stage) = destination_stage(view, legal_actions, mode, catalog) {
        if cast_destinations(view, legal_actions, mode, catalog).is_empty() {
            return None;
        }
        return Some(ManaPrompt {
            text: catalog.format(
                "spell.destination_prompt",
                &[(
                    "card",
                    &spell_name(stage.choice.card, stage.payment.is_strong(), catalog),
                )],
            ),
            payments: Vec::new(),
            cancel,
        });
    }
    let ScreenMode::Casting(choice) = *mode else {
        return None;
    };
    let payments = cast_buttons(view, legal_actions, choice, true, catalog);
    // Issue privée 258 : le bandeau reste pendant qu'on coche, même sans lancer pour ces cibles.
    let ticked = ticks_targets(legal_actions, choice);
    if payments.is_empty() && !ticked {
        return None;
    }
    let strong = matches!(choice.option, Some((true, _)));
    let mut text = catalog.format(
        "spell.prompt",
        &[("card", &spell_name(choice.card, strong, catalog))],
    );
    if ticked && !choice.picks.enemies.is_empty() {
        let cost = arcanum_core::wings_of_night_cost(choice.picks.enemies.len());
        text = format!(
            "{text} {}",
            catalog.format("wings_of_night.cost", &[("points", &cost.to_string())])
        );
    }
    Some(ManaPrompt {
        text,
        payments,
        cancel,
    })
}

/// Les boutons d'effet d'un sort : chaque option de base et renforcée, avec la pastille du mana en
/// plus de l'option, sinon de la couleur du sort (base) ou du noir (renforcé). Avec les actions
/// légales, la commande qui le lance, ou la raison du grisé (spec 054, Interface).
pub(super) fn spell_effects(
    card: CardId,
    view: Option<&PlayerView>,
    legal_actions: Option<&[Action]>,
    catalog: &Catalog,
) -> (Vec<EffectButton>, Vec<EffectButton>) {
    let Some(definition) = card.card().definition() else {
        return (Vec::new(), Vec::new());
    };
    let buttons = |strong: bool| -> Vec<EffectButton> {
        let effect = if strong {
            definition.strong
        } else {
            definition.basic
        };
        let BasicEffect::Options { options, .. } = effect else {
            return Vec::new();
        };
        (0u8..)
            .zip(options)
            .map(|(option, &points)| {
                let combat = ongoing(view);
                let label = spell_option_label(card, strong, option, combat, catalog);
                let points = against_combat(points, combat);
                let kinds = effect.option_modifiers(option);
                let face = if kinds.is_empty() && has_points(points) {
                    effects_face(points, catalog)
                } else {
                    EffectFace::Text(label.clone())
                };
                let color = effect
                    .option_mana(option)
                    .map(ManaColor::Basic)
                    .unwrap_or(if strong {
                        ManaColor::Black
                    } else {
                        ManaColor::Basic(definition.color)
                    });
                let mut tooltip = catalog.format(
                    if strong { "play.strong" } else { "play.basic" },
                    &[("effect", &label)],
                );
                let mut command = None;
                if let (Some(view), Some(legal_actions)) = (view, legal_actions) {
                    let choice = CastChoice {
                        card,
                        option: Some((strong, option)),
                        from_offer: false,
                        picks: CastPicks::default(),
                    };
                    command = cast_command(view, legal_actions, choice, catalog);
                    // MKUE p. 5, 2.a : pas de mana noir de jour.
                    let by_day = view.combat_time_of_day == TimeOfDay::Day;
                    let reason = if command.is_some() {
                        None
                    } else if strong && by_day {
                        Some(catalog.text("spell.strong_needs_black"))
                    } else if view.is_option_playable_now(points.resolved(), kinds) {
                        Some(missing_mana(view, definition.color, catalog))
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        tooltip = format!("{tooltip}\n{reason}");
                    }
                }
                EffectButton {
                    face,
                    mark: view.and_then(|view| halved(points, view, catalog)),
                    color: Some(color),
                    command,
                    tooltip,
                }
            })
            .collect()
    };
    (buttons(false), buttons(true))
}

fn effects_face(effect: Effect, catalog: &Catalog) -> EffectFace {
    super::effects::effect_face(effect.resolved(), catalog)
}

/// « ÷2 » quand l'effet compte à moitié contre la cible du combat (spec 032).
fn halved(effect: Effect, view: &PlayerView, catalog: &Catalog) -> Option<String> {
    view.combat
        .as_ref()
        .filter(|combat| !combat.ended)
        .and_then(|combat| combat.efficiency(effect.resolved()))
        .filter(|efficiency| *efficiency == Efficiency::Halved)
        .map(|_| catalog.text("combat.mark.halved"))
}

/// « Lancer » sur un sort de l'offre que Don pour la magie permet de lancer (spec 054, C27).
pub(super) fn cast_offer_button(
    card: CommonCard,
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
    let CommonCard::Deed(deed) = card else {
        return None;
    };
    let choice = CastChoice {
        card: deed,
        option: None,
        from_offer: true,
        picks: CastPicks::default(),
    };
    cast_command(view, legal_actions, choice, catalog).map(|command| Button {
        label: catalog.text("offers.cast"),
        command,
        shortcut: None,
    })
}
