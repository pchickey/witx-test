use proptest::prelude::*;
use witx;
use witx_test::*;
use wasi_headers;

fn main() {
    let mut runner = prop::test_runner::TestRunner::default();
    let limits = Limits::default();
    let gen_doc = GenDoc::strat(&limits)
        .new_tree(&mut runner)
        .expect("generate doc tree")
        .current();

    let gen_syntax = format!("{}", gen_doc);
    println!("{}", gen_syntax);

    let doc = witx::parse(&gen_syntax).expect("parse doc");

    let header = wasi_headers::to_c_header(&doc, "wasi-test generated");

    check_header(&header);
}
