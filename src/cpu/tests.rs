
mod control_tests;
mod alu_tests;
mod bitwise_tests;
mod ld_tests;
mod jump_branch_tests;

// #[test]
// fn benchmark_test()
// {
//     use crate::cpu::*;
//     use std::time::Instant;

//     let lstart = Instant::now();

//     let mut proc = Cpu::new();
//     let count = 1_000_000_000;
//     //Test add
//     proc.reg_a = 0b00000111;
//     proc.reg_b = 1;
//     for _ in 0..count
//     {
//         Cpu::add_r8_r8(&mut proc.reg_a, &mut proc.reg_b, &mut proc.reg_f);
//     }

//     let lend = Instant::now();
//     println!("TIME RESULT: {:?}", lend - lstart);
//     println!("TIME RESULT PER ADD: {:?}", (lend - lstart) / count);
//     println!("RESULTS: {} {} {:?}", proc.reg_a, proc.reg_b, proc.reg_f);
//     assert!(true);
// }