// use ztron::precompiles;
use primitive_types::H160;
// use precompiled::tron_precompile;

use crate::{ExitError, ExitSucceed};

pub fn tron_precompile(
    address: H160,
    input: &[u8],
    _target_gas: Option<usize>,
) -> Option<Result<(ExitSucceed, Vec<u8>, usize), ExitError>> {
    match address.to_low_u64_be() {
        0x1 => precompiled::tron_precompile(address, input, None),
        addr if addr <= 0x100000f => {
            log::debug!("precompile of address: {:x}", addr);
            unimplemented!()
        }
        _ => None
    }
}
