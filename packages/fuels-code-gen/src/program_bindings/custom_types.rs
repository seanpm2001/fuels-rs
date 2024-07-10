use std::collections::HashSet;

use fuel_abi_types::abi::full_program::FullTypeDeclaration;
use itertools::Itertools;
use quote::quote;

use crate::{
    error::Result,
    program_bindings::{
        custom_types::{enums::expand_custom_enum, structs::expand_custom_struct},
        generated_code::GeneratedCode,
        utils::sdk_provided_custom_types_lookup,
    },
    utils::TypePath,
};

mod enums;
mod structs;
mod utils;

/// Generates Rust code for each type inside `types` if:
/// * the type is not present inside `shared_types`, and
/// * if it should be generated (see: [`should_skip_codegen`], and
/// * if it is a struct or an enum.
///
///
/// # Arguments
///
/// * `types`: Types you wish to generate Rust code for.
/// * `shared_types`: Types that are shared between multiple
///                   contracts/scripts/predicates and thus generated elsewhere.
pub(crate) fn generate_types<'a, T: IntoIterator<Item = &'a FullTypeDeclaration>>(
    types: T,
    shared_types: &HashSet<FullTypeDeclaration>,
    no_std: bool,
) -> Result<GeneratedCode> {
    types
        .into_iter()
        .filter(|ttype| !should_skip_codegen(ttype))
        .map(|ttype: &FullTypeDeclaration| {
            if shared_types.contains(ttype) {
                reexport_the_shared_type(ttype, no_std)
            } else if ttype.is_struct_type() {
                expand_custom_struct(ttype, no_std)
            } else {
                expand_custom_enum(ttype, no_std)
            }
        })
        .fold_ok(GeneratedCode::default(), |acc, generated_code| {
            acc.merge(generated_code)
        })
}

/// Instead of generating bindings for `ttype` this fn will just generate a `pub use` pointing to
/// the already generated equivalent shared type.
fn reexport_the_shared_type(ttype: &FullTypeDeclaration, no_std: bool) -> Result<GeneratedCode> {
    // e.g. some_library::another_mod::SomeStruct
    let type_path = ttype
        .custom_type_path()
        .expect("This must be a custom type due to the previous filter step");

    let type_mod = type_path.parent();

    let from_top_lvl_to_shared_types =
        TypePath::new("super::shared_types").expect("This is known to be a valid TypePath");

    let top_lvl_mod = TypePath::default();
    let from_current_mod_to_top_level = top_lvl_mod.relative_path_from(&type_mod);

    let path = from_current_mod_to_top_level
        .append(from_top_lvl_to_shared_types)
        .append(type_path);

    // e.g. pub use super::super::super::shared_types::some_library::another_mod::SomeStruct;
    let the_reexport = quote! {pub use #path;};

    Ok(GeneratedCode::new(the_reexport, Default::default(), no_std).wrap_in_mod(type_mod))
}

// Checks whether the given type should not have code generated for it. This
// is mainly because the corresponding type in Rust already exists --
// e.g. the contract's Vec type is mapped to std::vec::Vec from the Rust
// stdlib, ContractId is a custom type implemented by fuels-rs, etc.
// Others like 'std::vec::RawVec' are skipped because they are
// implementation details of the contract's Vec type and are not directly
// used in the SDK.
fn should_skip_codegen(type_decl: &FullTypeDeclaration) -> bool {
    if !type_decl.is_custom_type() {
        return true;
    }

    let type_path = type_decl.custom_type_path().unwrap();

    is_type_sdk_provided(&type_path) || is_type_unused(&type_path)
}

fn is_type_sdk_provided(type_path: &TypePath) -> bool {
    sdk_provided_custom_types_lookup().contains_key(type_path)
}

fn is_type_unused(type_path: &TypePath) -> bool {
    let msg = "Known to be correct";
    [
        // TODO: To be removed once https://github.com/FuelLabs/fuels-rs/issues/881 is unblocked.
        TypePath::new("RawBytes").expect(msg),
        TypePath::new("std::vec::RawVec").expect(msg),
        TypePath::new("std::bytes::RawBytes").expect(msg),
        // TODO: To be removed once https://github.com/FuelLabs/fuels-rs/issues/881 is unblocked.
        TypePath::new("RawVec").expect(msg),
    ]
    .contains(type_path)
}

// Doing string -> TokenStream -> string isn't pretty but gives us the opportunity to
// have a better understanding of the generated code so we consider it ok.
// To generate the expected examples, output of the functions were taken
// with code @9ca376, and formatted in-IDE using rustfmt. It should be noted that
// rustfmt added an extra `,` after the last struct/enum field, which is not added
// by the `expand_custom_*` functions, and so was removed from the expected string.
// TODO(iqdecay): append extra `,` to last enum/struct field so it is aligned with rustfmt
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use fuel_abi_types::abi::program::{ProgramABI, TypeApplication, TypeDeclaration};
    use pretty_assertions::assert_eq;
    use quote::quote;

    use super::*;

    #[test]
    fn test_expand_custom_enum() -> Result<()> {
        let p = TypeDeclaration {
            type_id: "254e1430fc69530308933b4ebd8c79569530b1835705d0bd3f0e03c155dcd09a".to_string(),
            type_field: String::from("enum MatchaTea"),
            components: Some(vec![
                TypeApplication {
                    name: String::from("LongIsland"),
                    type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                        .to_owned(),
                    ..Default::default()
                },
                TypeApplication {
                    name: String::from("MoscowMule"),
                    type_id: "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                        .to_string(),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let types = [
            (
                "254e1430fc69530308933b4ebd8c79569530b1835705d0bd3f0e03c155dcd09a".to_string(),
                p.clone(),
            ),
            (
                "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0".to_owned(),
                TypeDeclaration {
                    type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                        .to_owned(),
                    type_field: String::from("u64"),
                    ..Default::default()
                },
            ),
            (
                "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903".to_string(),
                TypeDeclaration {
                    type_id: "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                        .to_string(),
                    type_field: String::from("bool"),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual = expand_custom_enum(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[allow(clippy::enum_variant_names)]
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub enum MatchaTea {
                LongIsland(::core::primitive::u64),
                MoscowMule(::core::primitive::bool),
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());
        Ok(())
    }

    #[test]
    fn test_enum_with_no_variants_cannot_be_constructed() -> Result<()> {
        let p = TypeDeclaration {
            type_id: "d8fd8fe5ee8bd8aff69949e20de05efec8dee5544994542573918de2f7285641".to_string(),
            type_field: "enum SomeEmptyEnum".to_string(),
            components: Some(vec![]),
            ..Default::default()
        };
        let types = [(
            "d8fd8fe5ee8bd8aff69949e20de05efec8dee5544994542573918de2f7285641".to_string(),
            p.clone(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();

        expand_custom_enum(&FullTypeDeclaration::from_counterpart(&p, &types), false)
            .expect_err("Was able to construct an enum without variants");

        Ok(())
    }

    #[test]
    fn test_expand_struct_inside_enum() -> Result<()> {
        let inner_struct = TypeApplication {
            name: String::from("Infrastructure"),
            type_id: "316547b0bcd829a39a44d2b90761e33de0447bb705d32b65600fb42bc48c7e97".to_string(),
            ..Default::default()
        };
        let enum_components = vec![
            inner_struct,
            TypeApplication {
                name: "Service".to_string(),
                type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                    .to_string(),
                ..Default::default()
            },
        ];
        let p = TypeDeclaration {
            type_id: "a74dd441314cf1e3424b713028813f25954369c200fda4005d5cafb705d34e95".to_string(),
            type_field: String::from("enum Amsterdam"),
            components: Some(enum_components),
            ..Default::default()
        };

        let types = [
            (
                "a74dd441314cf1e3424b713028813f25954369c200fda4005d5cafb705d34e95".to_string(),
                p.clone(),
            ),
            (
                "316547b0bcd829a39a44d2b90761e33de0447bb705d32b65600fb42bc48c7e97".to_string(),
                TypeDeclaration {
                    type_id: "316547b0bcd829a39a44d2b90761e33de0447bb705d32b65600fb42bc48c7e97"
                        .to_string(),
                    type_field: String::from("struct Building"),
                    components: Some(vec![]),
                    ..Default::default()
                },
            ),
            (
                "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc".to_string(),
                TypeDeclaration {
                    type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                        .to_string(),
                    type_field: String::from("u32"),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual = expand_custom_enum(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[allow(clippy::enum_variant_names)]
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub enum Amsterdam {
                Infrastructure(self::Building),
                Service(::core::primitive::u32),
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());
        Ok(())
    }

    #[test]
    fn test_expand_array_inside_enum() -> Result<()> {
        let enum_components = vec![TypeApplication {
            name: "SomeArr".to_string(),
            type_id: "50166fa6c33d28b68d351e667ec8999314f53b69d9247ca81b4f97ebb2066c2c".to_string(),
            ..Default::default()
        }];
        let p = TypeDeclaration {
            type_id: "e8eb147e011343bc28f52b84fcc29cd93b181471f88c5c6b77f5e0dd18193fa2".to_string(),
            type_field: String::from("enum SomeEnum"),
            components: Some(enum_components),
            ..Default::default()
        };
        let types = [
            (
                "e8eb147e011343bc28f52b84fcc29cd93b181471f88c5c6b77f5e0dd18193fa2".to_string(),
                p.clone(),
            ),
            (
                "50166fa6c33d28b68d351e667ec8999314f53b69d9247ca81b4f97ebb2066c2c".to_string(),
                TypeDeclaration {
                    type_id: "50166fa6c33d28b68d351e667ec8999314f53b69d9247ca81b4f97ebb2066c2c"
                        .to_string(),
                    type_field: "[u64; 7]".to_string(),
                    components: Some(vec![TypeApplication {
                        type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                            .to_string(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
            ),
            (
                "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0".to_string(),
                TypeDeclaration {
                    type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                        .to_string(),
                    type_field: "u64".to_string(),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual = expand_custom_enum(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[allow(clippy::enum_variant_names)]
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub enum SomeEnum {
                SomeArr([::core::primitive::u64; 7usize]),
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());
        Ok(())
    }

    #[test]
    fn test_expand_custom_enum_with_enum() -> Result<()> {
        let p = TypeDeclaration {
            type_id: "737207c461421c90ae0ebcfa619d600061a5e97cc00d2e2f080784a87af1fe4a".to_string(),
            type_field: String::from("enum EnumLevel3"),
            components: Some(vec![TypeApplication {
                name: String::from("El2"),
                type_id: "457aeb74d9b1794bbcbcab5a1502b95b56ea653c01c2290bbe590e3a809da15a"
                    .to_string(),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let types = [
            (
                "737207c461421c90ae0ebcfa619d600061a5e97cc00d2e2f080784a87af1fe4a".to_string(),
                p.clone(),
            ),
            (
                "457aeb74d9b1794bbcbcab5a1502b95b56ea653c01c2290bbe590e3a809da15a".to_string(),
                TypeDeclaration {
                    type_id: "457aeb74d9b1794bbcbcab5a1502b95b56ea653c01c2290bbe590e3a809da15a"
                        .to_string(),
                    type_field: String::from("enum EnumLevel2"),
                    components: Some(vec![TypeApplication {
                        name: String::from("El1"),
                        type_id: "9d2e59a632799cbd72e445556b5e9b574e6474c6128ae673c49390e5b65fb4ec"
                            .to_string(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
            ),
            (
                "9d2e59a632799cbd72e445556b5e9b574e6474c6128ae673c49390e5b65fb4ec".to_string(),
                TypeDeclaration {
                    type_id: "9d2e59a632799cbd72e445556b5e9b574e6474c6128ae673c49390e5b65fb4ec"
                        .to_string(),
                    type_field: String::from("enum EnumLevel1"),
                    components: Some(vec![TypeApplication {
                        name: String::from("Num"),
                        type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                            .to_string(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
            ),
            (
                "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc".to_string(),
                TypeDeclaration {
                    type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                        .to_string(),
                    type_field: String::from("u32"),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual = expand_custom_enum(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[allow(clippy::enum_variant_names)]
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub enum EnumLevel3 {
                El2(self::EnumLevel2),
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());
        Ok(())
    }

    #[test]
    fn test_expand_custom_struct() -> Result<()> {
        let p = TypeDeclaration {
            type_id: "d24355b16e923631e80d2ef3c2798faedff4df7987f62a1bcb42cb249019f17f".to_string(),
            type_field: String::from("struct Cocktail"),
            components: Some(vec![
                TypeApplication {
                    name: String::from("long_island"),
                    type_id: "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                        .to_string(),
                    ..Default::default()
                },
                TypeApplication {
                    name: String::from("cosmopolitan"),
                    type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                        .to_owned(),
                    ..Default::default()
                },
                TypeApplication {
                    name: String::from("mojito"),
                    type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                        .to_string(),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let types = [
            (
                "d24355b16e923631e80d2ef3c2798faedff4df7987f62a1bcb42cb249019f17f".to_string(),
                p.clone(),
            ),
            (
                "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903".to_string(),
                TypeDeclaration {
                    type_id: "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                        .to_string(),
                    type_field: String::from("bool"),
                    ..Default::default()
                },
            ),
            (
                "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0".to_owned(),
                TypeDeclaration {
                    type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                        .to_owned(),
                    type_field: String::from("u64"),
                    ..Default::default()
                },
            ),
            (
                "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc".to_string(),
                TypeDeclaration {
                    type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                        .to_string(),
                    type_field: String::from("u32"),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual =
            expand_custom_struct(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub struct Cocktail {
                pub long_island: ::core::primitive::bool,
                pub cosmopolitan: ::core::primitive::u64,
                pub mojito: ::core::primitive::u32,
            }
            impl Cocktail {
                pub fn new(
                    long_island: ::core::primitive::bool,
                    cosmopolitan: ::core::primitive::u64,
                    mojito: ::core::primitive::u32,
                ) -> Self {
                    Self {
                        long_island,
                        cosmopolitan,
                        mojito,
                    }
                }
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());

        Ok(())
    }

    #[test]
    fn test_struct_with_no_fields_can_be_constructed() -> Result<()> {
        let p = TypeDeclaration {
            type_id: "435bf4ae7355c5f2de2c011e5964bd06feaeee0c413998869c68a7c0ae611cff".to_string(),
            type_field: "struct SomeEmptyStruct".to_string(),
            components: Some(vec![]),
            ..Default::default()
        };
        let types = [(
            "435bf4ae7355c5f2de2c011e5964bd06feaeee0c413998869c68a7c0ae611cff".to_string(),
            p.clone(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual =
            expand_custom_struct(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::core::default::Default,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub struct SomeEmptyStruct {}
            impl SomeEmptyStruct {
                pub fn new() -> Self {
                    Self {}
                }
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());

        Ok(())
    }

    #[test]
    fn test_expand_custom_struct_with_struct() -> Result<()> {
        let p = TypeDeclaration {
            type_id: "d24355b16e923631e80d2ef3c2798faedff4df7987f62a1bcb42cb249019f17f".to_string(),
            type_field: String::from("struct Cocktail"),
            components: Some(vec![
                TypeApplication {
                    name: String::from("long_island"),
                    type_id: "85ada57d27badeb25270f015d6277a925b72c429c01408abe02331fe52ceea20"
                        .to_string(),
                    ..Default::default()
                },
                TypeApplication {
                    name: String::from("mojito"),
                    type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                        .to_string(),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let types = [
            (
                "d24355b16e923631e80d2ef3c2798faedff4df7987f62a1bcb42cb249019f17f".to_string(),
                p.clone(),
            ),
            (
                "85ada57d27badeb25270f015d6277a925b72c429c01408abe02331fe52ceea20".to_string(),
                TypeDeclaration {
                    type_id: "85ada57d27badeb25270f015d6277a925b72c429c01408abe02331fe52ceea20"
                        .to_string(),
                    type_field: String::from("struct Shaker"),
                    components: Some(vec![]),
                    ..Default::default()
                },
            ),
            (
                "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc".to_string(),
                TypeDeclaration {
                    type_id: "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc"
                        .to_string(),
                    type_field: String::from("u32"),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let actual =
            expand_custom_struct(&FullTypeDeclaration::from_counterpart(&p, &types), false)?;

        let expected = quote! {
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub struct Cocktail {
                pub long_island: self::Shaker,
                pub mojito: ::core::primitive::u32,
            }
            impl Cocktail {
                pub fn new(long_island: self::Shaker, mojito: ::core::primitive::u32,) -> Self {
                    Self {
                        long_island,
                        mojito,
                    }
                }
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());
        Ok(())
    }

    #[test]
    fn test_expand_struct_new_abi() -> Result<()> {
        let s = r#"
            {
                "types": [
                  {
                    "typeId": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                    "type": "u64",
                    "components": null,
                    "typeParameters": null
                  },
                  {
                    "typeId": "7c5ee1cecf5f8eacd1284feb5f0bf2bdea533a51e2f0c9aabe9236d335989f3b",
                    "type": "b256",
                    "components": null,
                    "typeParameters": null
                  },
                  {
                    "typeId": "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903",
                    "type": "bool",
                    "components": null,
                    "typeParameters": null
                  },
                  {
                    "typeId": "5599571157f54ae755e14c9acc667a8a7ebc9e723da12e7f35e9ed76f31153b1",
                    "type": "struct MyStruct1",
                    "components": [
                      {
                        "name": "x",
                        "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                        "typeArguments": null
                      },
                      {
                        "name": "y",
                        "type": "7c5ee1cecf5f8eacd1284feb5f0bf2bdea533a51e2f0c9aabe9236d335989f3b",
                        "typeArguments": null
                      }
                    ],
                    "typeParameters": null
                  },
                  {
                    "typeId": "535db000d52247639d2b0d6b9e55680642847fe98fab7e63f4e775bbdff1a351",
                    "type": "struct MyStruct2",
                    "components": [
                      {
                        "name": "x",
                        "type": "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903",
                        "typeArguments": null
                      },
                      {
                        "name": "y",
                        "type": "5599571157f54ae755e14c9acc667a8a7ebc9e723da12e7f35e9ed76f31153b1",
                        "typeArguments": []
                      }
                    ],
                    "typeParameters": null
                  }
                ],
                "functions": [
                  {
                    "type": "function",
                    "inputs": [],
                    "name": "some_abi_funct",
                    "output": {
                      "name": "",
                       "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                      "typeArguments": []
                    }
                  }
                ]
            }"#;
        let parsed_abi: ProgramABI = serde_json::from_str(s)?;
        let types = parsed_abi
            .types
            .into_iter()
            .map(|t| (t.type_id.clone(), t))
            .collect::<HashMap<String, TypeDeclaration>>();

        let s1 = types
            .get("5599571157f54ae755e14c9acc667a8a7ebc9e723da12e7f35e9ed76f31153b1")
            .unwrap();

        let actual =
            expand_custom_struct(&FullTypeDeclaration::from_counterpart(s1, &types), false)?;

        let expected = quote! {
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub struct MyStruct1 {
                pub x: ::core::primitive::u64,
                pub y: ::fuels::types::Bits256,
            }
            impl MyStruct1 {
                pub fn new(x: ::core::primitive::u64, y: ::fuels::types::Bits256,) -> Self {
                    Self { x, y, }
                }
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());

        let s2 = types
            .get("535db000d52247639d2b0d6b9e55680642847fe98fab7e63f4e775bbdff1a351")
            .unwrap();

        let actual =
            expand_custom_struct(&FullTypeDeclaration::from_counterpart(s2, &types), false)?;

        let expected = quote! {
            #[derive(
                Clone,
                Debug,
                Eq,
                PartialEq,
                ::fuels::macros::Parameterize,
                ::fuels::macros::Tokenizable,
                ::fuels::macros::TryFrom,
            )]
            pub struct MyStruct2 {
                pub x: ::core::primitive::bool,
                pub y: self::MyStruct1,
            }
            impl MyStruct2 {
                pub fn new(x: ::core::primitive::bool, y: self::MyStruct1,) -> Self {
                    Self { x, y, }
                }
            }
        };

        assert_eq!(actual.code().to_string(), expected.to_string());

        Ok(())
    }

    #[test]
    fn shared_types_are_just_reexported() {
        // given
        let type_decl = FullTypeDeclaration {
            type_field: "struct some_shared_lib::SharedStruct".to_string(),
            components: vec![],
            type_parameters: vec![],
        };
        let shared_types = HashSet::from([type_decl.clone()]);

        // when
        let generated_code = generate_types(&[type_decl], &shared_types, false).unwrap();

        // then
        let expected_code = quote! {
            #[allow(clippy::too_many_arguments)]
            #[no_implicit_prelude]
            pub mod some_shared_lib {
                use ::core::{
                    clone::Clone,
                    convert::{Into, TryFrom, From},
                    iter::IntoIterator,
                    iter::Iterator,
                    marker::Sized,
                    panic,
                };

                use ::std::{string::ToString, format, vec, default::Default};
                pub use super::super::shared_types::some_shared_lib::SharedStruct;
            }
        };

        assert_eq!(generated_code.code().to_string(), expected_code.to_string());
    }
}
