mod generate_doc;
mod render;

pub use generate_doc::{
    GenDoc, GenEnum, GenFlags, GenHandle, GenStruct, GenType, GenTypeRef, GenUnion, Limits,
};

#[cfg(test)]
mod tests {
    use super::{GenDoc, Limits};
    use proptest::proptest;
    proptest! {
        #[test]
        fn generate_witx(gen_doc in GenDoc::strat(&Limits::default())) {
            let gen_syntax = format!("{}", gen_doc);
            println!("{}", gen_syntax);
            witx::parse(&gen_syntax).unwrap();
        }
    }
}
