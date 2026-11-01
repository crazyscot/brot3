#![allow(missing_docs)]

use naga::{AddressSpace, Handle, Module, ScalarKind, TypeInner, VectorSize};

fn parse_module() -> Module {
    let mut src = String::from(include_str!("../src/wgsl/common.wgsl"));
    src.push('\n');
    src.push_str(&brot3_idl::ALL_WGSL);
    naga::front::wgsl::parse_str(&src).expect("WGSL parse failed for common.wgsl")
}

fn find_named_type(module: &Module, name: &str) -> Handle<naga::Type> {
    module
        .types
        .iter()
        .find_map(|(h, t)| (t.name.as_deref() == Some(name)).then_some(h))
        .unwrap_or_else(|| panic!("Type named {name} was not found in WGSL module"))
}

fn assert_struct_members(
    module: &Module,
    struct_name: &str,
    expected_members: &[(&str, ExpectedType)],
) {
    let handle = find_named_type(module, struct_name);
    let ty = &module.types[handle];
    let members = match &ty.inner {
        TypeInner::Struct { members, .. } => members,
        other => panic!("Type {struct_name} is not a struct, got {other:?}"),
    };

    assert_eq!(
        members.len(),
        expected_members.len(),
        "Struct {struct_name} member count mismatch"
    );

    for (idx, (member, (expected_name, expected_type))) in
        members.iter().zip(expected_members.iter()).enumerate()
    {
        assert_eq!(
            member.name.as_deref(),
            Some(*expected_name),
            "Struct {struct_name} member {idx} name mismatch"
        );
        assert_expected_type(module, member.ty, expected_type, struct_name, expected_name);
    }
}

#[derive(Clone, Copy, Debug)]
enum ExpectedType {
    NamedStruct(&'static str),
    U32,
    I32,
    F32,
    F32x2,
}

fn assert_expected_type(
    module: &Module,
    handle: Handle<naga::Type>,
    expected: &ExpectedType,
    struct_name: &str,
    member_name: &str,
) {
    let actual = &module.types[handle];
    match expected {
        ExpectedType::NamedStruct(expected_name) => {
            assert_eq!(
                actual.name.as_deref(),
                Some(*expected_name),
                "Struct {struct_name}.{member_name} expected named type {expected_name}"
            );
            assert!(
                matches!(actual.inner, TypeInner::Struct { .. }),
                "Struct {struct_name}.{member_name} expected struct type"
            );
        }
        ExpectedType::U32 => match actual.inner {
            TypeInner::Scalar(s) => {
                assert_eq!(s.kind, ScalarKind::Uint);
                assert_eq!(s.width, 4);
            }
            ref other => {
                panic!("Struct {struct_name}.{member_name} expected u32 scalar, got {other:?}")
            }
        },
        ExpectedType::I32 => match actual.inner {
            TypeInner::Scalar(s) => {
                assert_eq!(s.kind, ScalarKind::Sint);
                assert_eq!(s.width, 4);
            }
            ref other => {
                panic!("Struct {struct_name}.{member_name} expected i32 scalar, got {other:?}")
            }
        },
        ExpectedType::F32 => match actual.inner {
            TypeInner::Scalar(s) => {
                assert_eq!(s.kind, ScalarKind::Float);
                assert_eq!(s.width, 4);
            }
            ref other => {
                panic!("Struct {struct_name}.{member_name} expected f32 scalar, got {other:?}")
            }
        },
        ExpectedType::F32x2 => match actual.inner {
            TypeInner::Vector { size, scalar } => {
                assert_eq!(size, VectorSize::Bi);
                assert_eq!(scalar.kind, ScalarKind::Float);
                assert_eq!(scalar.width, 4);
            }
            ref other => {
                panic!("Struct {struct_name}.{member_name} expected vec2<f32>, got {other:?}")
            }
        },
    }
}

#[test]
fn fragment_constants_wgsl_definition_matches_contract() {
    let module = parse_module();

    let globals: Vec<_> = module.global_variables.iter().collect();
    let has_immediate_fragment_constants = globals.iter().any(|(_, gv)| {
        gv.space == AddressSpace::Immediate
            && module.types[gv.ty].name.as_deref() == Some("FragmentConstants")
    });
    for (name, gv) in &globals {
        println!(
            "Global variable: {:?}, space {:?}, type {:?}",
            name, gv.space, module.types[gv.ty].name
        );
    }
    assert!(
        has_immediate_fragment_constants,
        "Expected var<immediate> constants: FragmentConstants in WGSL"
    );

    assert_struct_members(
        &module,
        "Size",
        &[("width", ExpectedType::U32), ("height", ExpectedType::U32)],
    );
    assert_struct_members(
        &module,
        "PushExponent",
        &[
            ("typ", ExpectedType::U32),
            ("int", ExpectedType::I32),
            ("real", ExpectedType::F32),
        ],
    );
    assert_struct_members(
        &module,
        "Palette",
        &[
            ("colourer", ExpectedType::U32),
            ("colour_style", ExpectedType::U32),
            ("brightness_style", ExpectedType::U32),
            ("gradient", ExpectedType::F32),
            ("offset", ExpectedType::F32),
        ],
    );
    assert_struct_members(
        &module,
        "FragmentConstants",
        &[
            ("exponent", ExpectedType::NamedStruct("PushExponent")),
            ("flags", ExpectedType::NamedStruct("WgslFlags")),
            ("viewport_size", ExpectedType::NamedStruct("Size")),
            ("buffer_size", ExpectedType::NamedStruct("Size")),
            ("inspector_point_pixel_address", ExpectedType::F32x2),
            ("viewport_translate", ExpectedType::F32x2),
            ("viewport_zoom", ExpectedType::F32),
            ("algorithm", ExpectedType::U32),
            ("max_iter", ExpectedType::U32),
            ("palette", ExpectedType::NamedStruct("Palette")),
        ],
    );
}
