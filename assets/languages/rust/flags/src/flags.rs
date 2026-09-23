//! This service's feature flags, and the only place this side reads one.
//!
//! A flag is what makes merging and releasing two decisions. Every commit that passes `verify` on `main`
//! reaches production, so work that is not finished has to arrive there dark: the branch is merged, the flag
//! is off, and nobody outside sees it until somebody flips it. `.specify/memory/constitution.md` requires
//! exactly that, and this module is where the requirement stops being prose.
//!
//! # One flag, one name
//!
//! A flag is declared once, in `infra/service/flags.auto.tfvars`, under the service that reads it, with a key
//! in one spelling: `checkout-v2`. Ask by key and the declaration and the code cannot drift apart — which is
//! the whole reason a slice calls `flags::enabled("checkout-v2")` and never reaches for a variable name of its
//! own. A hand-derived variable is a typo waiting to happen, and a typo reads as absent, which reads as off.
//!
//! # Where a value comes from is one value, and it is not this function
//!
//! A [`Source`] answers two questions and no others: what this environment holds for one key, and what it
//! holds for all of them. Today [`default_source`] returns the process environment: the stack turns each key
//! into a parameter resolved into this container's environment as `FLAG_CHECKOUT_V2` — upper-cased, dashes to
//! underscores. [`variable`] is that transform and [`key`] its inverse, both written here and nowhere else. A
//! transport that is not an environment is a second `Source` and a one-line change to `default_source`, not a
//! change to any call site.
//!
//! # Off is the answer to every question this cannot answer
//!
//! A flag is on only for the exact value `on` — the spelling `make flag` writes. `off`, `true`, `1`, a value
//! that never arrived: all off. That is the safe direction, and it matters most between merging code that
//! reads a flag and the apply that creates its parameter: the feature starts quietly off rather than the
//! service failing to start.
//!
//! # The seam exists so both paths can be tested
//!
//! A test drives either path by passing a source — `fixed_source([("checkout-v2", "on")])` for the on path
//! and `fixed_source([])` for the one production runs while the flag is off. `make check-flags` holds every
//! declared flag to having both paths covered. `std::env` is read here and in no other module.
//!
//! Locally there is no parameter store: the flag is whatever this process's environment says, so
//! `FLAG_CHECKOUT_V2=on make dev` is the whole of it.

use std::collections::BTreeMap;

/// The only value that turns a flag on. Anything else, or nothing at all, gates its feature shut.
pub const ON: &str = "on";

/// What the stack gives every flag variable, and what [`key`] will answer to.
const PREFIX: &str = "FLAG_";

/// Where this service's flags come from, keyed by the flag's key and never by a variable name.
pub trait Source {
    /// This environment's raw value for one flag, or the empty string when it carries none.
    fn value(&self, key: &str) -> String;

    /// Every flag this source carries, by key, as of now — a snapshot, never a subscription. Only flags with a
    /// value appear: an absent flag is off, and saying so by omission is the answer `value` gives.
    fn snapshot(&self) -> BTreeMap<String, String>;
}

/// The environment variable a flag's key is read from: `checkout-v2` becomes `FLAG_CHECKOUT_V2`.
pub fn variable(key: &str) -> String {
    format!("{PREFIX}{}", key.replace('-', "_").to_uppercase())
}

/// The key a flag variable came from: `FLAG_CHECKOUT_V2` becomes `checkout-v2`, and a variable that is not a
/// flag's has none.
///
/// Exact only because a key may not contain an underscore — `check-flags.py` holds every declared key to
/// `[a-z0-9][a-z0-9-]*` — so every underscore in the variable came from a dash. Anchored on the prefix, which
/// is why `VITE_FLAG_CHECKOUT_V2`, the browser's spelling of the same flag, is not a flag variable here.
pub fn key(variable: &str) -> Option<String> {
    variable
        .strip_prefix(PREFIX)
        .map(|rest| rest.replace('_', "-").to_lowercase())
}

fn flags_of<'a>(variables: impl Iterator<Item = (&'a str, &'a str)>) -> BTreeMap<String, String> {
    variables
        .filter_map(|(name, value)| key(name).map(|flag| (flag, value.to_owned())))
        .collect()
}

struct ProcessEnvironment;

impl Source for ProcessEnvironment {
    fn value(&self, key: &str) -> String {
        std::env::var(variable(key)).unwrap_or_default()
    }

    fn snapshot(&self) -> BTreeMap<String, String> {
        let variables: Vec<(String, String)> = std::env::vars().collect();
        flags_of(
            variables
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str())),
        )
    }
}

struct EnvironmentSource(BTreeMap<String, String>);

impl Source for EnvironmentSource {
    fn value(&self, key: &str) -> String {
        self.0.get(&variable(key)).cloned().unwrap_or_default()
    }

    fn snapshot(&self) -> BTreeMap<String, String> {
        flags_of(
            self.0
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str())),
        )
    }
}

struct FixedSource(BTreeMap<String, String>);

impl Source for FixedSource {
    fn value(&self, key: &str) -> String {
        self.0.get(key).cloned().unwrap_or_default()
    }

    fn snapshot(&self) -> BTreeMap<String, String> {
        self.0.clone()
    }
}

/// A source over this process's own environment — the only one that knows a flag is carried under a
/// different name than the one it is declared with.
pub fn process_environment() -> impl Source {
    ProcessEnvironment
}

/// A source over the given variables. A key the map does not carry is off, exactly as an unset variable is.
pub fn environment_source<'a>(
    variables: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> impl Source {
    EnvironmentSource(
        variables
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
    )
}

/// A source over flags held by key — the key itself, so a test never spells a variable name.
pub fn fixed_source<'a>(values: impl IntoIterator<Item = (&'a str, &'a str)>) -> impl Source {
    FixedSource(
        values
            .into_iter()
            .map(|(flag, value)| (flag.to_owned(), value.to_owned()))
            .collect(),
    )
}

/// Where this service's flags come from — the one line a change of transport is.
pub fn default_source() -> impl Source {
    process_environment()
}

/// Whether the named flag is on, read through this service's own source.
pub fn enabled(key: &str) -> bool {
    enabled_in(key, &default_source())
}

/// Whether the named flag is on in the given source, which is what a test drives both paths with.
pub fn enabled_in(key: &str, source: &impl Source) -> bool {
    source.value(key) == ON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_spells_a_flag_the_way_the_stack_does() {
        // The same transform as `FLAG_${upper(replace(flag.key, "-", "_"))}` in infra/service/flags.tf. If this
        // changes the flag stops arriving — and an absent flag reads as off, so nothing fails loudly.
        assert_eq!(variable("checkout-v2"), "FLAG_CHECKOUT_V2");
        assert_eq!(variable("publish-table"), "FLAG_PUBLISH_TABLE");
    }

    #[test]
    fn key_is_the_exact_inverse_of_variable() {
        for flag in ["checkout-v2", "publish-table", "a", "b2b-invoicing-v10"] {
            assert_eq!(key(&variable(flag)).as_deref(), Some(flag));
        }
    }

    #[test]
    fn a_variable_that_is_not_a_flag_has_no_key() {
        for name in ["DATABASE_URL", "PGSSLMODE", "VITE_FLAG_CHECKOUT_V2"] {
            assert_eq!(key(name), None, "{name}");
        }
    }

    #[test]
    fn an_environment_source_reads_a_key_under_the_name_the_stack_gives_it() {
        assert_eq!(
            environment_source([("FLAG_CHECKOUT_V2", "on")]).value("checkout-v2"),
            "on"
        );
        assert_eq!(environment_source([]).value("checkout-v2"), "");
    }

    #[test]
    fn an_environment_source_snapshots_every_flag_and_nothing_that_is_not_one() {
        let source = environment_source([
            ("FLAG_CHECKOUT_V2", "on"),
            ("FLAG_PUBLISH_TABLE", "off"),
            ("DATABASE_URL", "postgres://nope"),
            ("VITE_FLAG_CHECKOUT_V2", "on"),
        ]);
        let want: BTreeMap<String, String> = [("checkout-v2", "on"), ("publish-table", "off")]
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .into();
        assert_eq!(source.snapshot(), want);
    }

    #[test]
    fn a_fixed_source_is_keyed_by_the_flag_key() {
        assert_eq!(
            fixed_source([("checkout-v2", "on")]).value("checkout-v2"),
            "on"
        );
        assert_eq!(fixed_source([]).value("checkout-v2"), "");
        let want: BTreeMap<String, String> = [("checkout-v2".to_owned(), "on".to_owned())].into();
        assert_eq!(fixed_source([("checkout-v2", "on")]).snapshot(), want);
    }

    #[test]
    fn the_default_source_is_the_transport_this_service_has() {
        // Asked for a key no project would declare, so somebody who exported a real flag locally does not fail
        // this suite by doing so.
        assert_eq!(default_source().value("no-flag-sets-this"), "");
    }

    #[test]
    fn the_process_environment_snapshot_carries_only_flags() {
        // Reads the real environment without writing to it: `set_var` is unsafe in edition 2024, and this
        // crate forbids unsafe code. Whatever flags this process happens to carry, nothing that is not one
        // may appear — `PATH` is always set and never a flag.
        let snapshot = process_environment().snapshot();
        assert!(!snapshot.contains_key("path"));
        assert!(snapshot.keys().all(|flag| !flag.contains('_')));
    }

    #[test]
    fn a_flag_is_on_only_for_the_value_make_flag_writes() {
        assert!(enabled_in(
            "checkout-v2",
            &fixed_source([("checkout-v2", "on")])
        ));
        assert!(!enabled_in(
            "checkout-v2",
            &fixed_source([("checkout-v2", "off")])
        ));
    }

    #[test]
    fn a_flag_nothing_set_is_off() {
        assert!(!enabled_in("checkout-v2", &fixed_source([])));
        assert!(!enabled("no-flag-sets-this"));
    }

    #[test]
    fn a_value_the_reader_does_not_understand_is_off() {
        for value in ["true", "1", "ON", "yes"] {
            assert!(
                !enabled_in("checkout-v2", &fixed_source([("checkout-v2", value)])),
                "{value}"
            );
        }
    }

    #[test]
    fn a_flag_reads_through_whichever_source_it_is_given() {
        assert!(enabled_in(
            "checkout-v2",
            &environment_source([("FLAG_CHECKOUT_V2", "on")])
        ));
        assert!(!enabled_in("checkout-v2", &environment_source([])));
    }
}
