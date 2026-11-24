# Plan de développement — spec 058 Compétences : second lot

- Statut : accepté (plan de la spec validée le 2026-11-13, Q7 : trois fils).
- Spec : [`058-competences-second-lot.md`](../058-competences-second-lot.md). Branche : `spec/058-competences-second-lot`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 298), celle de la spec.

## Découpage en fils

Trois fils successifs sur la même branche (spec, Q7) :

1. **Moteur 1** : étapes 1 à 4 (groupes A, B, C, Rituel de guérison et son avantage).
2. **Moteur 2** : étapes 5 à 10 (Vol, unités, Botte secrète, centre, vue, retour en arrière, persistance, C1).
3. **Écran** : étapes 11 à 14.

**C1 passe au fil moteur 2** (écart au plan de la spec) : « plus aucune `Unsupported` » ne peut être vert qu'une fois les
groupes D à G codés. Au fil moteur 1, le test `first_batch_skills_are_supported` (spec 051) est mis à jour pour les
compétences codées à chaque étape.

**Contrôle de la vue** (issue privée 130) : les champs de `PlayerView` de la section *Information cachée* sont tous produits au fil
moteur 2 (étape 9, C44) ; l'écran n'en découvre aucun.

## Choix de modèle de données (fil moteur 1)

- Groupe C et Rituel : effets nouveaux, choisis par `skills.ron` (ADR 0005) : `Effect::CrystalColorInfluence` (Fortune
  clinquante), `Effect::MovePerUnit { max }` (Marche en avant), `Effect::DiscardWoundToDraw` (Même pas mal),
  `Effect::ThrowWounds(n)` (Rituel de guérison).
- Groupe A : `SkillEffect::Sideways` dans `skills.ron` ; la carte jouée porte la compétence par une forme nouvelle
  `CardPlay::SidewaysSkill { sideways, skill, option, mana }` (les `CardPlay::Sideways` existants ne changent pas).
- Groupe B : `SkillEffect::Payment` ; deux formes nouvelles de `ManaPayment` : `Invocation(CardId)` et
  `Polarized { from, source }`, proposées par `mana_payments` et `black_payments` là où l'on paie aujourd'hui.
- Centre : `SkillState::InCenter`, événements `SkillPlacedInCenter`, `CenterSkillReturned`, `CenterAdvantageUsed`.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Groupe C : Fortune clinquante (Influence d'interaction), Marche en avant, Même pas mal | C18 à C21 | compétences sans action légale (`Unsupported`) | `skills.ron`, `skill.rs`, `card.rs`, `content.rs`, `engine.rs`, `state.rs` | non | fil principal |
| 2 | Groupe A : cartes de côté avec compétence, Blessure de côté, Magie inutile (plus de dé) | C2 à C9 | aucune `SidewaysSkill` légale | `card.rs`, `ids.rs`, `skill.rs`, `engine.rs` | non | fil principal |
| 3 | Groupe B : Invocation et Polarisation dans les paiements | C10 à C17 | aucun paiement nouveau proposé | `card.rs`, `engine.rs` | non | fil principal |
| 4 | Rituel de guérison : posé au centre, avantage au tour suivant | C36, C37 | Rituel sans action légale | `skill.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 5 | Vol | C22 à C25 | … | `engine.rs` | non | fil principal |
| 6 | Charisme, Loyauté sans faille | C26 à C32 | … | `engine.rs`, `action.rs` | non | fil principal |
| 7 | Botte secrète | C33 à C35 | … | `engine.rs` | non | fil principal |
| 8 | Embrasure de la Source, Temps calme, Surcharge de mana, durée de l'avantage | C38 à C43 | … | `engine.rs` | non | fil principal |
| 9 | Vue, contenu complet | C1, C44 | … | `view.rs`, `skill.rs` | non | fil principal |
| 10 | Retour en arrière, persistance | C45, C46 | … | `runner.rs`, `arcanum-persistance` | non | fil principal |
| 11 | Libellés `fr.json` et `en.json` | — | — | `assets/i18n/*.json` | oui | Haiku |
| 12 | Écran : choix de carte, d'ennemi, de case, de couleur, fenêtre du mana, unités | interface | écrans calculés | `arcanum-ui` | non | fil principal |
| 13 | Parties préparées, vérification à l'œil | Q8 | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 14 | Docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`, PLAN) | — | — | `docs/` | oui | Sonnet |

## Où en est l'implémentation (fin du fil moteur 1, 2026-11-14)

Étapes 1 à 4 faites, vertes (rouges vus : 4, 7, 8 et 2 échecs ; tests dans
`crates/arcanum-core/tests/competences_second_lot.rs`). À savoir pour la suite :

- **Centre** déjà générique (étape 4) : `SkillPlacedInCenter` pour toute coopérative utilisée (`color: None` : Surcharge de
  mana la remplira), `PlayerState::center_advantages` (`CenterAdvantage`, actif au `TurnEnded` qui suit la pose),
  `CenterSkillReturned` émis par `EndTurn` pour les avantages actifs, effacés par `RoundStarted`. L'étape 8 ajoute les trois
  autres avantages et le test C38 (durée, dernier tour de la manche).
- `first_batch_skills_are_supported` (spec 051) compte les compétences déjà codées (`SECOND_BATCH_DONE`) ; l'étape 9 le
  remplace par C1. C24 (`unsupported_skill_cannot_be_used`) et `view_shows_skills` prennent Botte secrète : à retirer ou
  reprendre à l'étape 7 (spec, *Effets sur les specs déjà implémentées*).
- **Fil écran** : arms provisoires à remplacer par les vrais libellés : `effect_text` (`InteractionInfluence`, `MovePerUnit`,
  `CrystalColorInfluence`, `DiscardWoundToDraw`, `ThrowWounds`), `skills.rs` (`InCenter` affiché comme retournée),
  `hand.rs` (`SidewaysSkill` sans bouton), `mana.rs`, `units.rs`, `ruins.rs`, `play_area.rs` (`ManaPayment::Invocation` et
  `::Polarized` sans ligne propre).

## Où en est l'implémentation (fin du fil moteur 2, 2026-11-14)

Étapes 5 à 10 faites, vertes (rouges vus : 4, 7, 3, 5 et 1 échecs ; C38, C45 et C46 verts d'emblée, le comportement venant
des étapes précédentes). Le moteur est complet : plus aucune compétence `Unsupported` (C1, `all_skills_are_supported`).
Tests de la spec 051 repris : C24 retiré (plus de compétence indisponible), `view_shows_skills` et le test UI des compétences
grisées sans leur partie « pas encore disponible ».

Ce que le fil écran trouve :

- **Actions nouvelles** : `UseSkill { target: Option<SkillTarget> }` (`Space` pour Vol, `Enemy` pour Botte secrète, `Color`
  pour Surcharge de mana, `Die` pour Embrasure de la Source) ; `RecruitUnit { bonds }` (Loyauté sans faille) ;
  `ActivateUnit { leadership }` (Charisme) ; `KeepSourceDie` (garder le dé d'Embrasure à la fin du tour, sinon relancé).
- **Vue** (`PlayerView`, issue privée 130) : `SkillView` a `cards` (jeu de côté, Invocation), `units` (Charisme) et `targets` ;
  `center_advantages`, `interaction_influence`, `source_dice_forbidden`, `bonds_unit`. L'écran n'a rien à recalculer.
- **Arms provisoires** à remplacer par de vrais libellés : `text.rs` (`Effect::Flight`, `ResistanceBreak`, `SourceOpening`,
  `CalmWeather`, `ManaOverload`, en plus de ceux du fil moteur 1), `play_area.rs` (`ModifierSource::Skill` affiche le nom
  de la compétence), `offers.rs` et `units.rs` (recrutement : seul `bonds: false` est proposé ; activation : les
  actions `leadership: true` donnent aujourd'hui un second bouton au même libellé, à distinguer), `skills.rs` (cibles à
  choisir).
- Événements nouveaux : `UnitUnderBonds`, `ResistanceBroken`, `SourceDieKept` (non bloquants pour le retour en arrière,
  comme Vol et Polarisation ; Même pas mal et la relance d'Embrasure le sont).

## Où en est l'implémentation (fin du fil écran, 2026-11-14)

Étapes 11 à 14 faites (rouges vus : 5, 8 et 2 échecs ; tests dans `crates/arcanum-ui/tests/competences_second_lot.rs`) :

- **Mode** `ScreenMode::ChoosingSkillTarget(SkillChoice)` (`Command::ChooseSkillTarget`) : la carte jouée de côté (bandeau
  `skill.choose_card`, boutons « Jouer de côté : … » sur les cartes), puis le mana de Pouvoir universel (bandeau du mana), ou la
  cible (case en évidence sur le plateau par `cast_destinations`, ennemi et couleur dans le bandeau, dé dans la Source).
- Au centre : l'option ne sert que l'avantage (« Avantage de … »), l'état grisé disparaît au tour de l'avantage ; avantages
  et « plus de dé » dans les effets en cours ; « Garder ce dé tel quel » près de la Source (`SourceLine::keep`).
- Paiements : `PaymentKind::Invocation` et `::Polarized`, `skill_payment_text` partagé par le choix du mana, les unités et
  l'offrande.
- Charisme : second bouton par option d'unité, valeur par `arcanum_core::leadership_effect` (rendue publique) ; Loyauté :
  `UnitsSection::bonds_token` et « Recruter sous Loyauté : … » (coût par `arcanum_core::bonds_cost`, rendue publique).
- Quatre parties préparées : `competences-cote`, `competences-goldyx`, `competences-norowas`, `competences-arythea`.

## Risques repérés

- Nombre d'actions légales : Invocation multiplie les paiements par les cartes de la main. Parade : une seule Blessure
  proposée (les Blessures sont identiques), une compétence par paiement au plus (une fois par tour).
- Les écrans qui énumèrent `ManaPayment` et `CardPlay` doivent compiler dès le fil moteur 1 : arms minimales, libellés au fil
  écran.
