#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[cfg(feature = "fuel-core-lib")]
    use fuels::prelude::NodeConfig;
    use fuels::{
        prelude::Result,
        types::{Bits256, EvmAddress, Identity},
    };

    #[tokio::test]
    async fn bytes32() -> Result<()> {
        // ANCHOR: bytes32
        use std::str::FromStr;

        use fuels::types::Bytes32;

        // Zeroed Bytes32
        let b256 = Bytes32::zeroed();

        // Grab the inner `[u8; 32]` from
        // `Bytes32` by dereferencing (i.e. `*`) it.
        assert_eq!([0u8; 32], *b256);

        // From a `[u8; 32]`.
        // ANCHOR: array_to_bytes32
        let my_slice = [1u8; 32];
        let b256 = Bytes32::new(my_slice);
        // ANCHOR_END: array_to_bytes32
        assert_eq!([1u8; 32], *b256);

        // From a hex string.
        // ANCHOR: hex_string_to_bytes32
        let hex_str = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let b256 = Bytes32::from_str(hex_str)?;
        // ANCHOR_END: hex_string_to_bytes32
        assert_eq!([0u8; 32], *b256);
        // ANCHOR_END: bytes32

        // ANCHOR: bytes32_format
        let b256_string = b256.to_string();
        let b256_hex_string = format!("{b256:#x}");
        // ANCHOR_END: bytes32_format

        assert_eq!(hex_str[2..], b256_string);
        assert_eq!(hex_str, b256_hex_string);

        // ANCHOR: bytes32_to_str
        let _str_from_bytes32: &str = b256.to_string().as_str();
        // ANCHOR_END: bytes32_to_str

        Ok(())
    }
    #[tokio::test]
    async fn address() -> Result<()> {
        // ANCHOR: address
        use std::str::FromStr;

        use fuels::types::Address;

        // Zeroed Bytes32
        let address = Address::zeroed();

        // Grab the inner `[u8; 32]` from
        // `Bytes32` by dereferencing (i.e. `*`) it.
        assert_eq!([0u8; 32], *address);

        // From a `[u8; 32]`.
        // ANCHOR: array_to_address
        let my_slice = [1u8; 32];
        let address = Address::new(my_slice);
        // ANCHOR_END: array_to_address
        assert_eq!([1u8; 32], *address);

        // From a string.
        // ANCHOR: hex_string_to_address
        let hex_str = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let address = Address::from_str(hex_str)?;
        // ANCHOR_END: hex_string_to_address
        assert_eq!([0u8; 32], *address);
        // ANCHOR_END: address

        // ANCHOR: address_to_identity
        let _identity_from_address = Identity::Address(address);
        // ANCHOR_END: address_to_identity

        // ANCHOR: address_to_str
        let _str_from_address: &str = address.to_string().as_str();
        // ANCHOR_END: address_to_str

        // ANCHOR: address_to_bits256
        let bits_256 = Bits256(address.into());
        // ANCHOR_END: address_to_bits256

        // ANCHOR: b256_to_evm_address
        let _evm_address = EvmAddress::from(bits_256);
        // ANCHOR_END: b256_to_evm_address

        Ok(())
    }
    #[tokio::test]
    async fn bech32() -> Result<()> {
        // ANCHOR: bech32
        use fuels::types::{bech32::Bech32Address, Address, Bytes32};

        // New from HRP string and a hash
        // ANCHOR: array_to_bech32
        let hrp = "fuel";
        let my_slice = [1u8; 32];
        let _bech32_address = Bech32Address::new(hrp, my_slice);
        // ANCHOR_END: array_to_bech32

        // Note that you can also pass a hash stored as Bytes32 to new:
        // ANCHOR: bytes32_to_bech32
        let my_hash = Bytes32::new([1u8; 32]);
        let _bech32_address = Bech32Address::new(hrp, my_hash);
        // ANCHOR_END: bytes32_to_bech32

        // From a string.
        // ANCHOR: str_to_bech32
        let address = "fuel1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsx2mt2";
        let bech32_address = Bech32Address::from_str(address)?;
        // ANCHOR_END: str_to_bech32
        assert_eq!([0u8; 32], *bech32_address.hash());

        // From Address
        // ANCHOR: address_to_bech32
        let plain_address = Address::new([0u8; 32]);
        let bech32_address = Bech32Address::from(plain_address);
        // ANCHOR_END: address_to_bech32
        assert_eq!([0u8; 32], *bech32_address.hash());

        // Convert to Address
        // ANCHOR: bech32_to_address
        let _plain_address: Address = bech32_address.into();
        // ANCHOR_END: bech32_to_address

        // ANCHOR_END: bech32

        Ok(())
    }
    #[tokio::test]
    async fn asset_id() -> Result<()> {
        // ANCHOR: asset_id
        use std::str::FromStr;

        use fuels::types::AssetId;

        // Zeroed Bytes32
        let asset_id = AssetId::zeroed();

        // Grab the inner `[u8; 32]` from
        // `Bytes32` by dereferencing (i.e. `*`) it.
        assert_eq!([0u8; 32], *asset_id);

        // From a `[u8; 32]`.
        // ANCHOR: array_to_asset_id
        let my_slice = [1u8; 32];
        let asset_id = AssetId::new(my_slice);
        // ANCHOR_END: array_to_asset_id
        assert_eq!([1u8; 32], *asset_id);

        // From a string.
        // ANCHOR: string_to_asset_id
        let hex_str = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let asset_id = AssetId::from_str(hex_str)?;
        // ANCHOR_END: string_to_asset_id
        assert_eq!([0u8; 32], *asset_id);
        // ANCHOR_END: asset_id
        Ok(())
    }
    #[tokio::test]
    async fn contract_id() -> Result<()> {
        // ANCHOR: contract_id
        use std::str::FromStr;

        use fuels::types::ContractId;

        // Zeroed Bytes32
        let contract_id = ContractId::zeroed();

        // Grab the inner `[u8; 32]` from
        // `Bytes32` by dereferencing (i.e. `*`) it.
        assert_eq!([0u8; 32], *contract_id);

        // From a `[u8; 32]`.
        // ANCHOR: array_to_contract_id
        let my_slice = [1u8; 32];
        let contract_id = ContractId::new(my_slice);
        // ANCHOR_END: array_to_contract_id
        assert_eq!([1u8; 32], *contract_id);

        // From a string.
        // ANCHOR: string_to_contract_id
        let hex_str = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let contract_id = ContractId::from_str(hex_str)?;
        // ANCHOR_END: string_to_contract_id
        assert_eq!([0u8; 32], *contract_id);
        // ANCHOR_END: contract_id

        // ANCHOR: contract_id_to_identity
        let _identity_from_contract_id = Identity::ContractId(contract_id);
        // ANCHOR_END: contract_id_to_identity

        // ANCHOR: contract_id_to_str
        let _str_from_contract_id: &str = contract_id.to_string().as_str();
        // ANCHOR_END: contract_id_to_str

        Ok(())
    }

    #[tokio::test]
    async fn type_conversion() -> Result<()> {
        // ANCHOR: type_conversion
        use fuels::types::{AssetId, ContractId};

        let contract_id = ContractId::new([1u8; 32]);

        let asset_id: AssetId = AssetId::new(*contract_id);

        assert_eq!([1u8; 32], *asset_id);
        // ANCHOR_END: type_conversion

        // ANCHOR: asset_id_to_str
        let _str_from_asset_id: &str = asset_id.to_string().as_str();
        // ANCHOR_END: asset_id_to_str

        // ANCHOR: contract_id_to_bits256
        let _contract_id_to_bits_256 = Bits256(contract_id.into());
        // ANCHOR_END: contract_id_to_bits256

        // ANCHOR: asset_id_to_bits256
        let _asset_id_to_bits_256 = Bits256(asset_id.into());
        // ANCHOR_END: asset_id_to_bits256

        Ok(())
    }

    #[tokio::test]
    async fn unused_generics() -> Result<()> {
        use fuels::prelude::*;
        abigen!(Contract(
            name = "MyContract",
            abi = r#" {
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
                  "typeId": "f097a9c7cabaa22d324d564b43210e927784b6f73609fe9e55900268b15910f5",
                  "type": "enum MyEnum",
                  "components": [
                    {
                      "name": "One",
                      "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                      "typeArguments": null
                    }
                  ],
                  "typeParameters": [
                    "8b8c08c464656c9a4b876c13199929c5ceb37ff6c927eaeefd756c12278e98c5",
                    "da229034ccc7938a0807cae31167ca4d3dc9f99e2eca4bfb4a2a740baf02a96f"

                  ]
                },
                {
                  "typeId": "da229034ccc7938a0807cae31167ca4d3dc9f99e2eca4bfb4a2a740baf02a96f",
                  "type": "generic K",
                  "components": null,
                  "typeParameters": null
                },
                {
                  "typeId": "8b8c08c464656c9a4b876c13199929c5ceb37ff6c927eaeefd756c12278e98c5",
                  "type": "generic T",
                  "components": null,
                  "typeParameters": null
                },
                {
                  "typeId": "c09f518d252533423934354a0974a7894bc99afbb03eb6f0956def50ae4146f0",
                  "type": "struct MyStruct",
                  "components": [
                    {
                      "name": "field",
                      "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
                      "typeArguments": null
                    }
                  ],
                  "typeParameters": [
                    "8b8c08c464656c9a4b876c13199929c5ceb37ff6c927eaeefd756c12278e98c5",
                    "da229034ccc7938a0807cae31167ca4d3dc9f99e2eca4bfb4a2a740baf02a96f"

                  ]
                },
                {
                  "typeId": "29881aad8730c5ab11d275376323d8e4ff4179aae8ccb6c13fe4902137e162ef",
                  "type": "u16",
                  "components": null,
                  "typeParameters": null
                },
                {
                  "typeId": "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc",
                  "type": "u32",
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
                      "name": "arg",
                      "type": "c09f518d252533423934354a0974a7894bc99afbb03eb6f0956def50ae4146f0",
                      "typeArguments": [
                        {
                          "name": "",
                          "type": "c89951a24c6ca28c13fd1cfdc646b2b656d69e61a92b91023be7eb58eb914b6b",
                          "typeArguments": null
                        },
                        {
                          "name": "",
                          "type": "29881aad8730c5ab11d275376323d8e4ff4179aae8ccb6c13fe4902137e162ef",
                          "typeArguments": null
                        }
                      ]
                    },
                    {
                      "name": "arg_2",
                      "type": "f097a9c7cabaa22d324d564b43210e927784b6f73609fe9e55900268b15910f5",
                      "typeArguments": [
                        {
                          "name": "",
                          "type": "d7649d428b9ff33d188ecbf38a7e4d8fd167fa01b2e10fe9a8f9308e52f1d7cc",
                          "typeArguments": null
                        },
                        {
                          "name": "",
                          "type": "1506e6f44c1d6291cdf46395a8e573276a4fa79e8ace3fc891e092ef32d1b0a0",
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

        // ANCHOR: unused_generics_struct
        assert_eq!(
            <MyStruct<u16, u32>>::new(15),
            MyStruct {
                field: 15,
                _unused_generic_0: std::marker::PhantomData,
                _unused_generic_1: std::marker::PhantomData
            }
        );
        // ANCHOR_END: unused_generics_struct

        let my_enum = <MyEnum<u32, u64>>::One(15);
        // ANCHOR: unused_generics_enum
        match my_enum {
            MyEnum::One(_value) => {}
            MyEnum::IgnoreMe(..) => panic!("Will never receive this variant"),
        }
        // ANCHOR_END: unused_generics_enum

        Ok(())
    }
}
