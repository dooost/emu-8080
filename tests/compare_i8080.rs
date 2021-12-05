mod utils;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use emu_8080::emulator::State8080;
use i8080::{Cpu, Linear, Memory};
use utils::{create_state_with_rom, print_output};

#[test]
fn compare_on_8080pre() {
    run_comparison("./resources/cpu_tests/8080PRE.COM", false);
}

#[test]
fn compare_on_tst8080() {
    run_comparison("./resources/cpu_tests/TST8080.COM", false);
}

#[test]
fn compare_on_cpudiag() {
    run_comparison("./resources/cpu_tests/cpudiag.bin", false);
}

#[test]
fn compare_on_cputest() {
    run_comparison("./resources/cpu_tests/CPUTEST.COM", false);
}

// This takes a few minutes to finish (3-4 mins on a high-end Intel MBP)
#[test]
fn compare_on_8080exm() {
    run_comparison("./resources/cpu_tests/8080EXM.COM", false);
}

// This is super valuable for debugging where things are going south, but the current implementation
// clones the old state to know what failed, which makes it very slow. Takes an hour for me to finish.
#[test]
#[ignore]
fn compare_on_8080exm_print_failed_instruction() {
    run_comparison("./resources/cpu_tests/8080EXM.COM", true);
}

fn run_comparison(path: impl AsRef<Path>, keep_old_state: bool) {
    let mut state = create_state_with_rom(path.as_ref());

    let mem = Rc::new(RefCell::new(Linear::new()));
    load_test(mem.clone(), path.as_ref());
    let mut cpu = Cpu::power_up(mem.clone());
    mem.borrow_mut().set(0x0005, 0xc9);
    // Because tests used the pseudo instruction ORG 0x0100
    cpu.reg.pc = 0x0100;

    let mut old_state = if keep_old_state {
        Some(state.clone())
    } else {
        None
    };

    let filename = path
        .as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    println!("Starting running suite {}...", filename);

    loop {
        if !compare_states(&state, &cpu) {
            if keep_old_state {
                old_state.unwrap().log_current_instruction();
            }
            panic!("States do not match");
        }

        if keep_old_state {
            old_state = Some(state.clone());
        }

        state = state.evaluating_next();
        cpu.next();

        print_output(&state);
        print_reference_output(&cpu);

        if state.pc == 0 {
            println!();
            println!("Jumped to 0x0000, halting");
            break;
        }

        if cpu.reg.pc == 0x00 {
            println!();
            println!("Reference Jumped to 0x0000, halting");
            break;
        }
    }
}

fn compare_states(state: &State8080, cpu: &Cpu) -> bool {
    if cpu.reg.a != state.a {
        println!(
            "Reg A mismatch: Should be {}, but is {}",
            cpu.reg.a, state.a
        );
        return false;
    } else if cpu.reg.b != state.b {
        println!(
            "Reg B mismatch: Should be {}, but is {}",
            cpu.reg.b, state.b
        );
        return false;
    } else if cpu.reg.c != state.c {
        println!(
            "Reg C mismatch: Should be {}, but is {}",
            cpu.reg.c, state.c
        );
        return false;
    } else if cpu.reg.d != state.d {
        println!(
            "Reg D mismatch: Should be {}, but is {}",
            cpu.reg.d, state.d
        );
        return false;
    } else if cpu.reg.e != state.e {
        println!(
            "Reg E mismatch: Should be {}, but is {}",
            cpu.reg.e, state.e
        );
        return false;
    } else if cpu.reg.h != state.h {
        println!(
            "Reg H mismatch: Should be {}, but is {}",
            cpu.reg.h, state.h
        );
        return false;
    } else if cpu.reg.l != state.l {
        println!(
            "Reg L mismatch: Should be {}, but is {}",
            cpu.reg.l, state.l
        );
        return false;
    } else if cpu.reg.sp != state.sp {
        println!(
            "Reg SP mismatch: Should be {}, but is {}",
            cpu.reg.sp, state.sp
        );
        return false;
    } else if cpu.reg.pc != state.pc {
        println!(
            "Reg PC mismatch: Should be {}, but is {}",
            cpu.reg.pc, state.pc
        );
        return false;
    } else if cpu.mem.borrow().get(cpu.reg.pc) != state.memory[state.pc as usize] {
        println!(
            "Memory data at PC mismatch: Should be {}, but is {}",
            cpu.mem.borrow().get(cpu.reg.pc),
            state.memory[state.pc as usize]
        );
        return false;
    } else if cpu.mem.borrow().get(cpu.reg.sp) != state.memory[state.sp as usize] {
        println!(
            "Memory data at PC mismatch: Should be {}, but is {}",
            cpu.mem.borrow().get(cpu.reg.sp),
            state.memory[state.sp as usize]
        );
        return false;
    } else if cpu.reg.f != state.cc.bits() {
        println!(
            "Reg F mismatch: Should be {:b}, but is {:b}",
            cpu.reg.f,
            state.cc.bits()
        );

        return false;
    }

    true
}

fn print_reference_output(cpu: &Cpu) {
    if cpu.reg.pc == 0x05 {
        if cpu.reg.c == 0x09 {
            let mut a = cpu.reg.get_de();
            loop {
                let c = cpu.mem.borrow().get(a);
                if c as char == '$' {
                    break;
                } else {
                    a += 1;
                }
                print!("{}", c as char);
            }
        }
        if cpu.reg.c == 0x02 {
            print!("{}", cpu.reg.e as char);
        }
    }
}

fn load_test(mem: Rc<RefCell<Linear>>, path: impl AsRef<Path>) {
    let mut file = File::open(path.as_ref()).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    mem.borrow_mut().data[0x0100..(buf.len() + 0x0100)].clone_from_slice(&buf[..]);
    println!("Test loaded: {:?}", path.as_ref());
}
