use proptest::prelude::*;
use witx_test::generate_doc::*;
use witx;

fn main() {
    let mut runner = prop::test_runner::TestRunner::default();
    let limits = Limits::default();
    let gen_doc = GenDoc::strat(&limits)
        .new_tree(&mut runner)
        .unwrap()
        .current();

    let gen_syntax = format!("{}", gen_doc);
    println!("{}", gen_syntax);

    witx::parse(&gen_syntax).unwrap();
    println!("valid!");
}
