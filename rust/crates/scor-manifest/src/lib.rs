//! SPDX-FileCopyrightText: 2026 The Tessera Project
//! SPDX-License-Identifier: Apache-2.0

//! Spoke manifest validation.
//!
//! One rule carries the whole independence guarantee:
//!
//! > A hard `requires` entry may name a hub service and nothing else.
//!
//! If a spoke could hard-require another spoke, disabling that other spoke
//! would cascade, and the promise that any organisation can switch off the
//! spokes it does not need would be false. Everything else in this module
//! exists to make that rule impossible to sidestep.
//!
//! Behaviour is held to `reference/python/tests/test_manifest.py`; the
//! Rust and Python validators must stay in lockstep.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// Severity of a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Blocks install.
    Error,
    /// Reported, does not block.
    Warning,
}

impl Severity {
    /// Stable machine spelling, matching the Python reference output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// A single validation finding. Validators return all of them, never just
/// the first, so one pass fixes the whole manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Error or warning.
    pub severity: Severity,
    /// Stable machine code, e.g. `requires.spoke_dependency`.
    pub code: String,
    /// Human message that names the fix, not just the problem.
    pub message: String,
}

impl Finding {
    /// Render like the Python reference's `Finding.__str__`.
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "[{}] {}: {}",
            self.severity.as_str(),
            self.code,
            self.message
        )
    }
}

/// The complete set of hub services a spoke may hard-require. Extending
/// this list is an origin-level decision, not a spoke's.
pub const HUB_SERVICES: &[&str] = &[
    "hub.origin",
    "hub.access",
    "hub.master_data",
    "hub.ingest",
    "hub.ledger",
    "hub.events",
    "hub.plugin_host",
    "hub.tenancy",
    "hub.ai_core",
];

/// Registered spoke codes. A manifest for an unregistered code is rejected
/// so a typo cannot create a shadow spoke.
pub const REGISTERED_SPOKES: &[&str] = &[
    "pln", "src", "trf", "ord", "ful", "ret", "inv", "srm", "ctr", "tsk", "prj",
];

/// Missing-value policies a `consumes` entry must declare for cross-spoke
/// reads.
pub const MISSING_POLICIES: &[&str] = &["default", "fail", "hold_last", "null"];

/// Agent capability tiers, least to most privileged.
pub const AGENT_TIERS: &[&str] = &["act", "advise", "observe"];

/// The findings for one manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationResult {
    /// Every finding, in emission order.
    pub findings: Vec<Finding>,
}

impl ValidationResult {
    fn error(&mut self, code: &str, message: String) {
        self.findings.push(Finding {
            severity: Severity::Error,
            code: code.to_string(),
            message,
        });
    }

    fn warn(&mut self, code: &str, message: String) {
        self.findings.push(Finding {
            severity: Severity::Warning,
            code: code.to_string(),
            message,
        });
    }

    /// Findings with error severity.
    #[must_use]
    pub fn errors(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .collect()
    }

    /// Findings with warning severity.
    #[must_use]
    pub fn warnings(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .collect()
    }

    /// True when nothing blocks install.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.errors().is_empty()
    }
}

/// `^[a-z][a-z0-9_]{1,15}\Z` — lowercase snake_case, 2 to 16 characters.
fn is_code(s: &str) -> bool {
    let bytes = s.as_bytes();
    let Some(first) = bytes.first() else {
        return false;
    };
    if !first.is_ascii_lowercase() || bytes.len() < 2 || bytes.len() > 16 {
        return false;
    }
    bytes.get(1..).is_some_and(|rest| {
        rest.iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
    })
}

/// `^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?\Z` — semver with optional
/// prerelease. ASCII digits only, matching the hardened Python regexes.
fn is_semver(s: &str) -> bool {
    let (core, pre) = match s.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (s, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    let numeric_core = parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    if !numeric_core {
        return false;
    }
    match pre {
        None => true,
        Some(p) => {
            !p.is_empty()
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        }
    }
}

/// `^[a-z][a-z0-9_]{1,15}\.[a-z][a-z0-9_.]*\Z` — namespaced object names.
fn is_namespaced(s: &str) -> bool {
    let (head, tail) = match s.split_once('.') {
        Some((h, t)) => (h, t),
        None => return false,
    };
    is_code(head)
        && starts_lowercase(tail)
        && tail
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.')
}

fn starts_lowercase(s: &str) -> bool {
    s.as_bytes().first().is_some_and(|b| b.is_ascii_lowercase())
}

/// `^[a-z][a-z0-9-]{1,39}$` — dashboard slugs, kebab-case.
fn is_slug(s: &str) -> bool {
    let bytes = s.as_bytes();
    let Some(first) = bytes.first() else {
        return false;
    };
    if !first.is_ascii_lowercase() || bytes.len() < 2 || bytes.len() > 40 {
        return false;
    }
    bytes.get(1..).is_some_and(|rest| {
        rest.iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
    })
}

fn as_str(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str)
}

fn as_array(v: Option<&Value>) -> Option<&[Value]> {
    v.and_then(Value::as_array).map(Vec::as_slice)
}

fn as_object(v: Option<&Value>) -> Option<&serde_json::Map<String, Value>> {
    v.and_then(Value::as_object)
}

/// Python truthiness for JSON values, used where the reference relies on
/// `or []` and `is_truthy`-style checks.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Validate a single spoke manifest, returning every finding.
#[must_use]
pub fn validate_manifest(manifest: &Value) -> ValidationResult {
    let mut result = ValidationResult::default();

    let spoke_raw = as_str(manifest.get("spoke")).unwrap_or("");
    let mut spoke = spoke_raw.to_string();
    if !is_code(spoke_raw) {
        result.error(
            "spoke.code",
            format!("spoke code {spoke_raw:?} must be lowercase snake_case"),
        );
        spoke = String::new();
    } else if !REGISTERED_SPOKES.contains(&spoke_raw) {
        result.error(
            "spoke.unregistered",
            format!("spoke {spoke_raw:?} is not in the register"),
        );
    }

    let version = as_str(manifest.get("version")).unwrap_or("");
    if !is_semver(version) {
        result.error(
            "spoke.version",
            format!("version {version:?} is not semver"),
        );
    }

    // Python's call sites apply defaults here: `manifest.get("requires", [])`
    // and friends. A missing key behaves as an empty collection — only a
    // value that is present but of the wrong type is a type error.
    let empty_arr = Value::Array(Vec::new());
    let empty_obj = Value::Object(serde_json::Map::new());
    let requires = manifest.get("requires").unwrap_or(&empty_arr);
    let enhances = manifest.get("enhances").unwrap_or(&empty_arr);
    let provides = manifest.get("provides").unwrap_or(&empty_obj);
    let consumes = manifest.get("consumes").unwrap_or(&empty_arr);
    let dashboards = manifest.get("dashboards").unwrap_or(&empty_arr);
    let ai = manifest.get("ai");

    validate_requires(requires, &mut result);
    validate_enhances(enhances, &spoke, &mut result);
    validate_provides(provides, &spoke, &mut result);
    validate_consumes(consumes, &spoke, &mut result);
    validate_ai(ai, &spoke, Some(requires), &mut result);
    validate_dashboards(dashboards, &spoke, provides, &mut result);

    result
}

fn validate_requires(requires: &Value, result: &mut ValidationResult) {
    let Some(entries) = as_array(Some(requires)) else {
        result.error("requires.type", "requires must be a list".to_string());
        return;
    };
    if entries.is_empty() {
        result.warn(
            "requires.empty",
            "spoke declares no hub services; is that intentional?".to_string(),
        );
    }
    for entry in entries {
        let Some(text) = entry.as_str() else {
            result.error(
                "requires.type",
                format!("requires entry {entry:?} must be a string"),
            );
            continue;
        };
        if text.starts_with("spoke.") {
            result.error(
                "requires.spoke_dependency",
                format!(
                    "{text:?} is a hard dependency on another spoke, which would break \
                     independent disabling; move it to 'enhances'"
                ),
            );
            continue;
        }
        if !HUB_SERVICES.contains(&text) {
            let mut allowed = HUB_SERVICES.to_vec();
            allowed.sort_unstable();
            result.error(
                "requires.unknown_service",
                format!(
                    "{text:?} is not a hub service; allowed values are {}",
                    allowed.join(", ")
                ),
            );
        }
    }
}

fn validate_enhances(enhances: &Value, spoke: &str, result: &mut ValidationResult) {
    let Some(entries) = as_array(Some(enhances)) else {
        result.error("enhances.type", "enhances must be a list".to_string());
        return;
    };
    for entry in entries {
        let Some(text) = entry.as_str() else {
            result.error(
                "enhances.format",
                format!("enhances entry {entry:?} must be 'spoke.<code>'"),
            );
            continue;
        };
        let Some(code) = text.strip_prefix("spoke.") else {
            result.error(
                "enhances.format",
                format!("enhances entry {text:?} must be 'spoke.<code>'"),
            );
            continue;
        };
        if !REGISTERED_SPOKES.contains(&code) {
            result.error(
                "enhances.unregistered",
                format!("spoke {code:?} is not in the register"),
            );
        }
        if code == spoke {
            result.error(
                "enhances.self",
                format!("spoke {spoke:?} cannot enhance itself"),
            );
        }
    }
}

fn validate_provides(provides: &Value, spoke: &str, result: &mut ValidationResult) {
    let Some(map) = as_object(Some(provides)) else {
        result.error("provides.type", "provides must be an object".to_string());
        return;
    };
    for bucket in ["objects", "events", "kpis"] {
        let Some(entries) = as_array(map.get(bucket)) else {
            continue;
        };
        for name in entries {
            let Some(text) = name.as_str() else {
                result.error(
                    "provides.namespace",
                    format!("{bucket} entry {name:?} must be namespaced as '<spoke>.<name>'"),
                );
                continue;
            };
            if !is_namespaced(text) {
                result.error(
                    "provides.namespace",
                    format!("{bucket} entry {text:?} must be namespaced as '<spoke>.<name>'"),
                );
                continue;
            }
            if !text.starts_with(&format!("{spoke}.")) {
                result.error(
                    "provides.foreign_namespace",
                    format!("{text:?} is published under another spoke's namespace"),
                );
            }
        }
    }
}

fn validate_consumes(consumes: &Value, spoke: &str, result: &mut ValidationResult) {
    let Some(entries) = as_array(Some(consumes)) else {
        result.error("consumes.type", "consumes must be a list".to_string());
        return;
    };
    for entry in entries {
        let Some(map) = entry.as_object() else {
            result.error(
                "consumes.type",
                format!("consumes entry {entry:?} must be an object"),
            );
            continue;
        };
        let field = as_str(map.get("field")).unwrap_or("");
        let policy = as_str(map.get("on_missing"));
        if !is_namespaced(field) {
            result.error(
                "consumes.field",
                format!("consumes field {field:?} must be namespaced"),
            );
            continue;
        }
        let owner = field.split_once('.').map_or("", |(o, _)| o);
        if owner == spoke {
            result.error(
                "consumes.own_namespace",
                format!("{field:?} belongs to this spoke; consumes is for cross-spoke reads only"),
            );
        }
        match policy {
            None => {
                result.error(
                    "consumes.on_missing",
                    format!(
                        "{field:?} must declare on_missing as one of {}",
                        MISSING_POLICIES.join(", ")
                    ),
                );
            }
            Some(p) if !MISSING_POLICIES.contains(&p) => {
                result.error(
                    "consumes.on_missing",
                    format!(
                        "{field:?} must declare on_missing as one of {}",
                        MISSING_POLICIES.join(", ")
                    ),
                );
            }
            Some("fail") => {
                let approved = map.get("origin_approval").is_some_and(truthy);
                if !approved {
                    result.error(
                        "consumes.fail_needs_origin",
                        format!(
                            "{field:?} uses on_missing 'fail', which can break this spoke when \
                             {owner:?} is disabled; it requires recorded origin approval"
                        ),
                    );
                }
            }
            Some(_) => {}
        }
    }
}

fn validate_ai(
    ai: Option<&Value>,
    spoke: &str,
    requires: Option<&Value>,
    result: &mut ValidationResult,
) {
    // Python's `manifest.get("ai")` treats an explicit null the same as a
    // missing key.
    let Some(ai) = ai.filter(|v| !v.is_null()) else {
        result.warn(
            "ai.absent",
            "spoke declares no agent; it will report nothing to the leader".to_string(),
        );
        return;
    };
    let Some(map) = ai.as_object() else {
        result.error("ai.type", "ai must be an object".to_string());
        return;
    };
    if !map.get("enabled").is_some_and(truthy) {
        return;
    }

    let tier = as_str(map.get("tier")).unwrap_or("");
    if !AGENT_TIERS.contains(&tier) {
        let mut sorted = AGENT_TIERS.to_vec();
        sorted.sort_unstable();
        result.error(
            "ai.tier",
            format!("agent tier {tier:?} must be one of {}", sorted.join(", ")),
        );
    }

    let required: BTreeSet<&str> = as_array(requires)
        .map(|entries| entries.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    for service in ["hub.ai_core", "hub.ledger"] {
        if !required.contains(service) {
            result.error(
                "ai.missing_service",
                format!(
                    "an agent-bearing spoke must require {service:?}: the leader cannot \
                     receive signals, and the agent's actions cannot be logged, without it"
                ),
            );
        }
    }

    // Python: allowlist = ai.get("act_allowlist", []) or []; any falsy
    // value (null, false, 0, "", {}) becomes an empty list without an
    // error; only truthy non-lists are malformed; entries are checked in
    // order and the raw list length decides the act-tier and
    // unused-allowlist rules below, even when entries are junk.
    let mut declared_any = false;
    let raw = map.get("act_allowlist");
    if raw.is_some_and(truthy) {
        match raw {
            Some(Value::Array(entries)) => {
                declared_any = !entries.is_empty();
                for entry in entries {
                    let ok = entry
                        .as_str()
                        .is_some_and(|pattern| pattern.starts_with(&format!("{spoke}.")));
                    if !ok {
                        result.error(
                            "ai.foreign_allowlist",
                            format!(
                                "{entry:?} is outside this spoke's namespace; an agent never \
                                 writes another spoke's data"
                            ),
                        );
                    }
                }
            }
            Some(_) => {
                result.error(
                    "ai.act_allowlist",
                    "act_allowlist must be a list of field patterns".to_string(),
                );
            }
            // `raw` is Some: the outer `is_some_and(truthy)` guard passed.
            None => {}
        }
    }

    if tier == "act" {
        if !declared_any {
            result.error(
                "ai.act_needs_allowlist",
                "act-tier agents must declare a non-empty act_allowlist; unbounded \
                 autonomous write is not a supported configuration"
                    .to_string(),
            );
        }
        if !map.get("origin_approval").is_some_and(truthy) {
            result.error(
                "ai.act_needs_origin",
                "act-tier autonomy requires recorded origin approval".to_string(),
            );
        }
    } else if declared_any {
        result.warn(
            "ai.allowlist_unused",
            format!("act_allowlist is declared but tier is {tier:?}, so it has no effect"),
        );
    }
}

fn validate_dashboards(
    dashboards: &Value,
    spoke: &str,
    provides: &Value,
    result: &mut ValidationResult,
) {
    let Some(entries) = as_array(Some(dashboards)) else {
        result.error("dashboards.type", "dashboards must be a list".to_string());
        return;
    };
    let published_kpis: BTreeSet<&str> = as_object(Some(provides))
        .and_then(|p| as_array(p.get("kpis")))
        .map(|kpis| kpis.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for entry in entries {
        let Some(map) = entry.as_object() else {
            result.error(
                "dashboards.type",
                format!("dashboard entry {entry:?} must be an object"),
            );
            continue;
        };
        let Some(slug) = as_str(map.get("slug")) else {
            result.error(
                "dashboards.slug",
                format!("dashboard slug {:?} must be kebab-case", map.get("slug")),
            );
            continue;
        };
        if !is_slug(slug) {
            result.error(
                "dashboards.slug",
                format!("dashboard slug {slug:?} must be kebab-case"),
            );
            continue;
        }
        if !seen.insert(slug) {
            result.error(
                "dashboards.duplicate",
                format!("dashboard slug {slug:?} is declared twice"),
            );
        }
        let Some(kpis) = as_array(map.get("kpis")) else {
            continue;
        };
        for kpi in kpis {
            let Some(name) = kpi.as_str() else {
                continue;
            };
            if published_kpis.contains(name) {
                continue;
            }
            let owner = name.split_once('.').map_or("", |(o, _)| o);
            if owner != spoke {
                result.warn(
                    "dashboards.cross_spoke_kpi",
                    format!(
                        "{name:?} belongs to another spoke; the widget resolves through the \
                         hub and will show as stale when that spoke is unavailable"
                    ),
                );
            } else {
                result.error(
                    "dashboards.unpublished_kpi",
                    format!("{name:?} is not in provides.kpis"),
                );
            }
        }
    }
}

/// Validate a set of manifests and cross-check the spoke graph.
///
/// Results are keyed by spoke name (`<unnamed-N>` when the manifest does
/// not carry a valid code), sorted, mirroring the Python reference.
#[must_use]
pub fn validate_all(manifests: &[Value]) -> BTreeMap<String, ValidationResult> {
    let mut results: BTreeMap<String, ValidationResult> = BTreeMap::new();
    for (i, manifest) in manifests.iter().enumerate() {
        let name = as_str(manifest.get("spoke"))
            .filter(|s| !s.is_empty())
            .map_or_else(|| format!("<unnamed-{i}>"), ToString::to_string);
        results.insert(name, validate_manifest(manifest));
    }

    // provides.objects must be globally unique across installed spokes.
    let mut published: BTreeMap<String, String> = BTreeMap::new();
    for manifest in manifests {
        let spoke = as_str(manifest.get("spoke")).unwrap_or("").to_string();
        let objects = as_object(manifest.get("provides"))
            .and_then(|p| as_array(p.get("objects")))
            .unwrap_or_default();
        for name in objects {
            let Some(name) = name.as_str() else {
                continue;
            };
            if let Some(existing) = published.get(name) {
                if existing != &spoke {
                    if let Some(result) = results.get_mut(&spoke) {
                        result.error(
                            "provides.collision",
                            format!("{name:?} is already published by {existing:?}"),
                        );
                    }
                }
            }
            published.insert(name.to_string(), spoke.clone());
        }
    }

    // consumes of an uninstalled registered spoke get a warning about the
    // missing-value policy that will apply.
    let installed: BTreeSet<&str> = manifests
        .iter()
        .filter_map(|m| as_str(m.get("spoke")))
        .collect();
    for manifest in manifests {
        let spoke = as_str(manifest.get("spoke")).unwrap_or("").to_string();
        let Some(consumes) = as_array(manifest.get("consumes")) else {
            continue;
        };
        for entry in consumes {
            let Some(map) = entry.as_object() else {
                continue;
            };
            let Some(field) = as_str(map.get("field")) else {
                continue;
            };
            let Some((owner, _)) = field.split_once('.') else {
                continue;
            };
            if REGISTERED_SPOKES.contains(&owner) && !installed.contains(owner) {
                let policy = as_str(map.get("on_missing")).unwrap_or("");
                if let Some(result) = results.get_mut(&spoke) {
                    result.warn(
                        "consumes.absent_owner",
                        format!(
                            "{field:?} is owned by {owner:?}, which is not installed; \
                             the {policy:?} policy will apply"
                        ),
                    );
                }
            }
        }
    }

    results
}
