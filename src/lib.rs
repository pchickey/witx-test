use tempfile;
#[cfg(test)]
use wasi_headers;

mod generate_doc;
mod render;

pub use generate_doc::{
    GenDoc, GenEnum, GenFlags, GenHandle, GenStruct, GenType, GenTypeRef, GenUnion, Limits,
};

pub fn check_header(header: &str) {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    let tempdir = TempDir::new().expect("create tempdir");
    let mut f = File::create(tempdir.path().join("test.h")).expect("create test.h");
    f.write_all(header.as_bytes()).expect("write test.h");

    let wasi_sdk = PathBuf::from("/opt/wasi-sdk/");
    //PathBuf::from(std::env::var("WASI_SDK").unwrap_or_else(|_| "/opt/wasi-sdk".to_string()));

    let run_clang = Command::new(wasi_sdk.join("bin").join("clang"))
        .arg("-c")
        .arg(tempdir.path().join("test.h"))
        .arg("-o")
        .arg(tempdir.path().join("test.o"))
        .output()
        .expect("run wasi-sdk clang");
    if !run_clang.status.success() {
        panic!(
            "clang rejected header: {}\n{}",
            header,
            String::from_utf8_lossy(&run_clang.stderr)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{check_header, GenDoc, Limits};
    use proptest::proptest;
    proptest! {
        #[test]
        fn generate_witx_spec(gen_doc in GenDoc::strat(&Limits::default())) {
            let gen_syntax = format!("{}", gen_doc);
            println!("{}", gen_syntax);
            witx::parse(&gen_syntax).unwrap();
        }


        #[test]
        fn generate_witx_header(gen_doc in GenDoc::strat(&Limits::default())) {
            let gen_syntax = format!("{}", gen_doc);
            println!("{}", gen_syntax);
            let doc = witx::parse(&gen_syntax).unwrap();
            let header = wasi_headers::to_c_header(&doc, "wasi-test generated");
            check_header(&header);

        }
    }
}
