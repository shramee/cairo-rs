use crate::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::{
        errors::{memory_errors::MemoryError, runner_errors::RunnerError},
        vm_memory::{
            memory::{Memory, ValidationRule},
            memory_segments::MemorySegmentManager,
        },
    },
};

use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use num_integer::Integer;
use num_traits::ToPrimitive;
use std::{any::Any, collections::HashMap};

#[derive(Debug)]
pub struct SignatureBuiltinRunner {
    _name: String,
    _included: bool,
    _ratio: usize,
    base: isize,
    pub(crate) cells_per_instance: u32,
    pub(crate) n_input_cells: u32,
    _total_n_bits: u32,
    signatures: HashMap<Relocatable, Signature>,
}

impl SignatureBuiltinRunner {
    pub fn new(ratio: usize) -> Self {
        SignatureBuiltinRunner {
            base: 0,
            _name: "name".to_string(),
            _included: false,
            _ratio: ratio,
            cells_per_instance: 5,
            n_input_cells: 2,
            _total_n_bits: 251,
            signatures: HashMap::new(),
        }
    }

    pub fn add_signature(&mut self, relocatable: Relocatable, signature: Signature) {
        self.signatures.entry(relocatable).or_insert(signature);
    }
}

impl SignatureBuiltinRunner {
    pub fn initialize_segments(
        &mut self,
        segments: &mut MemorySegmentManager,
        memory: &mut Memory,
    ) {
        self.base = segments.add(memory).segment_index
    }

    pub fn initial_stack(&self) -> Vec<MaybeRelocatable> {
        vec![MaybeRelocatable::from((self.base, 0))]
    }

    pub fn base(&self) -> isize {
        self.base
    }
    pub fn add_validation_rule(&self, memory: &mut Memory) -> Result<(), RunnerError> {
        let cells_per_instance = self.cells_per_instance;
        let signatures = self.signatures.clone();
        let rule: ValidationRule = ValidationRule(Box::new(
            move |memory: &Memory,
                  address: &MaybeRelocatable|
                  -> Result<Vec<MaybeRelocatable>, MemoryError> {
                let address = match address {
                    MaybeRelocatable::RelocatableValue(address) => address,
                    _ => return Err(MemoryError::AddressNotRelocatable),
                };

                let address_offset = address.offset.mod_floor(&(cells_per_instance as usize));
                let mem_addr_sum = memory.get(&(address + 1_i32));
                let mem_addr_less = if address.offset > 0 {
                    memory.get(
                        &address
                            .sub(1)
                            .map_err(|_| MemoryError::AddressNotRelocatable)?,
                    )
                } else {
                    Ok(None)
                };
                let (pubkey_addr, msg_addr) = match (address_offset, mem_addr_sum, mem_addr_less) {
                    (0, Ok(Some(_element)), _) => {
                        let pubkey_addr = address.clone();
                        let msg_addr = address + 1_i32;
                        (pubkey_addr, msg_addr)
                    }
                    (1, _, Ok(Some(_element))) if address.offset > 0 => {
                        let pubkey_addr = address
                            .sub(1)
                            .map_err(|_| MemoryError::AddressNotRelocatable)?;
                        let msg_addr = address.clone();
                        (pubkey_addr, msg_addr)
                    }
                    _ => return Ok(Vec::new()),
                };

                let (_sign, msg) = memory
                    .get_integer(&msg_addr)
                    .map_err(|_| MemoryError::AddressNotRelocatable)?
                    .to_bytes_be();
                let (_sign, pubkey) = memory
                    .get_integer(&pubkey_addr)
                    .map_err(|_| MemoryError::AddressNotRelocatable)?
                    .to_bytes_be();

                let verify_key = VerifyingKey::from_sec1_bytes(&pubkey)
                    .map_err(|_| MemoryError::AddressNotRelocatable)?;

                let signature = signatures
                    .get(&pubkey_addr)
                    .ok_or(MemoryError::AddressNotRelocatable)?;

                verify_key
                    .verify(&msg, signature)
                    .map_err(|_| MemoryError::AddressNotRelocatable)?;
                Ok(Vec::new())
            },
        ));
        memory.add_validation_rule(
            self.base
                .to_usize()
                .ok_or(RunnerError::RunnerInTemporarySegment(self.base))?,
            rule,
        );
        Ok(())
    }

    pub fn deduce_memory_cell(
        &mut self,
        _address: &Relocatable,
        _memory: &Memory,
    ) -> Result<Option<MaybeRelocatable>, RunnerError> {
        Ok(None)
    }

    pub fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_segments_for_range_check() {
        let mut builtin = SignatureBuiltinRunner::new(10);
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        builtin.initialize_segments(&mut segments, &mut memory);
        assert_eq!(builtin.base, 0);
    }

    #[test]
    fn get_initial_stack_for_range_check_with_base() {
        let mut builtin = SignatureBuiltinRunner::new(10);
        builtin.base = 1;
        let initial_stack = builtin.initial_stack();
        assert_eq!(
            initial_stack[0].clone(),
            MaybeRelocatable::RelocatableValue((builtin.base(), 0).into())
        );
        assert_eq!(initial_stack.len(), 1);
    }
}
