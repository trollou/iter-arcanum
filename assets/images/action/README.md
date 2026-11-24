# Icônes des boutons d'action du plateau (spec 075)

Les icônes des boutons ronds posés sur les cases (spec 075, « Boutons d'action du plateau ») : même cadre et même trait que
`ui/*.svg` (`viewBox="0 0 64 64"`, contour `#20242A` 2,2). Le bouton lui-même (disque de 36 points, fond sombre, rouge
pour « Engager le combat », bord doré) est dessiné par le code ; l'icône y tient sur environ les deux tiers du diamètre.
Écrites par le code : `scripts/figurines/actions.py` les régénère.

| Fichier | Icône | Boutons |
|---|---|---|
| `swords.svg` | deux épées croisées | Défier ; Engager le combat (sur fond rouge) |
| `speech.svg` | bulle de dialogue | Interagir |
| `purse.svg` | bourse et pièces | Piller |
| `flame.svg` | flamme | Incendier |
| `door.svg` | porte voûtée entrouverte | Entrer |
| `altar.svg` | autel surmonté d'une flamme de mana | Offrande |

Vérifier : `node scripts/figurines/planche-plateau.js sortie.png` (boutons à 36 et 72 points, en bas de la planche).
