contract;

mod contract_a_types;
mod another_lib;

use another_lib::VeryCommonNameStruct;

abi MyContract {
    fn test_function(arg: contract_a_types::VeryCommonNameStruct) -> VeryCommonNameStruct;
}

impl MyContract for Contract {
    fn test_function(arg: contract_a_types::VeryCommonNameStruct) -> VeryCommonNameStruct {
        VeryCommonNameStruct { field: arg.another_field }
    }
}
