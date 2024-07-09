use fuel_abi_types::abi::full_program::{FullABIFunction, FullProgramABI};
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};

use crate::{
    error::Result,
    program_bindings::{
        abigen::{
            bindings::function_generator::FunctionGenerator,
            configurables::generate_code_for_configurable_constants,
            logs::log_formatters_instantiation_code,
        },
        generated_code::GeneratedCode,
    },
    utils::{ident, TypePath},
};

pub(crate) fn contract_bindings(
    name: &Ident,
    abi: FullProgramABI,
    no_std: bool,
) -> Result<GeneratedCode> {
    if no_std {
        return Ok(GeneratedCode::default());
    }

    let log_formatters =
        log_formatters_instantiation_code(quote! {contract_id.clone().into()}, &abi.logged_types);

    let methods_name = ident(&format!("{name}Methods"));

    let contract_functions = expand_functions(&abi.functions)?;

    let configuration_struct_name = ident(&format!("{name}Configurables"));
    let constant_configuration_code =
        generate_code_for_configurable_constants(&configuration_struct_name, &abi.configurables)?;

    let code = quote! {
        #[derive(Debug, Clone)]
        pub struct #name<A: ::fuels::accounts::Account> {
            contract_id: ::fuels::types::bech32::Bech32ContractId,
            account: A,
            log_decoder: ::fuels::core::codec::LogDecoder,
            encoder_config: ::fuels::core::codec::EncoderConfig,
        }

        impl<A: ::fuels::accounts::Account> #name<A>
        {
            pub fn new(
                contract_id: impl ::core::convert::Into<::fuels::types::bech32::Bech32ContractId>,
                account: A,
            ) -> Self {
                let contract_id: ::fuels::types::bech32::Bech32ContractId = contract_id.into();
                let log_decoder = ::fuels::core::codec::LogDecoder::new(#log_formatters);
                let encoder_config = ::fuels::core::codec::EncoderConfig::default();
                Self { contract_id, account, log_decoder, encoder_config }
            }

            pub fn contract_id(&self) -> &::fuels::types::bech32::Bech32ContractId {
                &self.contract_id
            }

            pub fn account(&self) -> A {
                self.account.clone()
            }

            pub fn with_account<U: ::fuels::accounts::Account>(self, account: U)
            -> #name<U> {
                #name {
                        contract_id: self.contract_id,
                        account,
                        log_decoder: self.log_decoder,
                        encoder_config: self.encoder_config
                }
            }

            pub fn with_encoder_config(mut self, encoder_config: ::fuels::core::codec::EncoderConfig)
            -> #name::<A> {
                self.encoder_config = encoder_config;

                self
            }

            pub async fn get_balances(&self) -> ::fuels::types::errors::Result<::std::collections::HashMap<::fuels::types::AssetId, u64>> {
                ::fuels::accounts::ViewOnlyAccount::try_provider(&self.account)?
                                  .get_contract_balances(&self.contract_id)
                                  .await
                                  .map_err(::std::convert::Into::into)
            }

            pub fn methods(&self) -> #methods_name<A> {
                #methods_name {
                    contract_id: self.contract_id.clone(),
                    account: self.account.clone(),
                    log_decoder: self.log_decoder.clone(),
                    encoder_config: self.encoder_config.clone(),
                }
            }
        }

        // Implement struct that holds the contract methods
        pub struct #methods_name<A: ::fuels::accounts::Account> {
            contract_id: ::fuels::types::bech32::Bech32ContractId,
            account: A,
            log_decoder: ::fuels::core::codec::LogDecoder,
            encoder_config: ::fuels::core::codec::EncoderConfig,
        }

        impl<A: ::fuels::accounts::Account> #methods_name<A> {
            #contract_functions
        }

        impl<A: ::fuels::accounts::Account>
            ::fuels::programs::calls::ContractDependency for #name<A>
        {
            fn id(&self) -> ::fuels::types::bech32::Bech32ContractId {
                self.contract_id.clone()
            }

            fn log_decoder(&self) -> ::fuels::core::codec::LogDecoder {
                self.log_decoder.clone()
            }
        }

        #constant_configuration_code
    };

    // All publicly available types generated above should be listed here.
    let type_paths = [name, &methods_name, &configuration_struct_name]
        .map(|type_name| TypePath::new(type_name).expect("We know the given types are not empty"))
        .into_iter()
        .collect();

    Ok(GeneratedCode::new(code, type_paths, no_std))
}

fn expand_functions(functions: &[FullABIFunction]) -> Result<TokenStream> {
    functions
        .iter()
        .map(expand_fn)
        .fold_ok(TokenStream::default(), |mut all_code, code| {
            all_code.append_all(code);
            all_code
        })
}

/// Transforms a function defined in [`FullABIFunction`] into a [`TokenStream`]
/// that represents that same function signature as a Rust-native function
/// declaration.
pub(crate) fn expand_fn(abi_fun: &FullABIFunction) -> Result<TokenStream> {
    let mut generator = FunctionGenerator::new(abi_fun)?;

    generator.set_docs(abi_fun.doc_strings()?);

    let original_output = generator.output_type();
    generator.set_output_type(
        quote! {::fuels::programs::calls::CallHandler<A, ::fuels::programs::calls::ContractCall, #original_output> },
    );

    let fn_selector = generator.fn_selector();
    let arg_tokens = generator.tokenized_args();
    let is_payable = abi_fun.is_payable();
    let body = quote! {
            ::fuels::programs::calls::CallHandler::new_contract_call(
                self.contract_id.clone(),
                self.account.clone(),
                #fn_selector,
                &#arg_tokens,
                self.log_decoder.clone(),
                #is_payable,
                self.encoder_config.clone(),
            )
    };
    generator.set_body(body);

    Ok(generator.generate())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use fuel_abi_types::abi::{
        full_program::FullABIFunction,
        program::{ABIFunction, Attribute, ProgramABI, TypeApplication, TypeDeclaration},
    };
    use pretty_assertions::assert_eq;
    use quote::quote;

    use crate::{error::Result, program_bindings::abigen::bindings::contract::expand_fn};

    #[test]
    fn expand_contract_method_simple_abi() -> Result<()> {
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
                  }
                ],
                "functions": [
                  {
                    "type": "function",
                    "inputs": [
                      {
                        "name": "s1",
                        "type": "5599571157f54ae755e14c9acc667a8a7ebc9e723da12e7f35e9ed76f31153b1",
                        "typeArguments": []
                      },
                      {
                        "name": "s2",
                        "type": "535db000d52247639d2b0d6b9e55680642847fe98fab7e63f4e775bbdff1a351",
                        "typeArguments": []
                      }
                    ],
                    "name": "some_abi_funct",
                    "output": {
                      "name": "",
                      "type": "5599571157f54ae755e14c9acc667a8a7ebc9e723da12e7f35e9ed76f31153b1",
                      "typeArguments": []
                    },
                    "attributes": [
                      {
                        "name": "doc-comment",
                        "arguments": [
                          "This is a doc string"
                        ]
                      }
                    ]
                  }
                ]
              }
    "#;
        let parsed_abi: ProgramABI = serde_json::from_str(s)?;
        let types = parsed_abi
            .types
            .into_iter()
            .map(|t| (t.type_id.clone(), t))
            .collect::<HashMap<String, TypeDeclaration>>();

        // Grabbing the one and only function in it.
        let result = expand_fn(&FullABIFunction::from_counterpart(
            &parsed_abi.functions[0],
            &types,
        )?)?;

        let expected = quote! {
            #[doc = "This is a doc string"]
            pub fn some_abi_funct(
                &self,
                s_1: self::MyStruct1,
                s_2: self::MyStruct2
            ) -> ::fuels::programs::calls::CallHandler<A, ::fuels::programs::calls::ContractCall, self::MyStruct1> {
                ::fuels::programs::calls::CallHandler::new_contract_call(
                    self.contract_id.clone(),
                    self.account.clone(),
                    ::fuels::core::codec::encode_fn_selector("some_abi_funct"),
                    &[
                        ::fuels::core::traits::Tokenizable::into_token(s_1),
                        ::fuels::core::traits::Tokenizable::into_token(s_2)
                    ],
                    self.log_decoder.clone(),
                    false,
                    self.encoder_config.clone(),
                )
            }
        };

        assert_eq!(result.to_string(), expected.to_string());

        Ok(())
    }

    #[test]
    fn expand_contract_method_simple() -> Result<()> {
        let the_function = ABIFunction {
            inputs: vec![TypeApplication {
                name: "bimbam".to_string(),
                type_id: "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                    .to_string(),
                ..Default::default()
            }],
            name: "HelloWorld".to_string(),
            attributes: Some(vec![Attribute {
                name: "doc-comment".to_string(),
                arguments: vec!["This is a doc string".to_string()],
            }]),
            ..Default::default()
        };
        let types = [
            (
                "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d".to_string(),
                TypeDeclaration {
                    type_id: "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d"
                        .to_string(),
                    type_field: String::from("()"),
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
        let result = expand_fn(&FullABIFunction::from_counterpart(&the_function, &types)?);

        let expected = quote! {
            #[doc = "This is a doc string"]
            pub fn HelloWorld(&self, bimbam: ::core::primitive::bool) -> ::fuels::programs::calls::CallHandler<A, ::fuels::programs::calls::ContractCall, ()> {
                ::fuels::programs::calls::CallHandler::new_contract_call(
                    self.contract_id.clone(),
                    self.account.clone(),
                    ::fuels::core::codec::encode_fn_selector("HelloWorld"),
                    &[::fuels::core::traits::Tokenizable::into_token(bimbam)],
                    self.log_decoder.clone(),
                    false,
                    self.encoder_config.clone(),
                )
            }
        };

        assert_eq!(result?.to_string(), expected.to_string());

        Ok(())
    }

    #[test]
    fn expand_contract_method_complex() -> Result<()> {
        // given
        let the_function = ABIFunction {
            inputs: vec![TypeApplication {
                name: String::from("the_only_allowed_input"),
                type_id: "f45c66ebab65ce69b034b71b019458b77ab72a1194a78f9d79150e5b4e2eb60c"
                    .to_string(),
                ..Default::default()
            }],
            name: "hello_world".to_string(),
            output: TypeApplication {
                name: String::from("stillnotused"),
                type_id: "9d2926e7f0e9641f5feb6c2325f440ddfe5b0e2d8ab8e3612ef5667e1ea79310"
                    .to_string(),
                ..Default::default()
            },
            attributes: Some(vec![
                Attribute {
                    name: "doc-comment".to_string(),
                    arguments: vec!["This is a doc string".to_string()],
                },
                Attribute {
                    name: "doc-comment".to_string(),
                    arguments: vec!["This is another doc string".to_string()],
                },
            ]),
        };
        let types = [
            (
                "9d2926e7f0e9641f5feb6c2325f440ddfe5b0e2d8ab8e3612ef5667e1ea79310".to_string(),
                TypeDeclaration {
                    type_id: "9d2926e7f0e9641f5feb6c2325f440ddfe5b0e2d8ab8e3612ef5667e1ea79310"
                        .to_string(),
                    type_field: String::from("enum EntropyCirclesEnum"),
                    components: Some(vec![
                        TypeApplication {
                            name: String::from("Postcard"),
                            type_id:
                                "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                                    .to_string(),
                            ..Default::default()
                        },
                        TypeApplication {
                            name: String::from("Teacup"),
                            type_id:
                                "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                                    .to_string(),
                            ..Default::default()
                        },
                    ]),
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
            (
                "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0".to_string(),
                TypeDeclaration {
                    type_id: "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                        .to_string(),
                    type_field: String::from("u64"),
                    ..Default::default()
                },
            ),
            (
                "f45c66ebab65ce69b034b71b019458b77ab72a1194a78f9d79150e5b4e2eb60c".to_string(),
                TypeDeclaration {
                    type_id: "f45c66ebab65ce69b034b71b019458b77ab72a1194a78f9d79150e5b4e2eb60c"
                        .to_string(),
                    type_field: String::from("struct SomeWeirdFrenchCuisine"),
                    components: Some(vec![
                        TypeApplication {
                            name: String::from("Beef"),
                            type_id:
                                "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903"
                                    .to_string(),
                            ..Default::default()
                        },
                        TypeApplication {
                            name: String::from("BurgundyWine"),
                            type_id:
                                "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0"
                                    .to_string(),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        // when
        let result = expand_fn(&FullABIFunction::from_counterpart(&the_function, &types)?);

        //then

        // Some more editing was required because it is not rustfmt-compatible (adding/removing parentheses or commas)
        let expected = quote! {
            #[doc = "This is a doc string"]
            #[doc = "This is another doc string"]
            pub fn hello_world(
                &self,
                the_only_allowed_input: self::SomeWeirdFrenchCuisine
            ) -> ::fuels::programs::calls::CallHandler<A, ::fuels::programs::calls::ContractCall, self::EntropyCirclesEnum> {
                ::fuels::programs::calls::CallHandler::new_contract_call(
                    self.contract_id.clone(),
                    self.account.clone(),
                    ::fuels::core::codec::encode_fn_selector( "hello_world"),
                    &[::fuels::core::traits::Tokenizable::into_token(
                        the_only_allowed_input
                    )],
                    self.log_decoder.clone(),
                    false,
                    self.encoder_config.clone(),
                )
            }
        };

        assert_eq!(result?.to_string(), expected.to_string());

        Ok(())
    }
}
