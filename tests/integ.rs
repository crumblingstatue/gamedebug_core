use gamedebug_core::{imm, imm_dbg, per, per_dbg};

#[test]
fn test() {
    gamedebug_core::toggle();
    dbg!(gamedebug_core::enabled());
    per!("Hi!");
    imm!("Hi!");
    per_dbg!(42);
    imm_dbg!(42);
    imm_dbg!(2, 4, 6);
}
