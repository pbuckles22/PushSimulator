//! Times validation, play, and hit on the 14-card fixture.
//!
//! Prints median, p95, and p99 in nanoseconds, plus the host. Release is the
//! reference record. Debug prints the same shape and does not apply ceilings.

use std::process::Command;

use push_core::latency::{
    profile_fourteen_card, HIT_GATE, PLAY_GATE, VALIDATE_GATE, LatencyGate, TimedComponent,
};

fn main() {
    let profile = profile_fourteen_card(100);
    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default();
    let cpu = std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_default();
    println!(
        "reference arch={} os={} family={} cpu={} rustc={}",
        profile.machine.arch,
        profile.machine.os,
        profile.machine.family,
        cpu.trim(),
        rustc.trim()
    );
    print_component("validate", &profile.validate);
    print_component("play", &profile.play);
    print_component("hit", &profile.hit);
    if !cfg!(debug_assertions) {
        enforce("validate", &profile.validate, VALIDATE_GATE);
        enforce("play", &profile.play, PLAY_GATE);
        enforce("hit", &profile.hit, HIT_GATE);
        println!("reference gates=pass");
    }
}

fn enforce(name: &str, component: &TimedComponent, gate: LatencyGate) {
    if component.latency.within(gate) {
        return;
    }
    eprintln!(
        "{name} exceeded the reference ceiling median {} p95 {} p99 {} gate median {} p95 {} p99 {}",
        component.latency.median_ns,
        component.latency.p95_ns,
        component.latency.p99_ns,
        gate.median_ns,
        gate.p95_ns,
        gate.p99_ns
    );
    std::process::exit(1);
}

fn print_component(name: &str, component: &TimedComponent) {
    println!(
        "{name} actions={} samples={} median_ns={} p95_ns={} p99_ns={}",
        component.actions,
        component.latency.samples,
        component.latency.median_ns,
        component.latency.p95_ns,
        component.latency.p99_ns
    );
}
