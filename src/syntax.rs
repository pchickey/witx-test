use proptest::prelude::*;
pub use witx::{BuiltinType, IntRepr};

/// Limit the size of the generated specification
#[derive(Debug, Clone)]
pub struct Limits {
    pub types: usize,
    pub struct_members: usize,
    pub union_variants: usize,
    pub enum_variants: usize,
    pub flag_members: usize,
    pub handle_supertypes: usize,
    pub modules: usize,
    pub funcs: usize,
    pub func_params: usize,
    pub func_results: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            types: 20,
            struct_members: 10,
            union_variants: 5,
            enum_variants: 32,
            flag_members: 64,
            handle_supertypes: 2,
            modules: 3,
            funcs: 10,
            func_params: 5,
            func_results: 3,
        }
    }
}

// Normalize arbitrary specification into a valid one.
struct Norm {
    // Typeref can only refer to types less than this:
    max_typeref: usize,
    // Handles can only have supertypes from this set:
    valid_handles: Vec<usize>,
}

impl Norm {
    pub fn new(types: &[GenType]) -> Self {
        let max_typeref = types.len();
        let valid_handles = types
            .iter()
            .enumerate()
            .filter_map(|(ix, t)| match t {
                GenType::Handle { .. } => Some(ix),
                _ => None,
            })
            .collect();
        Norm {
            max_typeref,
            valid_handles,
        }
    }

    pub fn typeref(&self, t: &GenTypeRef) -> GenTypeRef {
        GenTypeRef {
            idx: t.idx % self.max_typeref,
        }
    }

    pub fn handle(&self, h: &GenHandle) -> GenHandle {
        if self.valid_handles.is_empty() {
            GenHandle {
                supertypes: Vec::new(),
            }
        } else {
            let num_valid_handles = self.valid_handles.len();
            let supertypes = h
                .supertypes
                .iter()
                .map(|tref| GenTypeRef {
                    idx: self.valid_handles[tref.idx % num_valid_handles],
                })
                .collect();
            GenHandle { supertypes }
        }
    }
}

/// Generated specification document
#[derive(Debug, Clone)]
pub struct GenDoc {
    typenames: Vec<GenType>,
    modules: Vec<GenModule>,
}

impl GenDoc {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        (
            GenEnum::strat(limits),
            prop::collection::vec(GenType::strat(limits), 0..limits.types),
            prop::collection::vec(GenModule::strat(limits), 1..limits.modules),
        )
            .prop_map(|(tzero, tnames, mods)| {
                // When normalizing, need a type at index 0 that does not refer to other types, so
                // generate an enum for that. This also ensures we have a valid non-pointer type to
                // use as the first return value.
                let tnames = {
                    let mut ext = vec![GenType::Enum(tzero)];
                    ext.extend(tnames);
                    ext
                };
                let typenames = tnames
                    .iter()
                    .enumerate()
                    .map(|(ix, t)| {
                        let norm = Norm::new(&tnames[0..ix]);
                        t.normalize(&norm)
                    })
                    .collect();
                let norm = Norm::new(&tnames);
                let modules = mods.iter().map(|m| m.normalize(&norm)).collect();
                GenDoc { typenames, modules }
            })
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub enum GenType {
    Name(GenTypeRef),
    Enum(GenEnum),
    Flags(GenFlags),
    Struct(GenStruct),
    Union(GenUnion),
    Handle(GenHandle),
    Array(GenTypeRef),
    Pointer(GenTypeRef),
    ConstPointer(GenTypeRef),
    Builtin(BuiltinType),
}

impl GenType {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        prop_oneof![
            GenTypeRef::strat(limits).prop_map(GenType::Name),
            GenEnum::strat(limits).prop_map(GenType::Enum),
            GenFlags::strat(limits).prop_map(GenType::Flags),
            GenStruct::strat(limits).prop_map(GenType::Struct),
            GenUnion::strat(limits).prop_map(GenType::Union),
            GenHandle::strat(limits).prop_map(GenType::Handle),
            GenTypeRef::strat(limits).prop_map(GenType::Array),
            GenTypeRef::strat(limits).prop_map(GenType::Pointer),
            GenTypeRef::strat(limits).prop_map(GenType::ConstPointer),
        ]
        .boxed()
    }
    fn normalize(&self, norm: &Norm) -> Self {
        use GenType::*;
        match self {
            Name(tref) => Name(norm.typeref(tref)),
            Enum(e) => Enum(e.clone()),
            Flags(f) => Flags(f.clone()),
            Struct(s) => Struct(s.normalize(norm)),
            Union(u) => Union(u.normalize(norm)),
            Handle(h) => Handle(norm.handle(h)),
            Array(a) => Array(a.normalize(norm)),
            Pointer(p) => Pointer(p.normalize(norm)),
            ConstPointer(p) => ConstPointer(p.normalize(norm)),
            Builtin(b) => Builtin(*b),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenTypeRef {
    idx: usize,
}

impl GenTypeRef {
    pub fn strat(_limits: &Limits) -> BoxedStrategy<Self> {
        prop::num::usize::ANY
            .prop_map(|idx| GenTypeRef { idx })
            .boxed()
    }
    fn normalize(&self, norm: &Norm) -> Self {
        GenTypeRef {
            idx: self.idx % norm.max_typeref,
        }
    }
}

trait NoParamGen
where
    Self: Sized,
{
    fn strat() -> BoxedStrategy<Self>;
}

impl NoParamGen for BuiltinType {
    fn strat() -> BoxedStrategy<Self> {
        prop_oneof![
            Just(BuiltinType::String),
            Just(BuiltinType::U8),
            Just(BuiltinType::U16),
            Just(BuiltinType::U32),
            Just(BuiltinType::U64),
            Just(BuiltinType::S8),
            Just(BuiltinType::S16),
            Just(BuiltinType::S32),
            Just(BuiltinType::S64),
            Just(BuiltinType::F32),
            Just(BuiltinType::F64),
        ]
        .boxed()
    }
}

impl NoParamGen for IntRepr {
    fn strat() -> BoxedStrategy<Self> {
        prop_oneof![
            Just(IntRepr::U8),
            Just(IntRepr::U16),
            Just(IntRepr::U32),
            Just(IntRepr::U64),
        ]
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct GenEnum {
    repr: IntRepr,
    members: usize,
}

impl GenEnum {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        let limits = limits.clone();
        (IntRepr::strat(), prop::num::usize::ANY)
            .prop_map(move |(repr, members)| {
                let max_members = std::cmp::min(
                    limits.enum_variants,
                    match repr {
                        IntRepr::U8 => std::u8::MAX as usize,
                        IntRepr::U16 => std::u16::MAX as usize,
                        IntRepr::U32 => std::u32::MAX as usize,
                        IntRepr::U64 => std::u64::MAX as usize,
                    },
                );
                GenEnum {
                    repr,
                    members: members % max_members,
                }
            })
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct GenFlags {
    repr: IntRepr,
    members: usize,
}

impl GenFlags {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        let limits = limits.clone();
        (IntRepr::strat(), prop::num::usize::ANY)
            .prop_map(move |(repr, members)| {
                let max_flags = std::cmp::min(
                    limits.flag_members,
                    match repr {
                        IntRepr::U8 => 8,
                        IntRepr::U16 => 16,
                        IntRepr::U32 => 32,
                        IntRepr::U64 => 64,
                    },
                );
                GenFlags {
                    repr,
                    members: members % max_flags,
                }
            })
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct GenStruct {
    members: Vec<GenTypeRef>,
}

impl GenStruct {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        prop::collection::vec(GenTypeRef::strat(limits), 1..limits.struct_members)
            .prop_map(|members| GenStruct { members })
            .boxed()
    }
    fn normalize(&self, norm: &Norm) -> Self {
        GenStruct {
            members: self.members.iter().map(|t| t.normalize(norm)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenUnion {
    variants: Vec<GenTypeRef>,
}

impl GenUnion {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        prop::collection::vec(GenTypeRef::strat(limits), 1..limits.union_variants)
            .prop_map(|variants| GenUnion { variants })
            .boxed()
    }
    fn normalize(&self, norm: &Norm) -> Self {
        GenUnion {
            variants: self.variants.iter().map(|t| t.normalize(norm)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenHandle {
    supertypes: Vec<GenTypeRef>,
}

impl GenHandle {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        prop::collection::vec(GenTypeRef::strat(limits), 1..limits.handle_supertypes)
            .prop_map(|supertypes| GenHandle { supertypes })
            .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct GenModule {
    funcs: Vec<GenFunc>,
}

impl GenModule {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        prop::collection::vec(GenFunc::strat(limits), 1..limits.funcs)
            .prop_map(|funcs| GenModule { funcs })
            .boxed()
    }
    fn normalize(&self, norm: &Norm) -> Self {
        GenModule {
            funcs: self.funcs.iter().map(|f| f.normalize(norm)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenFunc {
    params: Vec<GenTypeRef>,
    results: Vec<GenTypeRef>,
}

impl GenFunc {
    pub fn strat(limits: &Limits) -> BoxedStrategy<Self> {
        (
            prop::collection::vec(GenTypeRef::strat(limits), 0..limits.func_params),
            prop::collection::vec(GenTypeRef::strat(limits), 0..(limits.func_results - 1)),
        )
            .prop_map(|(params, results)| {
                // First result has to be passable by value - type 0 always is.
                let results = {
                    let mut r = vec![GenTypeRef { idx: 0 }];
                    r.extend(results);
                    r
                };
                GenFunc { params, results }
            })
            .boxed()
    }
    fn normalize(&self, norm: &Norm) -> Self {
        GenFunc {
            params: self.params.iter().map(|t| norm.typeref(t)).collect(),
            results: self.results.iter().map(|t| norm.typeref(t)).collect(),
        }
    }
}
