extern crate alloc;

#[cfg(test)]
mod tests {
    use std::{default::Default, str::FromStr};

    use fuels::{
        accounts::predicate::Predicate,
        core::{codec::ABIEncoder, traits::Tokenizable},
        macros::wasm_abigen,
        types::{bech32::Bech32Address, errors::Result},
    };
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn decoding_and_encoding() -> Result<()> {
        wasm_abigen!(Contract(
            name = "no_name",
            abi = r#"
                {
                  "programType": "script",
                  "specVersion": "0.0.0",
                  "abiVersion": "0.0.0",
                  "types": [
                    {
                      "typeId": "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d",
                      "type": "()",
                      "components": [],
                      "typeParameters": null
                    },
                    {
                      "typeId": "7c5ee1cecf5f8eacd1284feb5f0bf2bdea533a51e2f0c9aabe9236d335989f3b",
                      "type": "bool",
                      "components": null,
                      "typeParameters": null
                    },
                    {
                      "typeId": "e8eb147e011343bc28f52b84fcc29cd93b181471f88c5c6b77f5e0dd18193fa2",
                      "type": "enum SomeEnum",
                      "components": [
                        {
                          "name": "V1",
                          "type": "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d",
                          "typeArguments": null
                        },
                        {
                          "name": "V2",
                          "type": "8b8c08c464656c9a4b876c13199929c5ceb37ff6c927eaeefd756c12278e98c5",
                          "typeArguments": null
                        }
                      ],
                      "typeParameters": [
                      "8b8c08c464656c9a4b876c13199929c5ceb37ff6c927eaeefd756c12278e98c5"
                      ]
                    },
                    {
                      "typeId": "8b8c08c464656c9a4b876c13199929c5ceb37ff6c927eaeefd756c12278e98c5",
                      "type": "generic T",
                      "components": null,
                      "typeParameters": null
                    },
                    {
                      "typeId": "c672b07b5808bcc04715d73ca6d42eaabd332266144c1017c20833ef05a4a484",
                      "type": "struct SomeStruct",
                      "components": [
                        {
                          "name": "a",
                          "type": "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc",
                          "typeArguments": null
                        },
                        {
                          "name": "b",
                          "type": "7c5ee1cecf5f8eacd1284feb5f0bf2bdea533a51e2f0c9aabe9236d335989f3b",
                          "typeArguments": null
                        }
                      ],
                      "typeParameters": null
                    },
                    {
                      "typeId": "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc",
                      "type": "u32",
                      "components": null,
                      "typeParameters": null
                    }
                  ],
                  "functions": [
                    {
                      "inputs": [
                        {
                          "name": "arg",
                          "type": "e8eb147e011343bc28f52b84fcc29cd93b181471f88c5c6b77f5e0dd18193fa2",
                          "typeArguments": [
                            {
                              "name": "",
                              "type": "c672b07b5808bcc04715d73ca6d42eaabd332266144c1017c20833ef05a4a484",
                              "typeArguments": null
                            }
                          ]
                        }
                      ],
                      "name": "test_function",
                      "output": {
                        "name": "",
                        "type": "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d",
                        "typeArguments": null
                      },
                      "attributes": null
                    }
                  ],
                  "loggedTypes": [],
                  "messagesTypes": [],
                  "configurables": []
        }"#
        ));

        let original = SomeEnum::V2(SomeStruct { a: 123, b: false });

        let bytes = ABIEncoder::default().encode(&[original.clone().into_token()])?;

        let expected_bytes = [
            0, 0, 0, 0, 0, 0, 0, 1, // enum discriminant
            0, 0, 0, 123, 0, // SomeStruct
        ]
        .to_vec();

        assert_eq!(expected_bytes, bytes);

        let reconstructed = bytes.try_into().unwrap();

        assert_eq!(original, reconstructed);

        Ok(())
    }

    #[wasm_bindgen_test]
    fn predicate_abigen() -> Result<()> {
        wasm_abigen!(Predicate(
            name = "MyPredicate",
            abi = r#"
                    {
                      "programType": "script",
                      "specVersion": "0.0.0",
                      "abiVersion": "0.0.0",
                      "types": [
                        {
                          "typeId": "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903",
                          "type": "bool",
                          "components": null,
                          "typeParameters": null
                        },
                        {
                          "typeId": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                          "type": "u64",
                          "components": null,
                          "typeParameters": null
                        }
                      ],
                      "functions": [
                        {
                          "inputs": [
                            {
                              "name": "arg",
                              "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                              "typeArguments": null
                            }
                          ],
                          "name": "main",
                          "output": {
                            "name": "",
                            "type": "b760f44fa5965c2474a3b471467a22c43185152129295af588b022ae50b50903",
                            "typeArguments": null
                          },
                          "attributes": null
                        }
                      ],
                      "loggedTypes": [],
                      "messagesTypes": [],
                      "configurables": [
                        {
                          "name": "U64",
                          "configurableType": {
                            "name": "",
                            "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                            "typeArguments": null
                          },
                          "offset": 100
                        }
                      ]
                    }"#
        ));

        let code = vec![
            116, 0, 0, 3, 71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 93, 252, 192, 1, 16, 255, 243, 0,
            26, 236, 80, 0, 145, 0, 0, 0, 113, 68, 0, 3, 97, 73, 17, 1, 118, 72, 0, 2, 97, 65, 17,
            13, 116, 0, 0, 7, 114, 76, 0, 2, 19, 73, 36, 192, 90, 73, 32, 1, 118, 72, 0, 2, 97, 65,
            17, 31, 116, 0, 0, 1, 36, 0, 0, 0, 93, 65, 0, 0, 93, 71, 240, 0, 19, 65, 4, 64, 36, 64,
            0, 0, 71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42,
        ];
        let value = 128;

        let predicate_data = MyPredicateEncoder::default().encode_data(value)?;
        let configurables = MyPredicateConfigurables::default().with_U64(value)?;

        let predicate: Predicate = Predicate::from_code(code.clone())
            .with_data(predicate_data)
            .with_configurables(configurables);

        let mut expected_code = code.clone();
        *expected_code.last_mut().unwrap() = value as u8;

        assert_eq!(*predicate.code(), expected_code);

        let expected_address = Bech32Address::from_str(
            "fuel14z2xsxcp47z9zfhj9atrmd66ujvwy8ujgn4j0xsh95fjh2px4mcq4f7k3w",
        )?;

        assert_eq!(*predicate.address(), expected_address);

        Ok(())
    }
}
