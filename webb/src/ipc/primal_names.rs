// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical primal names and display formatting.
//!
//! Absorbed from primalSpring `primal_names.rs` (neuralSpring S170 pattern).
//! Discovery uses lowercase slugs (`squirrel`, `rhizocrypt`); handoffs and
//! dashboards use mixed-case display names (`Squirrel`, `rhizoCrypt`).
//!
//! Single source of truth — no duplicated name tables across modules.

/// Discovery slug for Squirrel.
pub const SQUIRREL: &str = "squirrel";
/// Discovery slug for petalTongue.
pub const PETALTONGUE: &str = "petaltongue";
/// Discovery slug for `ToadStool`.
pub const TOADSTOOL: &str = "toadstool";
/// Discovery slug for `NestGate`.
pub const NESTGATE: &str = "nestgate";
/// Discovery slug for ludoSpring.
pub const LUDOSPRING: &str = "ludospring";
/// Discovery slug for rhizoCrypt.
pub const RHIZOCRYPT: &str = "rhizocrypt";
/// Discovery slug for `LoamSpine`.
pub const LOAMSPINE: &str = "loamspine";
/// Discovery slug for sweetGrass.
pub const SWEETGRASS: &str = "sweetgrass";

/// Domain identifiers for capability-based discovery.
pub mod domain {
    /// AI domain (Squirrel).
    pub const AI: &str = "ai";
    /// Visualization domain (petalTongue).
    pub const VISUALIZATION: &str = "visualization";
    /// Compute domain (toadStool).
    pub const COMPUTE: &str = "compute";
    /// Storage domain (nestGate).
    pub const STORAGE: &str = "storage";
    /// Game science domain (ludoSpring).
    pub const GAME: &str = "game";
    /// DAG domain (rhizoCrypt).
    pub const DAG: &str = "dag";
    /// Lineage domain (loamSpine).
    pub const LINEAGE: &str = "lineage";
    /// Provenance domain (sweetGrass).
    pub const PROVENANCE: &str = "provenance";
}

/// Domain-to-default-primal mapping for discovery.
///
/// The bridge discovers by domain and uses names only for logging.
/// Primal code only has self-knowledge — these names come from the
/// ecosystem registry, not from importing primal code.
pub const DOMAIN_PRIMAL_MAP: &[(&str, &str)] = &[
    (domain::AI, SQUIRREL),
    (domain::VISUALIZATION, PETALTONGUE),
    (domain::COMPUTE, TOADSTOOL),
    (domain::STORAGE, NESTGATE),
    (domain::GAME, LUDOSPRING),
    (domain::DAG, RHIZOCRYPT),
    (domain::LINEAGE, LOAMSPINE),
    (domain::PROVENANCE, SWEETGRASS),
];

/// Lowercase discovery slug → mixed-case display name.
///
/// Returns the display name for known ecosystem primals, or passes
/// through the input unchanged for unknown names.
#[must_use]
pub fn display_name(slug: &str) -> &str {
    match slug {
        "squirrel" => "Squirrel",
        "petaltongue" => "petalTongue",
        "toadstool" => "ToadStool",
        "nestgate" => "NestGate",
        "ludospring" => "ludoSpring",
        "rhizocrypt" => "rhizoCrypt",
        "loamspine" => "LoamSpine",
        "sweetgrass" => "sweetGrass",
        "biomeos" => "biomeOS",
        "beardog" => "BearDog",
        "songbird" => "Songbird",
        "barracuda" => "barraCuda",
        "coralreef" => "coralReef",
        "fieldmouse" => "fieldMouse",
        "sourdough" => "sourDough",
        "esotericwebb" => "esotericWebb",
        _ => slug,
    }
}

/// Mixed-case display name → lowercase discovery slug.
///
/// Inverse of [`display_name`]. For known primals, returns the canonical
/// lowercase slug used in socket paths and environment variable lookups.
#[must_use]
pub fn discovery_slug(display: &str) -> &str {
    match display {
        "Squirrel" => "squirrel",
        "petalTongue" => "petaltongue",
        "ToadStool" => "toadstool",
        "NestGate" => "nestgate",
        "ludoSpring" => "ludospring",
        "rhizoCrypt" => "rhizocrypt",
        "LoamSpine" => "loamspine",
        "sweetGrass" => "sweetgrass",
        "biomeOS" => "biomeos",
        "BearDog" => "beardog",
        "Songbird" => "songbird",
        "barraCuda" => "barracuda",
        "coralReef" => "coralreef",
        "fieldMouse" => "fieldmouse",
        "sourDough" => "sourdough",
        "esotericWebb" => "esotericwebb",
        _ => display,
    }
}

/// Look up the default primal name for a given domain.
#[must_use]
pub fn primal_for_domain(domain: &str) -> Option<&'static str> {
    DOMAIN_PRIMAL_MAP
        .iter()
        .find(|&&(d, _)| d == domain)
        .map(|&(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_primals_have_display_names() {
        assert_eq!(display_name("squirrel"), "Squirrel");
        assert_eq!(display_name("rhizocrypt"), "rhizoCrypt");
        assert_eq!(display_name("ludospring"), "ludoSpring");
        assert_eq!(display_name("esotericwebb"), "esotericWebb");
    }

    #[test]
    fn unknown_slug_passes_through() {
        assert_eq!(display_name("unknown_primal"), "unknown_primal");
    }

    #[test]
    fn display_to_slug_round_trips() {
        let slugs = [
            SQUIRREL,
            PETALTONGUE,
            TOADSTOOL,
            NESTGATE,
            LUDOSPRING,
            RHIZOCRYPT,
            LOAMSPINE,
            SWEETGRASS,
        ];
        for slug in slugs {
            let display = display_name(slug);
            let back = discovery_slug(display);
            assert_eq!(
                back, slug,
                "round-trip failed for {slug} -> {display} -> {back}"
            );
        }
    }

    #[test]
    fn domain_primal_map_covers_all_domains() {
        assert_eq!(DOMAIN_PRIMAL_MAP.len(), 8);
        assert_eq!(primal_for_domain(domain::AI), Some(SQUIRREL));
        assert_eq!(primal_for_domain(domain::DAG), Some(RHIZOCRYPT));
        assert_eq!(primal_for_domain(domain::GAME), Some(LUDOSPRING));
        assert_eq!(primal_for_domain("nonexistent"), None);
    }

    #[test]
    fn unknown_display_passes_through() {
        assert_eq!(discovery_slug("UnknownPrimal"), "UnknownPrimal");
    }
}
