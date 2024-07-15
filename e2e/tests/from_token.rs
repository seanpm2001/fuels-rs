use std::str::FromStr;

use fuels::{core::traits::Tokenizable, prelude::*, types::Token};

pub fn null_contract_id() -> Bech32ContractId {
    // a bech32 contract address that decodes to [0u8;32]
    Bech32ContractId::from_str("fuel1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsx2mt2")
        .unwrap()
}

#[tokio::test]
async fn create_struct_from_decoded_tokens() -> Result<()> {
    // Generates the bindings from an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(Contract(
        name = "SimpleContract",
        abi = r#"
        {
            "programType": "contract",
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
                "typeId": "c09f518d252533423934354a0974a7894bc99afbb03eb6f0956def50ae4146f0",
                "type": "struct MyStruct",
                "components": [
                  {
                    "name": "foo",
                    "type": "c89951a24c6ca28c13fd1cfdc646b2b656d69e61a92b91023be7eb58eb914b6b",
                    "typeArguments": null
                  },
                  {
                    "name": "bar",
                    "type": "7c5ee1cecf5f8eacd1284feb5f0bf2bdea533a51e2f0c9aabe9236d335989f3b",
                    "typeArguments": null
                  }
                ],
                "typeParameters": null
              },
              {
                "typeId": "c89951a24c6ca28c13fd1cfdc646b2b656d69e61a92b91023be7eb58eb914b6b",
                "type": "u8",
                "components": null,
                "typeParameters": null
              }
            ],
            "functions": [
              {
                "inputs": [
                  {
                    "name": "my_val",
                    "type": "c09f518d252533423934354a0974a7894bc99afbb03eb6f0956def50ae4146f0",
                    "typeArguments": null
                  }
                ],
                "name": "takes_struct",
                "output": {
                  "name": "",
                  "type": "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d",
                  "typeArguments": null
                }
              }
            ]
          }
        "#,
    ));

    // Decoded tokens
    let u8_token = Token::U8(10);
    let bool_token = Token::Bool(true);

    // Create the struct using the decoded tokens.
    // `struct_from_tokens` is of type `MyStruct`.
    let struct_from_tokens = MyStruct::from_token(Token::Struct(vec![u8_token, bool_token]))?;

    assert_eq!(10, struct_from_tokens.foo);
    assert!(struct_from_tokens.bar);

    Ok(())
}

#[tokio::test]
async fn create_nested_struct_from_decoded_tokens() -> Result<()> {
    // Generates the bindings from the an ABI definition inline.
    // The generated bindings can be accessed through `SimpleContract`.
    abigen!(Contract(
        name = "SimpleContract",
        abi = r#"
        {
            "programType": "contract",
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
                "typeId": "a74273d5c9a1a2a57628cc8a418d741c6de337f2ea9335bee76c61a45e7a4669",
                "type": "struct InnerStruct",
                "components": [
                  {
                    "name": "a",
                    "type": "7c5ee1cecf5f8eacd1284feb5f0bf2bdea533a51e2f0c9aabe9236d335989f3b",
                    "typeArguments": null
                  }
                ],
                "typeParameters": null
              },
              {
                "typeId": "56ed4b87478d6f24d6fea9034e8264b3688e31c4502ec201b4017fef95fddd6b",
                "type": "struct MyNestedStruct",
                "components": [
                  {
                    "name": "x",
                    "type": "29881aad8730c5ab11d275376323d8e4ff4179aae8ccb6c13fe4902137e162ef",
                    "typeArguments": null
                  },
                  {
                    "name": "foo",
                    "type": "a74273d5c9a1a2a57628cc8a418d741c6de337f2ea9335bee76c61a45e7a4669",
                    "typeArguments": null
                  }
                ],
                "typeParameters": null
              },
              {
                "typeId": "29881aad8730c5ab11d275376323d8e4ff4179aae8ccb6c13fe4902137e162ef",
                "type": "u16",
                "components": null,
                "typeParameters": null
              }
            ],
            "functions": [
              {
                "inputs": [
                  {
                    "name": "top_value",
                    "type": "56ed4b87478d6f24d6fea9034e8264b3688e31c4502ec201b4017fef95fddd6b",
                    "typeArguments": null
                  }
                ],
                "name": "takes_nested_struct",
                "output": {
                  "name": "",
                  "type": "2e38e77b22c314a449e91fafed92a43826ac6aa403ae6a8acb6cf58239fbaf5d",
                  "typeArguments": null
                }
              }
            ]
          }
        "#,
    ));

    // Creating just the InnerStruct is possible
    let a = Token::Bool(true);
    let inner_struct_token = Token::Struct(vec![a.clone()]);
    let inner_struct_from_tokens = InnerStruct::from_token(inner_struct_token.clone())?;
    assert!(inner_struct_from_tokens.a);

    // Creating the whole nested struct `MyNestedStruct`
    // from tokens.
    // `x` is the token for the field `x` in `MyNestedStruct`
    // `a` is the token for the field `a` in `InnerStruct`
    let x = Token::U16(10);

    let nested_struct_from_tokens =
        MyNestedStruct::from_token(Token::Struct(vec![x, inner_struct_token]))?;

    assert_eq!(10, nested_struct_from_tokens.x);
    assert!(nested_struct_from_tokens.foo.a);

    Ok(())
}
