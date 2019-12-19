use crate::generate_doc::*;
use std::fmt;
use witx::SExpr;

impl fmt::Display for GenDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (ix, gentype) in self.typenames.iter().enumerate() {
            write!(f, "{}\n", gentype.to_sexpr(ix))?;
        }

        write!(f, "\n")?;

        for (ix, module) in self.modules.iter().enumerate() {
            write!(f, "{}\n", module.to_sexpr(ix))?;
        }
        Ok(())
    }
}

impl GenType {
    pub fn to_sexpr(&self, ix: usize) -> SExpr {
        let body = match self {
            GenType::Name(tref) => tref.to_sexpr(),
            GenType::Enum(e) => e.to_sexpr(),
            GenType::Flags(f) => f.to_sexpr(),
            GenType::Struct(s) => s.to_sexpr(),
            GenType::Union(u) => u.to_sexpr(),
            GenType::Handle(h) => h.to_sexpr(),
            GenType::Array(elem) => SExpr::Vec(vec![SExpr::word("array"), elem.to_sexpr()]),
            GenType::Pointer(elem) => SExpr::Vec(vec![
                SExpr::annot("witx"),
                SExpr::word("pointer"),
                elem.to_sexpr(),
            ]),
            GenType::ConstPointer(elem) => SExpr::Vec(vec![
                SExpr::annot("witx"),
                SExpr::word("const_pointer"),
                elem.to_sexpr(),
            ]),
            GenType::Builtin(b) => b.to_sexpr(),
        };
        SExpr::Vec(vec![
            SExpr::word("typename"),
            SExpr::Ident(format!("t_{}", ix)),
            body,
        ])
    }
}

impl GenTypeRef {
    pub fn to_sexpr(&self) -> SExpr {
        SExpr::Ident(format!("t_{}", self.idx))
    }
}

impl GenEnum {
    pub fn to_sexpr(&self) -> SExpr {
        let mut v = vec![SExpr::word("enum"), self.repr.to_sexpr()];
        v.extend(
            (0..self.members)
                .into_iter()
                .map(|ix| SExpr::Ident(format!("e_{}", ix)))
                .collect::<Vec<SExpr>>(),
        );
        SExpr::Vec(v)
    }
}

impl GenFlags {
    pub fn to_sexpr(&self) -> SExpr {
        let mut v = vec![SExpr::word("flags"), self.repr.to_sexpr()];
        v.extend(
            (0..self.members)
                .into_iter()
                .map(|ix| SExpr::Ident(format!("f_{}", ix)))
                .collect::<Vec<SExpr>>(),
        );
        SExpr::Vec(v)
    }
}

impl GenStruct {
    pub fn to_sexpr(&self) -> SExpr {
        let mut v = vec![SExpr::word("struct")];
        v.extend(
            self.members
                .iter()
                .enumerate()
                .map(|(ix, ty)| {
                    SExpr::Vec(vec![
                        SExpr::word("field"),
                        SExpr::Ident(format!("s_{}", ix)),
                        ty.to_sexpr(),
                    ])
                })
                .collect::<Vec<SExpr>>(),
        );
        SExpr::Vec(v)
    }
}

impl GenUnion {
    pub fn to_sexpr(&self) -> SExpr {
        let mut v = vec![SExpr::word("union")];
        v.extend(
            self.variants
                .iter()
                .enumerate()
                .map(|(ix, ty)| {
                    SExpr::Vec(vec![
                        SExpr::word("field"),
                        SExpr::Ident(format!("u_{}", ix)),
                        ty.to_sexpr(),
                    ])
                })
                .collect::<Vec<SExpr>>(),
        );
        SExpr::Vec(v)
    }
}

impl GenHandle {
    pub fn to_sexpr(&self) -> SExpr {
        let mut v = vec![SExpr::word("handle")];
        v.extend(
            self.supertypes
                .iter()
                .map(|ty| ty.to_sexpr())
                .collect::<Vec<SExpr>>(),
        );
        SExpr::Vec(v)
    }
}

impl GenModule {
    pub fn to_sexpr(&self, ix: usize) -> SExpr {
        let mut m = vec![SExpr::word("module"), SExpr::Ident(format!("m_{}", ix))];
        m.extend(self.funcs.iter().enumerate().map(|(ix, f)| f.to_sexpr(ix)));
        SExpr::Vec(m)
    }
}

impl GenFunc {
    pub fn to_sexpr(&self, ix: usize) -> SExpr {
        let mut f = vec![
            SExpr::annot("interface"),
            SExpr::word("func"),
            SExpr::Vec(vec![
                SExpr::word("export"),
                SExpr::Quote(format!("f_{}", ix)),
            ]),
        ];
        f.extend(
            self.params
                .iter()
                .enumerate()
                .map(|(ix, ty)| {
                    SExpr::Vec(vec![
                        SExpr::word("param"),
                        SExpr::Ident(format!("p_{}", ix)),
                        ty.to_sexpr(),
                    ])
                })
                .collect::<Vec<SExpr>>(),
        );
        f.extend(
            self.results
                .iter()
                .enumerate()
                .map(|(ix, ty)| {
                    SExpr::Vec(vec![
                        SExpr::word("result"),
                        SExpr::Ident(format!("r_{}", ix)),
                        ty.to_sexpr(),
                    ])
                })
                .collect::<Vec<SExpr>>(),
        );
        SExpr::Vec(f)
    }
}
