mod utils;

use utils::run_suite;

#[test]
fn run_8080pre() {
    run_suite("./resources/cpu_tests/8080PRE.COM");
}

#[test]
fn run_tst8080() {
    run_suite("./resources/cpu_tests/TST8080.COM");
}

#[test]
fn run_cpudiag() {
    run_suite("./resources/cpu_tests/cpudiag.bin");
}

#[test]
fn run_cputest() {
    run_suite("./resources/cpu_tests/CPUTEST.COM");
}

#[test]
fn run_8080exm() {
    run_suite("./resources/cpu_tests/8080EXM.COM");
}
