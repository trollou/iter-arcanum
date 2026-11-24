//! Textes d'un jeton d'ennemi (spec 030) : nom, valeurs, capacités et résistances.

use arcanum_core::{EnemyAbility, EnemyAttack, EnemyToken, Resistance};
use arcanum_i18n::Catalog;

/// « Rôdeurs (orques maraudeurs) ».
pub fn token_name(token: EnemyToken, catalog: &Catalog) -> String {
    let enemy = token.enemy();
    catalog.format(
        "enemy.token",
        &[
            ("name", &catalog.text(enemy.label_key())),
            ("group", &catalog.text(enemy.definition().color.label_key())),
        ],
    )
}

/// Plusieurs jetons, séparés par des virgules : « Rôdeurs (orques maraudeurs), … ».
pub fn token_names(tokens: impl IntoIterator<Item = EnemyToken>, catalog: &Catalog) -> String {
    let names: Vec<String> = tokens
        .into_iter()
        .map(|token| token_name(token, catalog))
        .collect();
    join(&names)
}

/// Lignes du survol d'un jeton : son nom et ses valeurs, puis ses capacités et ses résistances
/// s'il en a (spec 030, C23).
pub fn token_lines(token: EnemyToken, catalog: &Catalog) -> Vec<String> {
    let definition = token.enemy().definition();
    let values = token_values(token, catalog);
    let mut lines = vec![catalog.format(
        "enemy.description",
        &[("token", &token_name(token, catalog)), ("values", &values)],
    )];
    if !definition.abilities.is_empty() {
        let abilities: Vec<String> = definition
            .abilities
            .iter()
            .map(|ability| catalog.text(ability_key(*ability)))
            .collect();
        lines.push(catalog.format("enemy.abilities", &[("list", &join(&abilities))]));
    }
    if !definition.resistances.is_empty() {
        let resistances: Vec<String> = definition
            .resistances
            .iter()
            .map(|resistance| catalog.text(resistance_key(*resistance)))
            .collect();
        lines.push(catalog.format("enemy.resistances", &[("list", &join(&resistances))]));
    }
    lines
}

/// « Capacités : vif, toxique », s'il en a (carte d'ennemi de l'écran de combat, spec 033).
pub fn token_abilities(token: EnemyToken, catalog: &Catalog) -> Option<String> {
    let abilities: Vec<String> = token
        .enemy()
        .definition()
        .abilities
        .iter()
        .map(|ability| catalog.text(ability_key(*ability)))
        .collect();
    (!abilities.is_empty())
        .then(|| catalog.format("enemy.abilities", &[("list", &join(&abilities))]))
}

/// « armure 3, attaque 4, gloire 2 » (spec 030 ; carte d'ennemi de l'écran de combat, spec 031).
pub fn token_values(token: EnemyToken, catalog: &Catalog) -> String {
    let definition = token.enemy().definition();
    catalog.format(
        "enemy.values",
        &[
            ("armor", &definition.armor.to_string()),
            ("attack", &attack_text(definition.attack, catalog)),
            ("fame", &definition.fame.to_string()),
        ],
    )
}

/// « 4 », « feu 6 », « feu glacé 6 » ou « invocation » : l'élément physique ne s'écrit pas.
fn attack_text(attack: EnemyAttack, catalog: &Catalog) -> String {
    let with = |key: &str, value: u8| format!("{} {value}", catalog.text(key));
    match attack {
        EnemyAttack::Physical(value) => value.to_string(),
        EnemyAttack::Fire(value) => with("enemy.element.fire", value),
        EnemyAttack::Ice(value) => with("enemy.element.ice", value),
        EnemyAttack::ColdFire(value) => with("enemy.element.cold_fire", value),
        EnemyAttack::Summon => catalog.text("enemy.summon"),
    }
}

pub(crate) fn ability_key(ability: EnemyAbility) -> &'static str {
    match ability {
        EnemyAbility::Fortified => "enemy.ability.fortified",
        EnemyAbility::Swift => "enemy.ability.swift",
        EnemyAbility::Brutal => "enemy.ability.brutal",
        EnemyAbility::Poison => "enemy.ability.poison",
        EnemyAbility::Paralyze => "enemy.ability.paralyze",
    }
}

pub(crate) fn resistance_key(resistance: Resistance) -> &'static str {
    match resistance {
        Resistance::Physical => "enemy.element.physical",
        Resistance::Fire => "enemy.element.fire",
        Resistance::Ice => "enemy.element.ice",
    }
}

/// Une liste de mots : la virgule sert en français comme en anglais.
fn join(items: &[String]) -> String {
    items.join(", ")
}
