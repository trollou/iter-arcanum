//! Compétences des héros (spec 051) : identifiants, définitions lues dans `content/skills.ron`
//! (ADR 0005), état d'une compétence gagnée.

use serde::Deserialize;

use crate::{
    AdvancedAction, Artifact, BasicAction, DeedCard, Effect, GameContent, Hero, ManaColor, Spell,
    TimeOfDay,
};

/// Les compétences du solo (MKUE p. 3 ; Compendium, *Skill Tokens* S5) : dix par héros, la
/// coopérative gardée, la compétitive retirée.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum Skill {
    // Arythea.
    DarkPaths,
    BurningPower,
    HotSwordsmanship,
    DarkNegotiation,
    DarkFireMagic,
    MotivationArythea,
    PowerOfPain,
    Invocation,
    Polarization,
    RitualOfPain,
    // Goldyx.
    FreezingPower,
    PotionMaking,
    WhiteCrystalCraft,
    GreenCrystalCraft,
    RedCrystalCraft,
    MotivationGoldyx,
    GlitteringFortune,
    Flight,
    UniversalPower,
    SourceOpening,
    // Norowas.
    DaySharpshooting,
    BrightNegotiation,
    Inspiration,
    LeavesInTheWind,
    WhisperInTheTreetops,
    MotivationNorowas,
    ForwardMarch,
    Leadership,
    BondsOfLoyalty,
    CalmingTheWeather,
    // Tovak.
    DoubleTime,
    NightSharpshooting,
    ColdSwordsmanship,
    ShieldMastery,
    MotivationTovak,
    ResistanceBreak,
    IFeelNoPain,
    IDontGiveADamn,
    WhoNeedsMagic,
    ManaOverload,
}

impl Skill {
    pub const ALL: [Skill; 40] = [
        Skill::DarkPaths,
        Skill::BurningPower,
        Skill::HotSwordsmanship,
        Skill::DarkNegotiation,
        Skill::DarkFireMagic,
        Skill::MotivationArythea,
        Skill::PowerOfPain,
        Skill::Invocation,
        Skill::Polarization,
        Skill::RitualOfPain,
        Skill::FreezingPower,
        Skill::PotionMaking,
        Skill::WhiteCrystalCraft,
        Skill::GreenCrystalCraft,
        Skill::RedCrystalCraft,
        Skill::MotivationGoldyx,
        Skill::GlitteringFortune,
        Skill::Flight,
        Skill::UniversalPower,
        Skill::SourceOpening,
        Skill::DaySharpshooting,
        Skill::BrightNegotiation,
        Skill::Inspiration,
        Skill::LeavesInTheWind,
        Skill::WhisperInTheTreetops,
        Skill::MotivationNorowas,
        Skill::ForwardMarch,
        Skill::Leadership,
        Skill::BondsOfLoyalty,
        Skill::CalmingTheWeather,
        Skill::DoubleTime,
        Skill::NightSharpshooting,
        Skill::ColdSwordsmanship,
        Skill::ShieldMastery,
        Skill::MotivationTovak,
        Skill::ResistanceBreak,
        Skill::IFeelNoPain,
        Skill::IDontGiveADamn,
        Skill::WhoNeedsMagic,
        Skill::ManaOverload,
    ];

    /// La définition lue dans `content/skills.ron`.
    pub fn definition(self) -> &'static SkillDefinition {
        GameContent::embedded().skill(self)
    }

    /// Le héros de la compétence.
    pub fn hero(self) -> Hero {
        self.definition().hero
    }

    /// Vrai pour les quatre Motivation (une par héros) : une seule de la fin de son utilisation
    /// à la fin du tour suivant (texte de la compétence).
    pub fn is_motivation(self) -> bool {
        matches!(
            self,
            Skill::MotivationArythea
                | Skill::MotivationGoldyx
                | Skill::MotivationNorowas
                | Skill::MotivationTovak
        )
    }

    /// Identifiant des clés de libellés : `skill.<id>.name`, `skill.<id>.text` (ADR 0003).
    fn id(self) -> &'static str {
        match self {
            Skill::DarkPaths => "dark_paths",
            Skill::BurningPower => "burning_power",
            Skill::HotSwordsmanship => "hot_swordsmanship",
            Skill::DarkNegotiation => "dark_negotiation",
            Skill::DarkFireMagic => "dark_fire_magic",
            Skill::MotivationArythea => "motivation_arythea",
            Skill::PowerOfPain => "power_of_pain",
            Skill::Invocation => "invocation",
            Skill::Polarization => "polarization",
            Skill::RitualOfPain => "ritual_of_pain",
            Skill::FreezingPower => "freezing_power",
            Skill::PotionMaking => "potion_making",
            Skill::WhiteCrystalCraft => "white_crystal_craft",
            Skill::GreenCrystalCraft => "green_crystal_craft",
            Skill::RedCrystalCraft => "red_crystal_craft",
            Skill::MotivationGoldyx => "motivation_goldyx",
            Skill::GlitteringFortune => "glittering_fortune",
            Skill::Flight => "flight",
            Skill::UniversalPower => "universal_power",
            Skill::SourceOpening => "source_opening",
            Skill::DaySharpshooting => "day_sharpshooting",
            Skill::BrightNegotiation => "bright_negotiation",
            Skill::Inspiration => "inspiration",
            Skill::LeavesInTheWind => "leaves_in_the_wind",
            Skill::WhisperInTheTreetops => "whisper_in_the_treetops",
            Skill::MotivationNorowas => "motivation_norowas",
            Skill::ForwardMarch => "forward_march",
            Skill::Leadership => "leadership",
            Skill::BondsOfLoyalty => "bonds_of_loyalty",
            Skill::CalmingTheWeather => "calming_the_weather",
            Skill::DoubleTime => "double_time",
            Skill::NightSharpshooting => "night_sharpshooting",
            Skill::ColdSwordsmanship => "cold_swordsmanship",
            Skill::ShieldMastery => "shield_mastery",
            Skill::MotivationTovak => "motivation_tovak",
            Skill::ResistanceBreak => "resistance_break",
            Skill::IFeelNoPain => "i_feel_no_pain",
            Skill::IDontGiveADamn => "i_dont_give_a_damn",
            Skill::WhoNeedsMagic => "who_needs_magic",
            Skill::ManaOverload => "mana_overload",
        }
    }

    /// Les options d'une compétence de côté (spec 058) : Magie inutile en a deux (+2, +3).
    pub fn sideways_options(self) -> u8 {
        if self == Skill::WhoNeedsMagic { 2 } else { 1 }
    }

    /// La valeur d'une carte jouée de côté avec cette compétence, qui remplace la valeur 1 (texte
    /// des compétences ; Compendium, *Who Needs Magic* S2), ou `None` si elle ne s'applique pas à
    /// cette carte. `mana` : la couleur du mana de Pouvoir universel. Le Rituel de guérison donne
    /// son avantage du centre (spec 058, G).
    pub fn sideways_value(self, option: u8, card: DeedCard, mana: Option<ManaColor>) -> Option<u8> {
        let wound = card.is_wound();
        match (self, option) {
            // Une Blessure, seulement avec Souffrance sublimée ou l'avantage du Rituel (Compendium,
            // *Power of Pain* S2 ; *I Don't Give a Damn* S3).
            (Skill::PowerOfPain, 0) => wound.then_some(2),
            (Skill::RitualOfPain, 0) => wound.then_some(3),
            _ if wound => None,
            // Compendium, *I Don't Give a Damn* S2 : les actions de base du héros valent 2.
            (Skill::IDontGiveADamn, 0) => Some(match card {
                DeedCard::Advanced(_) | DeedCard::Spell(_) | DeedCard::Artifact(_) => 3,
                _ => 2,
            }),
            (Skill::WhoNeedsMagic, 0) => Some(2),
            (Skill::WhoNeedsMagic, 1) => Some(3),
            // Compendium, *Universal Power* S5, S6 : +4 pour une action ou un sort de la couleur du
            // mana ; un artefact ou le noir, +3.
            (Skill::UniversalPower, 0) => {
                let mana = mana?;
                let same = matches!(
                    card,
                    DeedCard::Basic(_) | DeedCard::Advanced(_) | DeedCard::Spell(_)
                ) && card
                    .definition()
                    .is_some_and(|definition| mana == ManaColor::Basic(definition.color));
                Some(if same { 4 } else { 3 })
            }
            _ => None,
        }
    }

    /// Les valeurs que peut donner l'option `option` de cette compétence de côté, triées et sans
    /// doublon, quelle que soit la carte : « De côté : 2 / 3 » quand aucun jeu n'est légal
    /// (issue privée 302).
    pub fn sideways_values(self, option: u8) -> Vec<u8> {
        // Une carte de chaque sorte suffit : la valeur ne dépend que de la sorte et, pour
        // Pouvoir universel, de l'accord entre la couleur du mana et celle de la carte.
        let cards = [
            DeedCard::Wound,
            DeedCard::Basic(BasicAction::March),
            DeedCard::Advanced(AdvancedAction::ALL[0]),
            DeedCard::Spell(Spell::ALL[0]),
            DeedCard::Artifact(Artifact::ALL[0]),
        ];
        let manas = std::iter::once(None).chain(ManaColor::ALL.map(Some));
        let mut values: Vec<u8> = manas
            .flat_map(|mana| {
                cards
                    .iter()
                    .filter_map(move |&card| self.sideways_value(option, card, mana))
            })
            .collect();
        values.sort_unstable();
        values.dedup();
        values
    }

    /// Clé du nom de la compétence.
    pub fn label_key(self) -> String {
        format!("skill.{}.name", self.id())
    }

    /// Clé du texte court de la compétence, écrit par nous (spec 051, Libellés).
    pub fn text_key(self) -> String {
        format!("skill.{}.text", self.id())
    }
}

/// Fréquence d'utilisation (MKUE p. 5, *Using Skills* 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, serde::Serialize)]
pub enum SkillUsage {
    /// Sans symbole : une fois par tour (4.c).
    Turn,
    /// Une fois par manche : retournée jusqu'à la manche suivante (4.a), ou posée au centre pour
    /// une coopérative (4.b, spec 058).
    Round,
    /// Effet permanent (Loyauté sans faille, spec 058).
    Permanent,
}

/// Une option d'une compétence : l'effet de jour, et celui de nuit s'il diffère (texte des
/// compétences ; Compendium, *Day Sharpshooting* S1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillOption {
    pub day: Effect,
    pub night: Option<Effect>,
}

impl SkillOption {
    /// L'effet au moment de la journée où la compétence est utilisée.
    pub fn effect(self, time: TimeOfDay) -> Effect {
        match (time, self.night) {
            (TimeOfDay::Night, Some(night)) => night,
            _ => self.day,
        }
    }
}

/// Ce que fait une compétence : des options au choix, une carte jouée de côté qui vaut plus
/// (spec 058, groupe A), ou rien encore (second lot, spec 058).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillEffect {
    Options(Vec<SkillOption>),
    Sideways,
    /// Un mana payé autrement (Invocation, Polarisation ; spec 058, groupe B).
    Payment,
    /// Un bonus porté par l'activation d'une unité (Charisme, spec 058, groupe E).
    Leadership,
    /// Un effet permanent, sans utilisation (Loyauté sans faille, spec 058, groupe E).
    Permanent,
    Unsupported,
}

/// La définition d'une compétence (`content/skills.ron`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillDefinition {
    pub hero: Hero,
    pub usage: SkillUsage,
    /// Compétence interactive coopérative (casque blanc), gardée en solo.
    pub cooperative: bool,
    /// « Sauf pendant un combat » (icône de la main ; Compendium, *Inspiration* Q1).
    pub out_of_combat: bool,
    pub effect: SkillEffect,
}

impl SkillDefinition {
    /// Les options, vides pour une compétence du second lot.
    pub fn options(&self) -> &[SkillOption] {
        match &self.effect {
            SkillEffect::Options(options) => options,
            SkillEffect::Sideways
            | SkillEffect::Payment
            | SkillEffect::Leadership
            | SkillEffect::Permanent
            | SkillEffect::Unsupported => &[],
        }
    }
}

/// L'état d'une compétence gagnée (MKUE p. 5, *Using Skills* 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SkillState {
    Ready,
    /// Utilisée ce tour : de nouveau prête au tour suivant.
    UsedThisTurn,
    /// Retournée face cachée jusqu'au début de la manche suivante (MKUE p. 4, 2.g).
    Flipped,
    /// Posée au centre (coopérative, MKUE p. 5, 4.b ; spec 058) : son avantage au tour suivant,
    /// puis retournée.
    InCenter,
}

/// L'avantage différé d'une compétence posée au centre (spec 058, G) : attendu jusqu'au tour
/// suivant (`active` faux), puis utilisable pendant ce tour, une fois.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CenterAdvantage {
    pub skill: Skill,
    /// La couleur choisie pour Surcharge de mana.
    pub color: Option<ManaColor>,
    pub active: bool,
    pub used: bool,
}

/// Une compétence gagnée, face visible devant le joueur, et son état.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedSkill {
    pub skill: Skill,
    pub state: SkillState,
}

/// Une compétence gagnée telle que l'écran la montre (spec 051, C26).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillView {
    pub skill: Skill,
    pub state: SkillState,
    /// Les options utilisables maintenant (celles des actions légales `UseSkill`).
    pub usable: Vec<u8>,
    /// Du second lot : se gagne sans servir (spec 058).
    pub unsupported: bool,
    /// Une Motivation bloquée par une autre jusqu'à la fin du tour suivant.
    pub motivation_blocked: bool,
    /// Spec 058 : les cartes de la main qu'elle accompagne maintenant (jeu de côté, Invocation).
    pub cards: Vec<crate::CardId>,
    /// Spec 058 : les unités qui peuvent la recevoir maintenant (Charisme).
    pub units: Vec<crate::UnitCard>,
    /// Spec 058 : ses cibles maintenant (cases, ennemis, couleurs, dés).
    pub targets: Vec<crate::SkillTarget>,
}

/// Une compétence de l'offre commune, et si le joueur peut la prendre maintenant : celle d'un
/// autre héros, pendant un choix de compétence (Compendium, *Skill Tokens* S3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommonSkillView {
    pub skill: Skill,
    pub takeable: bool,
}
