//! Migration des sauvegardes (spec #52) : une sauvegarde de l'une des trois dernières versions
//! publiées se charge avec la version courante. Chaque migration transforme le JSON entier d'une
//! sauvegarde du format N en JSON du format N+1, sans horloge, fichier ni contenu du jeu.

use serde_json::Value;

use crate::LoadError;

/// Une migration du format N au format N+1, ou la raison pour laquelle la sauvegarde ne se
/// migre pas.
pub type Migration = fn(Value) -> Result<Value, String>;

/// Format de la plus ancienne des trois dernières versions publiées (v0.1.0). Ne change qu'à la
/// release (`docs/release.md`).
pub const OLDEST_SUPPORTED_FORMAT: u16 = 1;

/// Format de la dernière version publiée (v0.1.0). Ne change qu'à la release.
pub const PREVIOUS_RELEASE_FORMAT: u16 = 1;

/// La chaîne, la première migration partant de `OLDEST_SUPPORTED_FORMAT` ; vide tant qu'aucun
/// changement incompatible n'a été fait.
pub const MIGRATIONS: &[Migration] = &[];

/// Les formats que `load` sait lire : les vrais (`Formats::SUPPORTED`), ou ceux d'un test.
#[derive(Clone, Copy, Debug)]
pub struct Formats {
    pub oldest: u16,
    pub current: u16,
    /// `current - oldest` migrations, la première partant de `oldest`.
    pub chain: &'static [Migration],
}

impl Formats {
    pub const SUPPORTED: Formats = Formats {
        oldest: OLDEST_SUPPORTED_FORMAT,
        current: arcanum_core::EVENT_FORMAT_VERSION,
        chain: MIGRATIONS,
    };

    /// Amène le JSON d'une sauvegarde au format courant : refuse un format hors de la fenêtre,
    /// applique les migrations depuis son format, et lui donne le numéro courant. Renvoie aussi
    /// le format lu, pour le journal.
    pub fn migrate(&self, mut value: Value) -> Result<(Value, u16), LoadError> {
        let found = read_version(&value)?;
        if found > self.current {
            return Err(LoadError::NewerVersion { found });
        }
        if found == 0 || found < self.oldest {
            return Err(LoadError::TooOld { found });
        }
        let first = usize::from(found - self.oldest);
        for (step, migration) in (found..).zip(self.chain.iter().skip(first)) {
            value =
                migration(value).map_err(|reason| LoadError::NotMigratable { found, reason })?;
            value["version"] = Value::from(step + 1);
        }
        Ok((value, found))
    }
}

/// Le numéro de format du fichier, lu avant le reste pour un message clair.
fn read_version(value: &Value) -> Result<u16, LoadError> {
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| LoadError::Invalid("version absente".to_string()))?;
    Ok(u16::try_from(version).unwrap_or(u16::MAX))
}
