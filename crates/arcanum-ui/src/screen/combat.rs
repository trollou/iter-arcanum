//! Écran de combat (spec 031, Q11) : de « Engager le combat » à « Terminer le combat », il prend
//! la place du plateau de jeu.

use super::*;
use crate::enemy_icon_value;
use arcanum_core::UndoStatus;

/// Ce que l'écran retient du combat en cours, jamais le moteur : les ennemis cochés pour la
/// prochaine attaque, et « Voir le plateau ».
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatChoice {
    /// Dans l'ordre où ils ont été cochés, par leur jeton (spec 044).
    pub targets: Vec<EnemyToken>,
    pub viewing_board: bool,
}

/// L'écran de combat, déjà traduit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatScreen {
    /// « Combat ».
    pub title: String,
    /// Les quatre phases, dans l'ordre.
    pub phases: Vec<PhaseStep>,
    /// « Phase suivante » ou « Terminer le combat », quand l'action est légale.
    pub phase_button: Option<Button>,
    /// « Voir le plateau », ou « Retour au combat » quand le plateau est montré.
    pub board_toggle: Button,
    /// Le plateau de jeu est montré à la place de l'écran de combat.
    pub viewing_board: bool,
    /// En phase d'attaque à distance, si un Invocateur est en jeu : sa créature sera tirée au
    /// début de la phase de parade, sans retour en arrière (spec 033, Q1).
    pub summon_warning: Option<String>,
    /// Une carte par ennemi, dans l'ordre du combat.
    pub enemies: Vec<EnemyCard>,
    /// « Attaquer », dans une phase d'attaque où un ennemi peut être ciblé ; grisé tant que les
    /// ennemis cochés ne forment pas une attaque légale.
    pub attack: Option<ActionButton>,
    /// L'attaque ou la parade en cours.
    pub in_progress: Option<InProgressPanel>,
    /// « Assommé : 6 Blessures reçues dans ce combat. » (la limite de main du niveau), sous le titre de la phase (spec 034).
    pub knocked_out: Option<String>,
    /// En phase d'assignation des dégâts : un bloc par ennemi dont les dégâts sont à assigner
    /// (spec 042).
    pub damage: Vec<DamageGroup>,
    /// « Vos unités ne participent pas à ce combat », pendant l'incendie d'un monastère, à la
    /// place des capacités d'unités (spec 041).
    pub units_excluded: Option<String>,
    /// « Règles de Nuit : le mana or ne paie rien », au donjon et au tombeau (spec 044, C44).
    pub night_rules: Option<String>,
}

/// « Rôdeurs : 4 dégâts à assigner », et un bouton par destination possible (spec 042).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageGroup {
    pub text: String,
    pub buttons: Vec<Button>,
}

/// Une phase dans les étapes du haut de l'écran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseStep {
    pub label: String,
    pub state: StepState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    Done,
    Current,
    Upcoming,
}

/// Bouton qui peut être grisé : sans commande, il ne fait rien.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionButton {
    pub label: String,
    pub command: Option<Command>,
}

/// La carte d'un ennemi du combat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyCard {
    pub hex: Hex,
    pub token: EnemyToken,
    /// Sa figurine, la même que sur le plateau de jeu (spec 075, C20b), et sa couleur de repli.
    pub image: crate::ImageId,
    pub fallback: eframe::egui::Color32,
    /// Socle sous la figurine, à la couleur de sa catégorie (spec #14, C13).
    pub base: eframe::egui::Color32,
    /// « Rôdeurs », sans la famille (spec 070, C13).
    pub name: String,
    /// « orques maraudeurs » : l'infobulle du jeton (spec 070, C13).
    pub family: String,
    /// Armure, attaque à l'icône de son élément (ou « invocation »), Gloire (spec 071, C8 et C9).
    pub values: Vec<crate::ValuePiece>,
    /// Ses capacités, une icône chacune, sans valeur, avec leur règle (spec 033 ; spec #27, C3).
    pub abilities: Vec<crate::IconValue>,
    /// Ses résistances, une icône chacune, sans valeur (spec 032 ; spec 071, C10).
    pub resistances: Vec<crate::IconValue>,
    /// « paré » ou « vaincu », ou ce qui l'empêche d'être ciblé (« fortifié : siège seulement ») ;
    /// rien quand il n'y a rien à dire (spec 070, C14).
    pub status: Option<String>,
    /// Vaincu : la carte est grisée.
    pub defeated: bool,
    /// Visé par l'attaque ou la parade en cours.
    pub engaged: bool,
    /// Case « cible », seulement s'il peut être ciblé maintenant.
    pub target: Option<DiscardToggle>,
    /// « Parer … », quand la parade est légale.
    pub block: Option<Button>,
    /// La créature invoquée par cet Invocateur, pendant la parade et les dégâts (spec 033).
    pub summoned: Option<SummonedCard>,
    /// « À parer : 14 (vif, 2 × 7) » puis « Si non paré : 7 dégâts, 4 blessure(s) », en phase de
    /// parade pour un ennemi qui attaque, ni paré ni vaincu (spec 033, Q4 ; spec 070, C16) ; vide
    /// sinon.
    pub stakes: Vec<String>,
    /// « Dans ce combat (2) », replié par défaut ; rien sans ligne (spec 070, C15).
    pub history_title: Option<String>,
    /// Ce qui lui est arrivé dans ce combat, dans l'ordre.
    pub history: Vec<String>,
}

/// La créature invoquée, dans la carte de son Invocateur (spec 033).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonedCard {
    pub token: EnemyToken,
    /// « Invoque : Minotaure ».
    pub title: String,
    /// Ses valeurs en icônes, comme celles de la carte d'ennemi (spec 071, C11).
    pub values: Vec<crate::ValuePiece>,
    /// Ses capacités en icônes, comme celles de la carte d'ennemi (spec #27, C7).
    pub abilities: Vec<crate::IconValue>,
    /// « retirée après les dégâts, sans gloire ».
    pub note: String,
}

/// L'attaque ou la parade en cours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InProgressPanel {
    /// Noms des cartes jouées, dans l'ordre, suivis de « ÷2 » quand elles comptent à moitié
    /// (spec 032).
    pub cards: Vec<String>,
    /// « Il faut la somme des armures : 3 + 3 = 6 », pour une attaque.
    pub needed: Option<String>,
    /// « Le groupe partage : résistance physique, des Cogneurs », pour une attaque dont une cible
    /// a une résistance, ou est fortifiée en phase d'attaque à distance (spec 032).
    pub shared: Option<String>,
    /// « Contre une attaque de glace, … », pour une parade contre une attaque élémentaire.
    pub block_rule: Option<String>,
    /// « Vif : il faut parer le double de son attaque, 2 × 7 = 14 » (spec 033).
    pub swift: Option<String>,
    /// « Divisées par deux : (2 + 1) ÷ 2 = 1 · Autres : 3 », s'il y a des points divisés.
    pub calculation: Option<String>,
    /// L'icône de la phase (arc, épée) ou la rondache, et « 3 / 6 » : le total qui compte sur ce
    /// qu'il faut (spec 071, C12).
    pub total: crate::IconValue,
    /// « Valider l'attaque » ou « Valider la parade » ; « Annuler » si elle ne suffit pas (spec
    /// 070).
    pub resolve: Option<Button>,
    /// « Mouvement en Attaque (2 restant) » ou « Influence en Parade (4 restante) » : un point
    /// converti par clic, quand l'action est légale (spec 052, C28, C29, C31 ; Q4).
    pub conversion: Option<Button>,
    /// « Ajouter l'Attaque de feu 4 » : l'attaque différée du Bouclier ardent, quand l'action est
    /// légale (spec 054, C19).
    pub deferred_attack: Option<Button>,
}

/// L'écran de combat, tant qu'un combat est engagé et pas terminé ; calculé depuis la vue seule.
pub fn combat_screen(
    view: &PlayerView,
    legal_actions: &[Action],
    choice: &CombatChoice,
    undo: UndoStatus,
    catalog: &Catalog,
) -> Option<CombatScreen> {
    let combat = view.combat.as_ref().filter(|combat| !combat.ended)?;
    // Spec 044 : le combat d'un donjon ou d'un tombeau, seul et sous les règles de Nuit.
    let night_site = view
        .adventure
        .and_then(|hex| site_at(view, hex))
        .is_some_and(is_night_site);
    let current = PHASES.iter().position(|phase| *phase == combat.phase);
    let phases = PHASES
        .iter()
        .enumerate()
        .map(|(index, phase)| PhaseStep {
            label: catalog.text(phase_key(*phase)),
            state: match current.map(|current| index.cmp(&current)) {
                Some(std::cmp::Ordering::Less) => StepState::Done,
                Some(std::cmp::Ordering::Equal) => StepState::Current,
                _ => StepState::Upcoming,
            },
        })
        .collect();
    let phase_button = legal_actions
        .iter()
        .enumerate()
        .find_map(|(index, action)| {
            let key = match action {
                Action::NextCombatPhase => "combat.next_phase",
                // Issue #8 : l'invocation attend le choix de la Bannière de terreur.
                Action::CallSummons => "combat.call_summons",
                Action::EndCombat => "combat.end",
                _ => return None,
            };
            Some(Button {
                label: catalog.text(key),
                command: Command::Play(index),
                shortcut: None,
            })
        });
    // Les ennemis ciblables : ceux d'au moins une attaque légale (Q10).
    let targetable: Vec<EnemyToken> = combat
        .enemies
        .iter()
        .map(|enemy| enemy.token)
        .filter(|token| {
            legal_actions.iter().any(
                |action| matches!(action, Action::DeclareAttack { targets } if targets.contains(token)),
            )
        })
        .collect();
    // Spec 070, C1 et C2 : une seule cible possible est cochée d'office, sans case.
    let single = targetable.len() == 1;
    // Les ennemis cochés encore ciblables, dans l'ordre du combat.
    let selected: Vec<EnemyToken> = targetable
        .iter()
        .copied()
        .filter(|token| single || choice.targets.contains(token))
        .collect();
    let attack = (!targetable.is_empty()).then(|| ActionButton {
        label: catalog.text("combat.attack"),
        command: legal_actions
            .iter()
            .position(|action| {
                matches!(action, Action::DeclareAttack { targets }
                    if !selected.is_empty()
                        && targets.len() == selected.len()
                        && selected.iter().all(|token| targets.contains(token)))
            })
            .map(Command::Play),
    });
    let engaged: Vec<EnemyToken> = match &combat.in_progress {
        Some(InProgress::Attack { targets, .. }) => targets.clone(),
        Some(InProgress::Block { target, .. }) => vec![*target],
        None => Vec::new(),
    };
    let enemies = combat
        .enemies
        .iter()
        .map(|enemy| {
            let history: Vec<String> = combat
                .history
                .iter()
                .filter_map(|record| history_line(record, enemy.token, catalog))
                .collect();
            EnemyCard {
                hex: enemy.hex,
                token: enemy.token,
                image: crate::figure_image(enemy.token.enemy()),
                fallback: crate::enemy_color(enemy.token.color()),
                base: crate::enemy_color(enemy.token.color()),
                values: combat_enemy_values(enemy, catalog),
                abilities: enemy_abilities(enemy, catalog),
                resistances: resistance_icons(&enemy.effective_resistances(), catalog),
                status: status(enemy, combat.phase, catalog),
                defeated: enemy.defeated,
                engaged: engaged.contains(&enemy.token),
                target: (!single && targetable.contains(&enemy.token)).then(|| DiscardToggle {
                    label: catalog.text("combat.target"),
                    checked: selected.contains(&enemy.token),
                    command: Command::ToggleTarget(enemy.token),
                }),
                block: legal_actions
                    .iter()
                    .position(|action| {
                        matches!(action, Action::DeclareBlock { target } if *target == enemy.token)
                    })
                    .map(|index| Button {
                        label: catalog.text("combat.block"),
                        command: Command::Play(index), shortcut: None,
                    }),
                summoned: enemy.summoned.map(|token| SummonedCard {
                    token,
                    title: catalog.format(
                        "combat.enemy.summons",
                        &[("monster", &catalog.text(token.enemy().label_key()))],
                    ),
                    values: enemy_values(
                        token,
                        token.enemy().definition().armor,
                        attack_value(token),
                        catalog,
                    ),
                    abilities: token_ability_icons(token, catalog),
                    note: catalog.text("combat.enemy.summoned_note"),
                }),
                stakes: (combat.phase == CombatPhase::Block && !enemy.defeated && !enemy.blocked)
                    .then(|| stakes(enemy, view.armor, catalog))
                    .flatten()
                    .unwrap_or_default(),
                history_title: (!history.is_empty()).then(|| {
                    catalog.format(
                        "combat.history.count",
                        &[("count", &history.len().to_string())],
                    )
                }),
                history,
                name: catalog.text(enemy.token.enemy().label_key()),
                family: catalog.text(enemy.token.enemy().definition().color.label_key()),
            }
        })
        .collect();
    Some(CombatScreen {
        title: catalog.text("combat.title"),
        phases,
        phase_button,
        board_toggle: Button {
            label: catalog.text(if choice.viewing_board {
                "combat.back"
            } else {
                "combat.show_board"
            }),
            command: Command::ToggleCombatBoard,
            shortcut: None,
        },
        viewing_board: choice.viewing_board,
        summon_warning: summon_warning(combat, catalog),
        enemies,
        attack,
        in_progress: in_progress_panel(combat, view, legal_actions, undo, catalog),
        // Spec 035 : le seuil est la limite de main du niveau.
        knocked_out: combat.knocked_out.then(|| {
            catalog.format(
                "combat.knocked_out",
                &[("limit", &view.hand_limit.to_string())],
            )
        }),
        damage: damage_groups(combat, legal_actions, catalog),
        // Spec 041 : le combat d'un incendie se fait sans les unités.
        // Spec 041 et 044 : sans les unités pendant un incendie, au donjon et au tombeau.
        units_excluded: (view.burning.is_some() || night_site)
            .then(|| catalog.text("burn.no_units")),
        night_rules: night_site.then(|| catalog.text("combat.night_rules")),
    })
}

/// Les blocs de l'assignation des dégâts : un par ennemi qu'une action légale vise, dans l'ordre
/// du combat, avec ses dégâts restants et un bouton par destination (MKUE p. 9, *Assign Damage
/// Phase* ; spec 042).
fn damage_groups(combat: &Combat, legal_actions: &[Action], catalog: &Catalog) -> Vec<DamageGroup> {
    combat
        .enemies
        .iter()
        .filter_map(|enemy| {
            let buttons: Vec<Button> = legal_actions
                .iter()
                .enumerate()
                .filter_map(|(index, action)| {
                    let Action::AssignDamage { enemy: token, to } = action else {
                        return None;
                    };
                    if *token != enemy.token {
                        return None;
                    }
                    let label = match to {
                        DamageTarget::Hero => catalog.text("damage.to_hero"),
                        DamageTarget::Unit(unit) => catalog.format(
                            "damage.to_unit",
                            &[
                                ("name", &catalog.text(unit.unit.label_key())),
                                ("armor", &unit.unit.definition().armor.to_string()),
                            ],
                        ),
                    };
                    Some(Button {
                        label,
                        command: Command::Play(index),
                        shortcut: None,
                    })
                })
                .collect();
            if buttons.is_empty() {
                return None;
            }
            let remaining = combat.remaining_damage(enemy).unwrap_or(0);
            let name = enemy
                .attacker()
                .map_or_else(String::new, |token| catalog.text(token.enemy().label_key()));
            Some(DamageGroup {
                text: catalog.format(
                    "damage.remaining",
                    &[("enemy", &name), ("damage", &remaining.to_string())],
                ),
                buttons,
            })
        })
        .collect()
}

const PHASES: [CombatPhase; 4] = [
    CombatPhase::RangedAndSiege,
    CombatPhase::Block,
    CombatPhase::AssignDamage,
    CombatPhase::Attack,
];

fn phase_key(phase: CombatPhase) -> &'static str {
    match phase {
        CombatPhase::RangedAndSiege => "combat.phase.ranged",
        CombatPhase::Block => "combat.phase.block",
        CombatPhase::AssignDamage => "combat.phase.damage",
        CombatPhase::Attack => "combat.phase.attack",
    }
}

/// Valeur de l'attaque imprimée sur un jeton, quel que soit l'élément ; 0 pour une invocation.
fn attack_value(token: EnemyToken) -> u32 {
    match token.enemy().definition().attack {
        EnemyAttack::Physical(value)
        | EnemyAttack::Fire(value)
        | EnemyAttack::Ice(value)
        | EnemyAttack::ColdFire(value) => u32::from(value),
        EnemyAttack::Summon => 0,
    }
}

fn has_ability(token: EnemyToken, ability: EnemyAbility) -> bool {
    token.enemy().definition().abilities.contains(&ability)
}

/// « Au début de la phase de parade, Invocateurs orques invoquent une créature… » (spec 033, Q1).
fn summon_warning(combat: &Combat, catalog: &Catalog) -> Option<String> {
    if combat.phase != CombatPhase::RangedAndSiege {
        return None;
    }
    let names: Vec<String> = combat
        .enemies
        .iter()
        .filter(|enemy| {
            // Spec 053 : un Invocateur qui n'attaque pas ne tire pas de créature.
            !enemy.defeated
                && !enemy.does_not_attack()
                && enemy.token.enemy().definition().attack == EnemyAttack::Summon
        })
        .map(|enemy| catalog.text(enemy.token.enemy().label_key()))
        .collect();
    (!names.is_empty())
        .then(|| catalog.format("combat.summon_warning", &[("enemies", &names.join(", "))]))
}

/// « À parer : 14 (vif, 2 × 7) », puis « Si non paré : 8 (brutal, 2 × 4) dégâts, 4 blessure(s) »
/// (spec 033, Q4 ; spec 070, C16) ; `None` pour un Invocateur sans créature.
///
/// Spec 062 : l'attaque et les capacités qui comptent, bonus de la cité compris.
fn stakes(enemy: &CombatEnemy, armor: u8, catalog: &Catalog) -> Option<Vec<String>> {
    let attack = enemy.effective_attack()?.to_string();
    let block = enemy.block_needed()?.to_string();
    let damage = enemy.damage()?.to_string();
    let block = if enemy.has_ability(EnemyAbility::Swift) {
        catalog.format(
            "combat.enemy.swift_block",
            &[("attack", &attack), ("block", &block)],
        )
    } else {
        block
    };
    let damage = if enemy.has_ability(EnemyAbility::Brutal) {
        catalog.format(
            "combat.enemy.brutal_damage",
            &[("attack", &attack), ("damage", &damage)],
        )
    } else {
        damage
    };
    Some(vec![
        catalog.format("combat.enemy.stakes_block", &[("block", &block)]),
        catalog.format(
            "combat.enemy.stakes_damage",
            &[
                ("damage", &damage),
                ("wounds", &enemy.wounds_if_unblocked(armor)?.to_string()),
            ],
        ),
    ])
}

/// « paré » ou « vaincu », et ce qui l'empêche d'être ciblé ; `None` s'il n'y a rien à dire
/// (spec 070, C14).
fn status(enemy: &CombatEnemy, phase: CombatPhase, catalog: &Catalog) -> Option<String> {
    let mut parts = Vec::new();
    // Spec 054 : détruit par un sort, plutôt que vaincu.
    if enemy.destroyed {
        parts.push(catalog.text("combat.enemy.destroyed"));
    } else if enemy.defeated {
        parts.push(catalog.text("combat.enemy.defeated"));
    } else if enemy.blocked {
        parts.push(catalog.text("combat.enemy.blocked"));
    }
    if !enemy.defeated {
        // Spec 043, C39 : le défenseur d'un site fortifié ; deux fois fortifié, aucune attaque à
        // distance ne le vise (MKUE p. 9, 3.a ; Compendium, *Combat* S13).
        // Spec 054, C31 : Exposition lui retire ses fortifications.
        let lost = enemy.lost_fortifications();
        if lost && (enemy.site_fortified || enemy.is_fortified()) {
            parts.push(catalog.text("combat.enemy.no_fortification"));
        } else if enemy.fortification() >= 2 && phase == CombatPhase::RangedAndSiege {
            parts.push(catalog.text("combat.enemy.fortified_twice"));
        } else if enemy.site_fortified && !lost {
            parts.push(catalog.text("combat.enemy.site_fortified"));
        }
        // MKUE p. 9, *Ranged and Siege Attack Phase* 3 : fortifié, seul le siège l'atteint.
        if phase == CombatPhase::RangedAndSiege && enemy.fortification() == 1 {
            parts.push(catalog.text("combat.enemy.siege_only"));
        }
        // Spec 053, C37 : les modificateurs d'ennemi.
        let printed = enemy.token.enemy().definition().armor;
        if enemy.effective_armor() != printed {
            parts.push(catalog.format(
                "combat.enemy.armor_reduced",
                &[
                    ("armor", &enemy.effective_armor().to_string()),
                    ("printed", &printed.to_string()),
                ],
            ));
        }
        // Spec 062, C39 : l'attaque renforcée par la cité bleue, comme une armure modifiée.
        let printed_attack = enemy.attacker().and_then(|token| {
            u8::try_from(attack_value(token))
                .ok()
                .filter(|&value| value > 0)
        });
        if let (Some(attack), Some(printed)) = (enemy.effective_attack(), printed_attack)
            && attack != printed
        {
            parts.push(catalog.format(
                "combat.enemy.attack_increased",
                &[
                    ("attack", &attack.to_string()),
                    ("printed", &printed.to_string()),
                ],
            ));
        }
        if enemy.does_not_attack() {
            parts.push(catalog.text("combat.enemy.does_not_attack"));
        }
        // Spec 054, C31 : les résistances perdues (Exposition, Frisson).
        let printed = &enemy.token.enemy().definition().resistances;
        let lost_all = enemy
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == arcanum_core::EnemyModifierKind::LosesResistances);
        if lost_all && !printed.is_empty() {
            parts.push(catalog.text("combat.enemy.no_resistances"));
        } else if printed.contains(&Resistance::Fire)
            && !enemy.effective_resistances().contains(&Resistance::Fire)
        {
            parts.push(catalog.text("combat.enemy.no_fire_resistance"));
        }
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// Les noms des cartes jouées, puis des unités activées (spec 042), « — » s'il n'y en a pas.
fn card_names(cards: &[CardId], units: &[UnitCard], catalog: &Catalog) -> String {
    if cards.is_empty() && units.is_empty() {
        return "—".to_string();
    }
    cards
        .iter()
        .map(|card| card_name(*card, catalog))
        .chain(units.iter().map(|unit| catalog.text(unit.unit.label_key())))
        .collect::<Vec<_>>()
        .join(", ")
}

/// La ligne « Dans ce combat » de cet enregistrement, s'il concerne l'ennemi `hex` (son jeton).
fn history_line(record: &CombatRecord, hex: EnemyToken, catalog: &Catalog) -> Option<String> {
    let line = |phase: CombatPhase,
                cards: &[CardId],
                units: &[UnitCard],
                points: u32,
                needed: u32,
                success: bool| {
        catalog.format(
            "combat.history.attack",
            &[
                ("phase", &catalog.text(phase_key(phase))),
                ("cards", &card_names(cards, units, catalog)),
                ("total", &points.to_string()),
                ("needed", &needed.to_string()),
                (
                    "result",
                    &catalog.text(if success {
                        "combat.history.success"
                    } else {
                        "combat.history.failure"
                    }),
                ),
            ],
        )
    };
    match record {
        CombatRecord::Attack {
            phase,
            targets,
            cards,
            points,
            needed,
            success,
            units,
            ..
        } if targets.contains(&hex) => Some(line(*phase, cards, units, *points, *needed, *success)),
        CombatRecord::Block {
            target,
            cards,
            points,
            needed,
            success,
            units,
            ..
        } if *target == hex => Some(line(
            CombatPhase::Block,
            cards,
            units,
            *points,
            *needed,
            *success,
        )),
        CombatRecord::Damage {
            enemy,
            attacker,
            damage,
            wounds,
            poisoned,
            paralyzed,
        } if *enemy == hex => Some(damage_line(
            *attacker, *damage, *wounds, *poisoned, *paralyzed, catalog,
        )),
        // Spec 034 : après les dégâts de l'ennemi qui l'a provoqué.
        CombatRecord::KnockedOut { enemy, cards } if *enemy == hex => Some(catalog.format(
            "combat.history.knocked_out",
            &[("count", &cards.to_string())],
        )),
        // Spec 042 : les Blessures d'une unité, après celles du héros dans l'ordre du combat.
        CombatRecord::UnitDamage {
            enemy,
            unit,
            wounds,
        } if *enemy == hex => Some(catalog.format(
            "combat.history.unit_damage",
            &[
                ("name", &catalog.text(unit.unit.label_key())),
                ("wounds", &wounds.to_string()),
            ],
        )),
        CombatRecord::UnitDestroyed { enemy, unit } if *enemy == hex => Some(catalog.format(
            "combat.history.unit_destroyed",
            &[("name", &catalog.text(unit.unit.label_key()))],
        )),
        // Spec 040 : après la ligne de l'attaque qui a vaincu des ennemis errants.
        CombatRecord::Reputation { targets, change } if targets.contains(&hex) => {
            Some(catalog.format(
                "combat.history.reputation",
                &[("amount", &super::interaction::signed(i32::from(*change)))],
            ))
        }
        _ => None,
    }
}

/// « Dégâts : Cogneurs, brutal 2 × 4 = 8 : 4 blessure(s) », puis l'effet toxique ou paralysant
/// (spec 033).
fn damage_line(
    attacker: EnemyToken,
    damage: u32,
    wounds: u8,
    poisoned: u8,
    paralyzed: u8,
    catalog: &Catalog,
) -> String {
    let name = catalog.text(attacker.enemy().label_key());
    let shown = if has_ability(attacker, EnemyAbility::Brutal) {
        catalog.format(
            "combat.history.brutal",
            &[
                // Spec 062 : l'attaque qui a compté, bonus de la cité bleue compris.
                ("attack", &(damage / 2).to_string()),
                ("damage", &damage.to_string()),
            ],
        )
    } else {
        damage.to_string()
    };
    let mut line = catalog.format(
        "combat.history.damage_enemy",
        &[
            ("enemy", &name),
            ("damage", &shown),
            ("count", &wounds.to_string()),
        ],
    );
    if poisoned > 0 {
        line = format!(
            "{line}, {}",
            catalog.format("combat.history.poison", &[("count", &poisoned.to_string())])
        );
    }
    if paralyzed > 0 {
        line = format!(
            "{line}, {}",
            catalog.format(
                "combat.history.paralyze",
                &[("count", &paralyzed.to_string())]
            )
        );
    }
    line
}

/// « Annuler » l'attaque ou la parade en cours, quand elle n'atteint pas ce qu'il faut et que le
/// retour en arrière est possible (spec 070, C7, C19 ; L3, C21) : la valider ne ferait que perdre
/// les cartes jouées (MKUE p. 9, *Ranged and Siege Attack Phase* 6, *Block Phase* 6).
pub(crate) fn cancel_command(combat: &Combat, undo: UndoStatus) -> Option<Command> {
    if undo != UndoStatus::Possible {
        return None;
    }
    let (command, needed) = match combat.in_progress.as_ref()? {
        InProgress::Attack { targets, .. } => (
            Command::CancelAttack,
            combat
                .enemies
                .iter()
                .filter(|enemy| targets.contains(&enemy.token))
                .map(|enemy| u32::from(enemy.effective_armor()))
                .sum(),
        ),
        InProgress::Block { target, .. } => (
            Command::CancelBlock,
            combat.enemy(*target).and_then(CombatEnemy::block_needed)?,
        ),
    };
    (combat.in_progress_total().unwrap_or(0) < needed).then_some(command)
}

/// « Ajouter l'Attaque de feu 4 » : l'attaque différée du Bouclier ardent, tant que l'action est
/// légale (spec 054, C19).
fn deferred_attack_button(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<Button> {
    let index = legal_actions
        .iter()
        .position(|action| *action == Action::UseDeferredAttack)?;
    let (amount, element) = view
        .modifiers
        .iter()
        .find_map(|modifier| match modifier.kind {
            ModifierKind::DeferredAttack { amount, element } => Some((amount, element)),
            _ => None,
        })?;
    let attack = effect_text(
        Effect::Attack {
            amount,
            element,
            kind: AttackKind::Melee,
        },
        catalog,
    );
    Some(Button {
        label: catalog.format("combat.use_deferred_attack", &[("attack", &attack)]),
        command: Command::Play(index),
        shortcut: None,
    })
}

/// L'attaque ou la parade en cours : cartes, ce qu'il faut atteindre, total et « Valider », ou
/// « Annuler » si elle ne suffit pas (spec 070).
fn in_progress_panel(
    combat: &Combat,
    view: &PlayerView,
    legal_actions: &[Action],
    undo: UndoStatus,
    catalog: &Catalog,
) -> Option<InProgressPanel> {
    let play_area = &view.play_area;
    // Spec 052, Q4 : un clic convertit un point ; le bouton n'existe que si l'action est légale.
    let conversion = |wanted: Action, key: &str, name: &str, points: String| {
        legal_actions
            .iter()
            .position(|action| *action == wanted)
            .map(|index| Button {
                label: catalog.format(key, &[(name, &points)]),
                command: Command::Play(index),
                shortcut: None,
            })
    };
    let cancel = cancel_command(combat, undo);
    let resolve = |wanted: Action, key: &str| match cancel {
        Some(command) => Some(Button {
            label: catalog.text("combat.cancel"),
            command,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        }),
        None => legal_actions
            .iter()
            .position(|action| *action == wanted)
            .map(|index| Button {
                label: catalog.text(key),
                command: Command::Play(index),
                shortcut: shortcut_label(Shortcut::Confirm, catalog),
            }),
    };
    let tokens = |targets: &[EnemyToken]| -> Vec<&CombatEnemy> {
        combat
            .enemies
            .iter()
            .filter(|enemy| targets.contains(&enemy.token))
            .collect()
    };
    // Spec 071, C12 : l'arc en phase d'attaque à distance, l'épée en phase d'attaque, la
    // rondache physique pour une parade ; « 3 / 7 ».
    let total = |icon: Icon, key: &str, points: u32, needed: u32| crate::IconValue {
        icon,
        value: format!("{points} / {needed}"),
        tooltip: catalog.text(key),
        rule: None,
    };
    // L'effet joué par chaque carte de l'attaque ou de la parade en cours.
    let played: Vec<(CardId, Option<Effect>)> = combat
        .played
        .iter()
        .map(|card| {
            let effect = play_area
                .iter()
                .find(|played| played.card == *card)
                .map(|played| played.effect);
            (*card, effect)
        })
        .collect();
    let halved = |effect: Option<Effect>| {
        effect.and_then(|effect| combat.efficiency(effect)) == Some(Efficiency::Halved)
    };
    let cards = played
        .iter()
        .map(|&(card, effect)| {
            let name = card_name(card, catalog);
            if halved(effect) {
                format!("{name} {}", catalog.text("combat.mark.halved"))
            } else {
                name
            }
        })
        // Spec 042 : les unités activées pour l'attaque ou la parade, à la suite des cartes.
        .chain(
            combat
                .played_units
                .iter()
                .map(|unit| catalog.text(unit.unit.label_key())),
        )
        .collect();
    let calculation = calculation(combat, &played, &halved, catalog);
    let points = combat.in_progress_total().unwrap_or(0);
    match &combat.in_progress {
        Some(InProgress::Attack { targets, .. }) => {
            // Spec 053 : l'armure effective, réduite par les modificateurs d'ennemi.
            let armors: Vec<u32> = tokens(targets)
                .iter()
                .map(|enemy| u32::from(enemy.effective_armor()))
                .collect();
            let needed: u32 = armors.iter().sum();
            let mut sum = armors
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(" + ");
            if armors.len() > 1 {
                sum = format!("{sum} = {needed}");
            }
            Some(InProgressPanel {
                deferred_attack: deferred_attack_button(view, legal_actions, catalog),
                cards,
                needed: Some(catalog.format("combat.needed_armor", &[("sum", &sum)])),
                shared: shared_line(combat, targets, catalog),
                block_rule: None,
                swift: None,
                calculation,
                total: if combat.phase == CombatPhase::RangedAndSiege {
                    total(
                        crate::attack_icon(AttackKind::Ranged, Element::Physical),
                        "icon.ranged_or_siege",
                        points,
                        needed,
                    )
                } else {
                    total(
                        crate::attack_icon(AttackKind::Melee, Element::Physical),
                        "icon.attack.physical",
                        points,
                        needed,
                    )
                },
                resolve: resolve(Action::ResolveAttack, "combat.resolve_attack"),
                conversion: conversion(
                    Action::SpendMoveAsAttack,
                    if combat.phase == CombatPhase::RangedAndSiege {
                        "combat.move_as_ranged"
                    } else {
                        "combat.move_as_attack"
                    },
                    "move",
                    view.move_points.to_string(),
                ),
            })
        }
        Some(InProgress::Block { target, .. }) => {
            // La créature invoquée pour un Invocateur (spec 033).
            let enemy = combat.enemy(*target);
            let token = enemy.and_then(CombatEnemy::attacker);
            let needed = enemy.and_then(CombatEnemy::block_needed).unwrap_or(0);
            let swift = enemy
                .filter(|enemy| enemy.has_ability(EnemyAbility::Swift))
                .and_then(CombatEnemy::effective_attack)
                .map(|attack| {
                    catalog.format(
                        "combat.swift_line",
                        &[
                            ("attack", &attack.to_string()),
                            ("block", &needed.to_string()),
                        ],
                    )
                });
            let block_rule = token
                .and_then(|token| token.enemy().definition().attack.element())
                .and_then(|element| match element {
                    Element::Physical => None,
                    Element::Fire => Some("combat.block_efficient.fire"),
                    Element::Ice => Some("combat.block_efficient.ice"),
                    Element::ColdFire => Some("combat.block_efficient.cold_fire"),
                })
                .map(|key| catalog.text(key));
            Some(InProgressPanel {
                deferred_attack: None,
                cards,
                needed: None,
                shared: None,
                block_rule,
                swift,
                calculation,
                total: total(
                    crate::block_icon(Element::Physical),
                    "icon.block.physical",
                    points,
                    needed,
                ),
                resolve: resolve(Action::ResolveBlock, "combat.resolve_block"),
                conversion: conversion(
                    Action::SpendInfluenceAsBlock,
                    "combat.influence_as_block",
                    "influence",
                    view.influence_points.to_string(),
                ),
            })
        }
        None => None,
    }
}

/// Armure, attaque et Gloire d'un jeton (spec 071, C8 et C9) ; l'armure qui compte, réduite ou non
/// (spec 053, C37), et l'attaque qui compte, renforcée par la cité bleue (spec 062, C39).
fn enemy_values(
    token: EnemyToken,
    armor: u8,
    attack: u32,
    catalog: &Catalog,
) -> Vec<crate::ValuePiece> {
    use crate::ValuePiece;
    let definition = token.enemy().definition();
    let attack = match definition.attack.element() {
        Some(element) => ValuePiece::Icon(enemy_icon_value(
            crate::attack_icon(AttackKind::Melee, element),
            attack.to_string(),
            catalog,
        )),
        // Spec #27, C6 : l'icône Invocation, sans valeur, à la place du mot.
        None => ValuePiece::Icon(enemy_icon_value(Icon::Summon, "", catalog)),
    };
    vec![
        ValuePiece::Icon(icon_value(Icon::Armor, armor.to_string(), catalog)),
        attack,
        ValuePiece::Icon(icon_value(Icon::Fame, definition.fame.to_string(), catalog)),
    ]
}

/// Les valeurs d'un ennemi du combat : celles qui comptent, sauf l'attaque d'un Invocateur, dont
/// la créature a sa propre carte.
fn combat_enemy_values(enemy: &CombatEnemy, catalog: &Catalog) -> Vec<crate::ValuePiece> {
    let attack = enemy
        .effective_attack()
        .filter(|_| enemy.summoned.is_none())
        .map_or_else(|| attack_value(enemy.token), u32::from);
    enemy_values(enemy.token, enemy.effective_armor(), attack, catalog)
}

/// Les capacités d'un ennemi en icônes (spec #27, C3 et C8) : celles du jeton, puis celles gagnées
/// de sa cité (spec 062, C39), titrées « Brutal (cité) ».
fn enemy_abilities(enemy: &CombatEnemy, catalog: &Catalog) -> Vec<crate::IconValue> {
    let mut abilities = token_ability_icons(enemy.token, catalog);
    abilities.extend(enemy.city_abilities().into_iter().map(|ability| {
        let value = enemy_icon_value(Icon::Ability(ability), "", catalog);
        crate::IconValue {
            tooltip: catalog.format("combat.enemy.city_ability", &[("ability", &value.tooltip)]),
            ..value
        }
    }));
    abilities
}

/// Les capacités d'un jeton, dans l'ordre de `enemies.ron` (spec #27, C3 et C7).
fn token_ability_icons(token: EnemyToken, catalog: &Catalog) -> Vec<crate::IconValue> {
    token
        .enemy()
        .definition()
        .abilities
        .iter()
        .map(|&ability| enemy_icon_value(Icon::Ability(ability), "", catalog))
        .collect()
}

/// Une icône par résistance, sans valeur (spec 071, C10).
/// Spec 054 : les résistances qui comptent, sans celles perdues.
fn resistance_icons(resistances: &[Resistance], catalog: &Catalog) -> Vec<crate::IconValue> {
    resistances
        .iter()
        .map(|&resistance| enemy_icon_value(Icon::Resistance(resistance), "", catalog))
        .collect()
}

/// « Le groupe partage : … » : la fortification des cibles en phase d'attaque à distance, puis
/// chaque résistance, avec les ennemis qui l'apportent (MKUE p. 9, 3.a et 4.a ; Compendium,
/// *Combat* S12). `None` si le groupe ne partage rien.
fn shared_line(combat: &Combat, targets: &[EnemyToken], catalog: &Catalog) -> Option<String> {
    let enemies: Vec<&CombatEnemy> = combat
        .enemies
        .iter()
        .filter(|enemy| targets.contains(&enemy.token))
        .collect();
    let names = |keep: &dyn Fn(&CombatEnemy) -> bool| -> Option<String> {
        let names: Vec<String> = enemies
            .iter()
            .filter(|enemy| keep(enemy))
            .map(|enemy| catalog.text(enemy.token.enemy().label_key()))
            .collect();
        (!names.is_empty()).then(|| names.join(", "))
    };
    let mut items = Vec::new();
    if combat.phase == CombatPhase::RangedAndSiege
        // Spec 043 : fortifié une fois, par sa capacité ou par son site ; deux fois, il n'est
        // plus une cible à distance.
        && let Some(names) = names(&|enemy| enemy.fortification() == 1)
    {
        items.push(catalog.format("combat.shared.fortified", &[("enemies", &names)]));
    }
    for (resistance, key) in [
        (Resistance::Physical, "combat.shared.physical"),
        (Resistance::Fire, "combat.shared.fire"),
        (Resistance::Ice, "combat.shared.ice"),
    ] {
        // Spec 054 : les résistances qui comptent.
        let resists = |enemy: &CombatEnemy| enemy.effective_resistances().contains(&resistance);
        if let Some(names) = names(&resists) {
            items.push(catalog.format(key, &[("enemies", &names)]));
        }
    }
    (!items.is_empty()).then(|| {
        format!(
            "{} {}",
            catalog.text("combat.shared.title"),
            items.join(" ; ")
        )
    })
}

/// « Divisées par deux : (2 + 1) ÷ 2 = 1 · Autres : 3 », seulement s'il y a des points divisés
/// (MKUE p. 9, 4.a et *Block Phase* 5.b : une seule division).
fn calculation(
    combat: &Combat,
    played: &[(CardId, Option<Effect>)],
    halved: &dyn Fn(Option<Effect>) -> bool,
    catalog: &Catalog,
) -> Option<String> {
    let amounts: Vec<String> = played
        .iter()
        .filter(|(_, effect)| halved(*effect))
        .filter_map(|(_, effect)| effect.and_then(|effect| combat.counted_points(effect)))
        .map(|amount| amount.to_string())
        .collect();
    if amounts.is_empty() {
        return None;
    }
    let (halved_sum, full) = combat.in_progress_split()?;
    let listed = if amounts.len() > 1 {
        format!("({})", amounts.join(" + "))
    } else {
        amounts.join("")
    };
    Some(catalog.format(
        "combat.calculation",
        &[
            ("halved", &listed),
            ("halved_sum", &(halved_sum / 2).to_string()),
            ("full", &full.to_string()),
        ],
    ))
}
