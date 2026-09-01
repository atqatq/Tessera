//! The v0.1 exit criterion, as an executable sentence (E1 / ROADMAP.md):
//!
//!   a person can install the kernel, install `inv`, ingest a CSV of
//!   stock positions, get a safety-stock recommendation, and read the
//!   ledger entry recording it.
//!
//! This test is the criterion, committed failing — on purpose. It runs
//! the user-facing flow through the `tessera` CLI, which does not exist
//! yet; `cargo test -- --ignored` shows it red for exactly that reason.
//! Every commit stays green (Part A6) because the test is `#[ignore]`d
//! with this file as the explanation. At v0.1 the ignore attribute is
//! deleted and the test joins the required suite; CI gains a job that
//! asserts THIS test passes before any v0.1 tag.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::Command;

/// Runs a `tessera` CLI command in the sandbox root, asserting success.
fn tessera(args: &[&str]) -> String {
    let out = Command::new("tessera")
        .args(args)
        .env("TESSERA_HOME", env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("the tessera CLI must exist on PATH at v0.1");
    assert!(
        out.status.success(),
        "`tessera {}` failed:\nstdout: {}\nstderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A minimal stock-positions CSV — the shape the ingest docs promise:
/// one row per echelon, demand statistics, lead times, service level.
const STOCK_CSV: &str = "\
echelon,parent,mean_demand,sd_demand,mean_lead_time,sd_lead_time,service_level
store-1,,50,12,2,0,0.90
store-2,,50,12,2,0,0.90
dc,store-1|store-2,0,0,6,1,0.95
";

#[test]
#[ignore = "v0.1 exit criterion (ROADMAP.md): the tessera CLI does not exist yet. \
            This test is the criterion, committed failing on purpose; it flips to \
            required when the runtime lands."]
fn a_stranger_gets_a_recommendation_and_can_read_its_ledger_entry() {
    let home = env!("CARGO_MANIFEST_DIR");
    let csv = format!("{home}/stock.csv");
    std::fs::write(&csv, STOCK_CSV).expect("write fixture CSV");

    // install the kernel + the inv module into a fresh tenant
    tessera(&["init", "--tenant", "acme"]);
    tessera(&["module", "install", "inv"]);

    // ingest stock positions
    let ingest_out = tessera(&["ingest", "csv", &csv, "--into", "inv", "--tenant", "acme"]);
    assert!(
        ingest_out.contains("3 echelons"),
        "ingest must report what it accepted: {ingest_out}"
    );

    // the recommendation: safety stock under staged service levels
    let rec = tessera(&["inv", "recommend", "--tenant", "acme"]);
    for echelon in ["store-1", "store-2", "dc"] {
        assert!(
            rec.contains(echelon),
            "recommendation must cover {echelon}: {rec}"
        );
    }
    // every recommendation carries its method and assumptions (E2.3)
    assert!(
        rec.contains("staged service-level MEIO"),
        "a bare number is not a recommendation: {rec}"
    );

    // the override loop: an override is free but never silent (E2.4)
    tessera(&[
        "inv",
        "override",
        "--tenant",
        "acme",
        "--echelon",
        "dc",
        "--safety-stock",
        "200",
        "--reason",
        "supplier moq",
    ]);

    // the ledger entry recording the recommendation exists and verifies
    let ledger = tessera(&["ledger", "read", "--tenant", "acme"]);
    assert!(
        ledger.contains("recommend"),
        "the recommendation must be ledger-stamped: {ledger}"
    );
    assert!(
        ledger.contains("override"),
        "the override must carry its receipt: {ledger}"
    );

    std::fs::remove_file(&csv).ok();
}
