//! Artefacts (spec 055) : boutons Jouer et Détruire d'un artefact de la main, bannières (choix de
//! l'unité, bannière portée, capacités).

use arcanum_core::{ArtifactChoices, OwnedUnit, Picks};

use super::*;

/// Les actions légales qui jouent `card` pour l'option `option`, de base ou renforcée (détruite),
/// avec leur index et leurs choix.
pub(super) fn artifact_plays(
    legal_actions: &[Action],
    card: CardId,
    strong: bool,
    option: u8,
) -> Vec<(usize, ArtifactChoices)> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::PlayCard {
                card: played,
                play: CardPlay::Artifact { option: o, choices },
            } if *played == card && !strong && *o == option => Some((index, *choices)),
            Action::PlayCard {
                card: played,
                play: CardPlay::ThrowAway { option: o, choices },
            } if *played == card && strong && *o == option => Some((index, *choices)),
            _ => None,
        })
        .collect()
}

/// Ce que fait le bouton d'un effet d'artefact : la bannière à assigner, l'action seule quand elle
/// n'a pas de choix, le choix de la carte d'offre (Livre de sagesse), sinon le bandeau des choix.
fn artifact_command(
    card: CardId,
    strong: bool,
    option: u8,
    view: &PlayerView,
    legal_actions: &[Action],
) -> Option<Command> {
    if !strong
        && legal_actions.iter().any(
            |action| matches!(action, Action::AssignBanner { card: banner, .. } if *banner == card),
        )
    {
        return Some(Command::AssignBanner(card));
    }
    let plays = artifact_plays(legal_actions, card, strong, option);
    match plays.as_slice() {
        [] => None,
        [(index, choices)] if *choices == ArtifactChoices::default() => Some(Command::Play(*index)),
        _ if plays.iter().any(|(_, choices)| choices.gain.is_some()) => Some(gain_command(
            legal_actions,
            &view.source,
            GainChoice {
                card,
                option,
                strong,
                discard: None,
                gain: None,
            },
        )),
        _ => Some(Command::ChooseArtifact(ArtifactPlay {
            card,
            strong,
            option,
            picks: ArtifactChoices::default(),
        })),
    }
}

/// Les boutons d'effet d'un artefact : son texte écrit, de base (« Jouer : … » ou « Assigner à une
/// unité ») et renforcé (« Détruire : … »), sans pastille de mana ; la commande avec les actions
/// légales.
pub(super) fn artifact_effects(
    card: CardId,
    view: Option<&PlayerView>,
    legal_actions: Option<&[Action]>,
    catalog: &Catalog,
) -> (Vec<EffectButton>, Vec<EffectButton>) {
    let Some((basic_key, strong_key)) = card.card().unsupported_text_keys() else {
        return (Vec::new(), Vec::new());
    };
    let button = |strong: bool| {
        let text = catalog.text(if strong { strong_key } else { basic_key });
        let banner = !strong && option_effect_of(card, 0, false) == Some(Effect::BannerAssigned);
        let tooltip = if strong {
            catalog.format("play.throw_away", &[("effect", &text)])
        } else if banner {
            catalog.text("play.assign_banner")
        } else {
            catalog.format("play.basic", &[("effect", &text)])
        };
        // Le texte de la carte est au-dessus du bouton : le bouton dit seulement ce qu'il fait.
        let face = catalog.text(if strong {
            "artifact.throw_away"
        } else if banner {
            "banner.assign"
        } else {
            "artifact.play"
        });
        EffectButton {
            face: EffectFace::Text(face),
            mark: None,
            color: None,
            command: view
                .zip(legal_actions)
                .and_then(|(view, legal)| artifact_command(card, strong, 0, view, legal)),
            tooltip,
        }
    };
    (vec![button(false)], vec![button(true)])
}

/// La bannière portée par l'unité : son nom, et en infobulle « Bannière : … », retournée ou non.
pub(super) fn banner_tag(owned: &OwnedUnit, catalog: &Catalog) -> Option<BannerTag> {
    let card = owned.banner?;
    let name = card_name(card, catalog);
    let mut tooltip = catalog.format("banner.on_unit", &[("card", &name)]);
    if owned.banner_flipped {
        tooltip = format!("{tooltip}\n{}", catalog.text("banner.flipped"));
    }
    Some(BannerTag {
        name,
        tooltip,
        flipped: owned.banner_flipped,
    })
}

/// Les capacités de la bannière de l'unité qu'une action légale permet : une attaque à annuler par
/// ennemi (Terreur), « Rendre disponible » (Courage).
pub(super) fn banner_uses(
    unit: UnitCard,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Vec<Button> {
    legal_actions
        .iter()
        .enumerate()
        .filter_map(|(index, action)| match action {
            Action::UseBanner {
                unit: carrier,
                enemy,
            } if *carrier == unit => Some(Button {
                label: match enemy {
                    Some(enemy) => catalog.format(
                        "banner.use_fear_enemy",
                        &[("enemy", &catalog.text(enemy.enemy().label_key()))],
                    ),
                    None => catalog.text("banner.use_courage"),
                },
                command: Command::Play(index),
                shortcut: None,
            }),
            _ => None,
        })
        .collect()
}

/// « Assigner » sur l'unité, pendant le choix de l'unité de la bannière `card`.
pub(super) fn assign_button(
    card: CardId,
    unit: UnitCard,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
    legal_actions
        .iter()
        .position(|action| *action == Action::AssignBanner { card, unit })
        .map(|index| Button {
            label: catalog.text("banner.assign"),
            command: Command::Play(index),
            shortcut: None,
        })
}

/// Le bandeau du choix de l'unité d'une bannière : « Bannière de gloire : à quelle unité
/// l'assigner ? » et « Annuler » (Échap).
pub(super) fn assign_prompt(
    mode: &ScreenMode,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<DiscardPrompt> {
    let ScreenMode::AssigningBanner(card) = *mode else {
        return None;
    };
    legal_actions
        .iter()
        .any(
            |action| matches!(action, Action::AssignBanner { card: banner, .. } if *banner == card),
        )
        .then(|| DiscardPrompt {
            text: catalog.format("banner.choose_unit", &[("card", &card_name(card, catalog))]),
            cancel: Button {
                label: catalog.text("discard_choice.cancel"),
                command: Command::CancelChoice,
                shortcut: shortcut_label(Shortcut::Cancel, catalog),
            },
        })
}

/// La bannière dont l'unité se choisit, tant que l'assignation reste légale.
pub(super) fn assigning(mode: &ScreenMode, legal_actions: &[Action]) -> Option<CardId> {
    match *mode {
        ScreenMode::AssigningBanner(card)
            if legal_actions.iter().any(|action| {
                matches!(action, Action::AssignBanner { card: banner, .. } if *banner == card)
            }) =>
        {
            Some(card)
        }
        _ => None,
    }
}

/// Le choix d'un artefact en cours, s'il mène encore à une action légale, avec ces actions.
pub(super) fn active_artifact(
    mode: &ScreenMode,
    legal_actions: &[Action],
) -> Option<(ArtifactPlay, Vec<(usize, ArtifactChoices)>)> {
    let ScreenMode::PlayingArtifact(choice) = *mode else {
        return None;
    };
    let plays = artifact_plays(legal_actions, choice.card, choice.strong, choice.option);
    (!plays.is_empty()).then_some((choice, plays))
}

/// Vrai si `a` et `b` ont les mêmes éléments, dans n'importe quel ordre.
fn same<T: Copy + PartialEq>(a: Picks<T>, b: Picks<T>) -> bool {
    a.len() == b.len() && a.iter().all(|item| b.contains(item))
}

/// `picks` avec `item` en plus, ou en moins s'il y est.
fn toggled<T: Copy + PartialEq>(picks: Picks<T>, item: T) -> Picks<T> {
    let mut items = picks.to_vec();
    match items.iter().position(|&picked| picked == item) {
        Some(index) => {
            items.remove(index);
        }
        None => items.push(item),
    }
    Picks::of(&items)
}

/// Vrai si `wanted` tient dans `colors`, chaque couleur comptée autant de fois qu'elle y figure.
fn holds(colors: Picks<ManaColor>, wanted: &[ManaColor]) -> bool {
    let mut left = colors.to_vec();
    wanted.iter().all(|color| {
        left.iter()
            .position(|other| other == color)
            .map(|index| left.remove(index))
            .is_some()
    })
}

/// Épée de justice : les cartes à défausser se cochent dans la main.
fn ticks_discard(plays: &[(usize, ArtifactChoices)]) -> bool {
    plays
        .iter()
        .any(|(_, choices)| !choices.discard.is_empty() && choices.gain.is_none())
}

/// Bannière de peur renforcée : les ennemis se cochent sur l'écran de combat.
fn ticks_enemies(plays: &[(usize, ArtifactChoices)]) -> bool {
    plays.iter().any(|(_, choices)| !choices.enemies.is_empty())
}

/// La carte de la main pendant le choix d'un artefact : sa case « Défausser » si l'Épée de justice
/// peut la défausser.
pub(super) fn artifact_hand_card(
    card: CardId,
    (choice, plays): &(ArtifactPlay, Vec<(usize, ArtifactChoices)>),
    catalog: &Catalog,
) -> HandCard {
    let discardable = ticks_discard(plays)
        && plays
            .iter()
            .any(|(_, choices)| choices.discard.contains(card));
    HandCard {
        discard_toggle: discardable.then(|| DiscardToggle {
            label: catalog.text("end_turn.discard"),
            checked: choice.picks.discard.contains(card),
            command: Command::ChooseArtifact(ArtifactPlay {
                picks: ArtifactChoices {
                    discard: toggled(choice.picks.discard, card),
                    ..choice.picks
                },
                ..*choice
            }),
        }),
        ..idle_card(card, catalog)
    }
}

/// La case « cible » d'un ennemi de la Bannière de peur renforcée sur l'écran de combat, par
/// ennemi qu'une action légale peut viser ; vide hors de ce choix.
pub(crate) fn artifact_enemy_toggles(
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Vec<(EnemyToken, DiscardToggle)> {
    let Some((choice, plays)) = active_artifact(mode, legal_actions) else {
        return Vec::new();
    };
    let mut enemies: Vec<EnemyToken> = Vec::new();
    for (_, choices) in &plays {
        for enemy in choices.enemies.iter() {
            if !enemies.contains(&enemy) {
                enemies.push(enemy);
            }
        }
    }
    enemies
        .into_iter()
        .map(|enemy| {
            (
                enemy,
                DiscardToggle {
                    label: catalog.text("combat.target"),
                    checked: choice.picks.enemies.contains(enemy),
                    command: Command::ChooseArtifact(ArtifactPlay {
                        picks: ArtifactChoices {
                            enemies: toggled(choice.picks.enemies, enemy),
                            ..choice.picks
                        },
                        ..choice
                    }),
                },
            )
        })
        .collect()
}

/// Le bandeau du choix d'un artefact : la couleur du jeton suivant (Amulette des ténèbres), le
/// bonus (Corne de la colère renforcée), l'action des cartes ou des ennemis cochés (Épée de justice,
/// Bannière de peur renforcée).
pub(super) fn artifact_prompt(
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<ManaPrompt> {
    let (choice, plays) = active_artifact(mode, legal_actions)?;
    let name = card_name(choice.card, catalog);
    let written = choice
        .card
        .card()
        .unsupported_text_keys()
        .map(|(basic, strong)| catalog.text(if choice.strong { strong } else { basic }))
        .unwrap_or_default();
    let mut payments: Vec<Button> = Vec::new();
    let mut push = |label: String, command: Command| {
        if !payments.iter().any(|button| button.label == label) {
            payments.push(Button {
                label,
                command,
                shortcut: None,
            });
        }
    };
    let horn = plays.iter().any(|(_, choices)| choices.bonus > 0);
    let discard = ticks_discard(&plays);
    let enemies = ticks_enemies(&plays);
    let text = if discard {
        catalog.text("sword.discard_prompt")
    } else if horn {
        catalog.text("horn.extra")
    } else {
        catalog.format("artifact.prompt", &[("card", &name), ("effect", &written)])
    };
    if plays.iter().any(|(_, choices)| !choices.colors.is_empty()) {
        // Les couleurs, une à une : la dernière joue l'artefact.
        let picked = choice.picks.colors.to_vec();
        for color in ManaColor::ALL {
            let mut next = picked.clone();
            next.push(color);
            let exact = plays.iter().find(|(_, choices)| {
                choices.colors.len() == next.len() && holds(choices.colors, &next)
            });
            let command = match exact {
                Some((index, _)) => Command::Play(*index),
                None if plays
                    .iter()
                    .any(|(_, choices)| holds(choices.colors, &next)) =>
                {
                    Command::ChooseArtifact(ArtifactPlay {
                        picks: ArtifactChoices {
                            colors: Picks::of(&next),
                            ..choice.picks
                        },
                        ..choice
                    })
                }
                None => continue,
            };
            push(
                catalog.format("artifact.color", &[("color", &mana_name(color, catalog))]),
                command,
            );
        }
    } else {
        for (index, choices) in &plays {
            if (discard && !same(choices.discard, choice.picks.discard))
                || (enemies && !same(choices.enemies, choice.picks.enemies))
            {
                continue;
            }
            let label = if discard {
                let amount = u8::try_from(3 * choices.discard.len()).unwrap_or(u8::MAX);
                let attack = Effect::Attack {
                    amount,
                    element: Element::Physical,
                    kind: AttackKind::Melee,
                };
                catalog.format("play.basic", &[("effect", &effect_text(attack, catalog))])
            } else if horn {
                // L'Attaque de siège 5 de la carte, plus le bonus.
                catalog.format(
                    "horn.bonus",
                    &[
                        ("amount", &choices.bonus.to_string()),
                        ("total", &(5 + u32::from(choices.bonus)).to_string()),
                    ],
                )
            } else {
                let key = if choice.strong {
                    "play.throw_away"
                } else {
                    "play.basic"
                };
                catalog.format(key, &[("effect", &written)])
            };
            push(label, Command::Play(*index));
        }
    }
    Some(ManaPrompt {
        text,
        payments,
        cancel: Button {
            label: catalog.text("mana_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    })
}

/// La fenêtre de fin de manche (C9) : une ligne par bannière assignée, reprise si cochée dans le
/// mode, et « Valider » qui joue les reprises cochées.
pub(super) fn reclaim_window(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<ReclaimWindow> {
    if !view.banner_reclaim {
        return None;
    }
    let picks = match *mode {
        ScreenMode::ReclaimingBanners(picks) => picks,
        _ => Picks::default(),
    };
    let banners = view
        .units
        .iter()
        .filter_map(|owned| {
            let card = owned.banner?;
            let taken = picks.contains(card);
            let with = toggled(picks, card);
            Some(ReclaimLine {
                text: catalog.format(
                    "banner.reclaim_line",
                    &[
                        ("card", &card_name(card, catalog)),
                        ("unit", &catalog.text(owned.card.unit.label_key())),
                    ],
                ),
                taken,
                keep: Button {
                    label: catalog.text("banner.reclaim_keep"),
                    command: Command::ChooseReclaimed(if taken { with } else { picks }),
                    shortcut: None,
                },
                take: Button {
                    label: catalog.text("banner.reclaim_take"),
                    command: Command::ChooseReclaimed(if taken { picks } else { with }),
                    shortcut: None,
                },
            })
        })
        .collect();
    let confirm = legal_actions
        .iter()
        .position(|action| {
            matches!(action, Action::ReclaimBanners { banners }
                if same(Picks::of(banners), picks))
        })
        .map(|index| Button {
            label: catalog.text("banner.reclaim_confirm"),
            command: Command::Play(index),
            shortcut: None,
        });
    Some(ReclaimWindow {
        title: catalog.text("banner.reclaim_title"),
        banners,
        confirm,
    })
}
