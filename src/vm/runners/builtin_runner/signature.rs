use crate::{
    math_utils::safe_div_usize,
    types::{
        instance_definitions::ecdsa_instance_def::EcdsaInstanceDef,
        relocatable::{MaybeRelocatable, Relocatable},
    },
    vm::{
        errors::{memory_errors::MemoryError, runner_errors::RunnerError},
        vm_core::VirtualMachine,
        vm_memory::{
            memory::{Memory, ValidationRule},
            memory_segments::MemorySegmentManager,
        },
    },
};

use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use num_integer::{div_ceil, Integer};
use num_traits::ToPrimitive;
use std::{any::Any, collections::HashMap};

#[derive(Debug)]
pub struct SignatureBuiltinRunner {
    _name: String,
    included: bool,
    ratio: u32,
    base: isize,
    pub(crate) cells_per_instance: u32,
    pub(crate) n_input_cells: u32,
    _total_n_bits: u32,
    pub(crate) stop_ptr: Option<usize>,
    instances_per_component: u32,
    signatures: HashMap<Relocatable, Signature>,
}

impl SignatureBuiltinRunner {
    pub(crate) fn new(instance_def: &EcdsaInstanceDef, included: bool) -> Self {
        SignatureBuiltinRunner {
            base: 0,
            _name: "name".to_string(),
            included,
            ratio: instance_def.ratio,
            cells_per_instance: 5,
            n_input_cells: 2,
            _total_n_bits: 251,
            stop_ptr: None,
            instances_per_component: 1,
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
        if self.included {
            vec![MaybeRelocatable::from((self.base, 0))]
        } else {
            vec![]
        }
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

    pub fn ratio(&self) -> u32 {
        self.ratio
    }

    pub fn get_allocated_memory_units(&self, vm: &VirtualMachine) -> Result<usize, MemoryError> {
        let value = safe_div_usize(vm.current_step, self.ratio as usize)
            .map_err(|_| MemoryError::ErrorCalculatingMemoryUnits)?;
        Ok(self.cells_per_instance as usize * value)
    }

    pub fn get_memory_segment_addresses(&self) -> (&'static str, (isize, Option<usize>)) {
        ("ecdsa", (self.base, self.stop_ptr))
    }

    pub fn get_used_cells(&self, vm: &VirtualMachine) -> Result<usize, MemoryError> {
        let base = self.base();
        vm.segments
            .get_segment_used_size(
                base.try_into()
                    .map_err(|_| MemoryError::AddressInTemporarySegment(base))?,
            )
            .ok_or(MemoryError::MissingSegmentUsedSizes)
    }

    pub fn get_used_cells_and_allocated_size(
        &self,
        vm: &VirtualMachine,
    ) -> Result<(usize, usize), MemoryError> {
        let ratio = self.ratio as usize;
        let cells_per_instance = self.cells_per_instance;
        let min_step = ratio * self.instances_per_component as usize;
        if vm.current_step < min_step {
            Err(MemoryError::InsufficientAllocatedCells)
        } else {
            let used = self.get_used_cells(vm)?;
            let size = cells_per_instance as usize
                * safe_div_usize(vm.current_step, ratio)
                    .map_err(|_| MemoryError::InsufficientAllocatedCells)?;
            Ok((used, size))
        }
    }

    pub fn get_used_instances(&self, vm: &VirtualMachine) -> Result<usize, MemoryError> {
        let used_cells = self.get_used_cells(vm)?;
        Ok(div_ceil(used_cells, self.cells_per_instance as usize))
    }

    pub fn final_stack(
        &self,
        vm: &VirtualMachine,
        pointer: Relocatable,
    ) -> Result<(Relocatable, usize), RunnerError> {
        if self.included {
            if let Ok(stop_pointer) = vm
                .get_relocatable(&(pointer.sub(1)).map_err(|_| RunnerError::FinalStack)?)
                .as_deref()
            {
                if self.base() != stop_pointer.segment_index {
                    return Err(RunnerError::InvalidStopPointer("ecdsa".to_string()));
                }
                let stop_ptr = stop_pointer.offset;
                let num_instances = self
                    .get_used_instances(vm)
                    .map_err(|_| RunnerError::FinalStack)?;
                let used_cells = num_instances * self.cells_per_instance as usize;
                if stop_ptr != used_cells {
                    return Err(RunnerError::InvalidStopPointer("ecdsa".to_string()));
                }

                Ok((
                    pointer.sub(1).map_err(|_| RunnerError::FinalStack)?,
                    stop_ptr,
                ))
            } else {
                Err(RunnerError::FinalStack)
            }
        } else {
            let stop_ptr = self.base() as usize;
            Ok((pointer, stop_ptr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::vm;
    use crate::vm::{errors::memory_errors::MemoryError, vm_core::VirtualMachine};
    use crate::{
        types::instance_definitions::ecdsa_instance_def::EcdsaInstanceDef,
        vm::runners::builtin_runner::BuiltinRunner,
    };
    use num_bigint::BigInt;
    use num_bigint::Sign;

    #[test]
    fn initialize_segments_for_range_check() {
        let mut builtin = SignatureBuiltinRunner::new(&EcdsaInstanceDef::default(), true);
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        builtin.initialize_segments(&mut segments, &mut memory);
        assert_eq!(builtin.base, 0);
    }

    #[test]
    fn get_used_cells_missing_segment_used_sizes() {
        let builtin = BuiltinRunner::Signature(SignatureBuiltinRunner::new(
            &&EcdsaInstanceDef::default(),
            true,
        ));
        let vm = vm!();

        assert_eq!(
            builtin.get_used_cells(&vm),
            Err(MemoryError::MissingSegmentUsedSizes)
        );
    }

    #[test]
    fn get_used_cells_empty() {
        let builtin = BuiltinRunner::Signature(SignatureBuiltinRunner::new(
            &EcdsaInstanceDef::default(),
            true,
        ));
        let mut vm = vm!();

        vm.segments.segment_used_sizes = Some(vec![0]);
        assert_eq!(builtin.get_used_cells(&vm), Ok(0));
    }

    #[test]
    fn get_used_cells() {
        let builtin = BuiltinRunner::Signature(SignatureBuiltinRunner::new(
            &EcdsaInstanceDef::default(),
            true,
        ));
        let mut vm = vm!();

        vm.segments.segment_used_sizes = Some(vec![4]);
        assert_eq!(builtin.get_used_cells(&vm), Ok(4));
    }

    #[test]
    fn get_initial_stack_for_range_check_with_base() {
        let mut builtin = SignatureBuiltinRunner::new(&EcdsaInstanceDef::default(), true);
        builtin.base = 1;
        let initial_stack = builtin.initial_stack();
        assert_eq!(
            initial_stack[0].clone(),
            MaybeRelocatable::RelocatableValue((builtin.base(), 0).into())
        );
        assert_eq!(initial_stack.len(), 1);
    }

    #[test]
    fn initial_stack_not_included_test() {
        let ecdsa_builtin = SignatureBuiltinRunner::new(&EcdsaInstanceDef::default(), false);
        assert_eq!(ecdsa_builtin.initial_stack(), Vec::new())
    }
}
