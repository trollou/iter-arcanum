//! Actions avancées, sorts et artefacts du jeu de base (spec 050) : identifiants et clés de
//! libellés. Leurs définitions sont dans `content/advanced_actions.ron`, `spells.ron` et
//! `artifacts.ron`.

use serde::Deserialize;

/// Une famille de cartes : l'enum, `ALL` dans l'ordre des numéros imprimés, et les clés de
/// libellés `card.<id>.…` (ADR 0003).
macro_rules! card_family {
    ($(#[$meta:meta])* $name:ident, $count:literal { $($variant:ident => $id:literal,)* }) => {
        $(#[$meta])*
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
        )]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub const ALL: [$name; $count] = [$($name::$variant,)*];

            /// Clé du nom de la carte (pour un sort : le nom de l'effet de base).
            pub fn label_key(self) -> &'static str {
                match self {
                    $($name::$variant => concat!("card.", $id, ".name"),)*
                }
            }

            /// Clé du nom de l'effet renforcé : seulement pour un sort, dont chaque moitié a son nom
            /// (MKUE p. 4, 3.b).
            pub fn strong_name_key(self) -> &'static str {
                match self {
                    $($name::$variant => concat!("card.", $id, ".strong_name"),)*
                }
            }

            /// Clé du texte écrit par nous d'un effet de base non codé (spec 050, Q1).
            pub fn basic_text_key(self) -> &'static str {
                match self {
                    $($name::$variant => concat!("card.", $id, ".basic_text"),)*
                }
            }

            /// Clé du texte écrit par nous d'un effet renforcé non codé (spec 050, Q1).
            pub fn strong_text_key(self) -> &'static str {
                match self {
                    $($name::$variant => concat!("card.", $id, ".strong_text"),)*
                }
            }
        }
    };
}

card_family! {
    /// Les 28 actions avancées, dans l'ordre des numéros n/28 (MKUE p. 3).
    AdvancedAction, 28 {
        FireBolt => "fire_bolt",
        IceBolt => "ice_bolt",
        SwiftBolt => "swift_bolt",
        CrushingBolt => "crushing_bolt",
        BloodRage => "blood_rage",
        IceShield => "ice_shield",
        Agility => "agility",
        RefreshingWalk => "refreshing_walk",
        Intimidate => "intimidate",
        FrostBridge => "frost_bridge",
        SongOfWind => "song_of_wind",
        PathFinding => "path_finding",
        BloodRitual => "blood_ritual",
        PureMagic => "pure_magic",
        HeroicTale => "heroic_tale",
        Regeneration => "regeneration",
        IntoTheHeat => "into_the_heat",
        SteadyTempo => "steady_tempo",
        Diplomacy => "diplomacy",
        InNeed => "in_need",
        Decompose => "decompose",
        CrystalMastery => "crystal_mastery",
        ManaStorm => "mana_storm",
        Ambush => "ambush",
        MaximalEffect => "maximal_effect",
        MagicTalent => "magic_talent",
        Learning => "learning",
        Training => "training",
    }
}

card_family! {
    /// Les 20 sorts, dans l'ordre des numéros n/20 ; nommés par leur effet de base (MKUE p. 4, 3.b).
    Spell, 20 {
        Fireball => "fireball",
        Snowstorm => "snowstorm",
        Expose => "expose",
        Tremor => "tremor",
        FlameWall => "flame_wall",
        ManaBolt => "mana_bolt",
        Whirlwind => "whirlwind",
        UndergroundTravel => "underground_travel",
        BurningShield => "burning_shield",
        Chill => "chill",
        WingsOfWind => "wings_of_wind",
        Restoration => "restoration",
        Demolish => "demolish",
        SpaceBending => "space_bending",
        CallToArms => "call_to_arms",
        Meditation => "meditation",
        ManaMeltdown => "mana_meltdown",
        ManaClaim => "mana_claim",
        MindRead => "mind_read",
        EnergyFlow => "energy_flow",
    }
}

impl Spell {
    /// Les quatre sorts compétitifs, ceux qui visent les autres joueurs (17/20 à 20/20 ; MKUE
    /// p. 19, *Cards and Skills* : retirés en Conquête en solitaire, spec 064).
    pub fn is_competitive(self) -> bool {
        matches!(
            self,
            Spell::ManaMeltdown | Spell::ManaClaim | Spell::MindRead | Spell::EnergyFlow
        )
    }
}

card_family! {
    /// Les 16 artefacts, dans l'ordre des numéros n/16 (MKUE p. 4, 3.c).
    Artifact, 16 {
        BannerOfGlory => "banner_of_glory",
        BannerOfFear => "banner_of_fear",
        BannerOfProtection => "banner_of_protection",
        BannerOfCourage => "banner_of_courage",
        RubyRing => "ruby_ring",
        SapphireRing => "sapphire_ring",
        DiamondRing => "diamond_ring",
        EmeraldRing => "emerald_ring",
        SwordOfJustice => "sword_of_justice",
        HornOfWrath => "horn_of_wrath",
        EndlessBagOfGold => "endless_bag_of_gold",
        EndlessGemPouch => "endless_gem_pouch",
        GoldenGrail => "golden_grail",
        BookOfWisdom => "book_of_wisdom",
        AmuletOfSun => "amulet_of_sun",
        AmuletOfDarkness => "amulet_of_darkness",
    }
}

impl Artifact {
    /// Une des quatre bannières, qui s'assignent à une unité (MKUE p. 5, 8 ; spec 055).
    pub fn is_banner(self) -> bool {
        matches!(
            self,
            Artifact::BannerOfGlory
                | Artifact::BannerOfFear
                | Artifact::BannerOfProtection
                | Artifact::BannerOfCourage
        )
    }
}
