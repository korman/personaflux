#![allow(clippy::missing_safety_doc)]

use crate::PfSimulation;
use crate::error::{
    PF_OK, PfResult, buffer_too_small, checked_u32, fail_core, ffi_result, invalid_argument,
    not_found, read_ref, validate_array_stride, write_result,
};
use crate::types::{
    PfMemberState, PfMemoryRecord, PfRelationshipLookup, expected_size, lookup_to_c, memory_to_c,
    pad_to_c, validate_output_size,
};

unsafe fn simulation_ref<'a>(pointer: *const PfSimulation) -> Result<&'a PfSimulation, PfResult> {
    // SAFETY: The caller must pass a live handle created by pf_simulation_create.
    unsafe { read_ref(pointer) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_current_tick(
    simulation: *const PfSimulation,
    out_tick: *mut u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        unsafe { write_result(out_tick, simulation.inner.current_tick()) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_faction_name_get(
    simulation: *const PfSimulation,
    faction_id: u64,
    out_name: *mut u8,
    name_capacity: u32,
    out_name_len: *mut u32,
) -> PfResult {
    ffi_result(|| {
        if out_name_len.is_null() {
            return invalid_argument("out_name_len is null");
        }
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let name = match simulation
            .inner
            .faction_name(crate::types::faction_id(faction_id))
        {
            Some(name) => name.as_bytes(),
            None => return not_found("faction was not found"),
        };
        let required = match checked_u32(name.len()) {
            Ok(required) => required,
            Err(error) => return error,
        };
        unsafe { write_result(out_name_len, required) };
        if name_capacity < required {
            return buffer_too_small("faction name buffer is too small");
        }
        if required > 0 && out_name.is_null() {
            return invalid_argument("out_name is null for a non-empty name");
        }
        if required > 0 {
            // SAFETY: The caller supplied a buffer at least as large as `required`.
            unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), out_name, required as usize) };
        }
        PF_OK
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_member_state_get(
    simulation: *const PfSimulation,
    member_id: u64,
    out_state: *mut PfMemberState,
    out_state_size: u32,
) -> PfResult {
    ffi_result(|| {
        if let Err(error) = validate_output_size(out_state_size, expected_size::<PfMemberState>()) {
            return error;
        }
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let member_id = crate::types::member_id(member_id);
        let faction_id = match simulation.inner.member_faction(member_id) {
            Some(faction) => faction,
            None => return not_found("member was not found"),
        };
        let pad = match simulation.inner.member_pad(member_id) {
            Some(pad) => pad,
            None => return not_found("member was not found"),
        };
        let state = PfMemberState {
            struct_size: expected_size::<PfMemberState>(),
            api_version: crate::types::PF_ABI_VERSION,
            faction_id: faction_id.into_raw(),
            pad: pad_to_c(pad),
        };
        unsafe { write_result(out_state, state) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_member_affinity_get(
    simulation: *const PfSimulation,
    observer: u64,
    actor: u64,
    out_affinity: *mut f32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let affinity = match simulation.inner.member_affinity(
            crate::types::member_id(observer),
            crate::types::member_id(actor),
        ) {
            Ok(affinity) => affinity,
            Err(error) => return fail_core(&error),
        };
        unsafe { write_result(out_affinity, affinity.value()) }
    })
}

fn write_relationship(
    output: *mut PfRelationshipLookup,
    output_size: u32,
    lookup: personaflux_core::RelationshipLookup,
) -> PfResult {
    if let Err(error) = validate_output_size(output_size, expected_size::<PfRelationshipLookup>()) {
        return error;
    }
    // SAFETY: The caller supplies a writable output buffer of the validated size.
    unsafe { write_result(output, lookup_to_c(lookup)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_member_to_member_get(
    simulation: *const PfSimulation,
    observer: u64,
    target: u64,
    out_relationship: *mut PfRelationshipLookup,
    out_relationship_size: u32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let lookup = match simulation.inner.member_relationship(
            crate::types::member_id(observer),
            crate::types::member_id(target),
        ) {
            Ok(lookup) => lookup,
            Err(error) => return fail_core(&error),
        };
        write_relationship(out_relationship, out_relationship_size, lookup)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_faction_to_member_get(
    simulation: *const PfSimulation,
    faction: u64,
    member: u64,
    out_relationship: *mut PfRelationshipLookup,
    out_relationship_size: u32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let lookup = match simulation.inner.faction_member_relationship(
            crate::types::faction_id(faction),
            crate::types::member_id(member),
        ) {
            Ok(lookup) => lookup,
            Err(error) => return fail_core(&error),
        };
        write_relationship(out_relationship, out_relationship_size, lookup)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_faction_to_faction_get(
    simulation: *const PfSimulation,
    source: u64,
    target: u64,
    out_relationship: *mut PfRelationshipLookup,
    out_relationship_size: u32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let lookup = match simulation.inner.faction_relationship(
            crate::types::faction_id(source),
            crate::types::faction_id(target),
        ) {
            Ok(lookup) => lookup,
            Err(error) => return fail_core(&error),
        };
        write_relationship(out_relationship, out_relationship_size, lookup)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_effective_member_get(
    simulation: *const PfSimulation,
    observer: u64,
    target: u64,
    out_relationship: *mut PfRelationshipLookup,
    out_relationship_size: u32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let lookup = match simulation.inner.effective_member_relationship(
            crate::types::member_id(observer),
            crate::types::member_id(target),
        ) {
            Ok(lookup) => lookup,
            Err(error) => return fail_core(&error),
        };
        write_relationship(out_relationship, out_relationship_size, lookup)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_memory_get(
    simulation: *const PfSimulation,
    observer: u64,
    deed_id: u64,
    out_memory: *mut PfMemoryRecord,
    out_memory_size: u32,
    out_present: *mut u8,
) -> PfResult {
    ffi_result(|| {
        if let Err(error) = validate_output_size(out_memory_size, expected_size::<PfMemoryRecord>())
        {
            return error;
        }
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let record = match simulation
            .inner
            .memory(crate::types::member_id(observer), deed_id)
        {
            Ok(record) => record,
            Err(error) => return fail_core(&error),
        };
        let present = u8::from(record.is_some());
        unsafe { write_result(out_present, present) };
        let output = record.map(memory_to_c).unwrap_or(PfMemoryRecord {
            struct_size: expected_size::<PfMemoryRecord>(),
            api_version: crate::types::PF_ABI_VERSION,
            ..PfMemoryRecord::default()
        });
        unsafe { write_result(out_memory, output) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_memories_count(
    simulation: *const PfSimulation,
    observer: u64,
    out_count: *mut u32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let records = match simulation
            .inner
            .memories_for(crate::types::member_id(observer))
        {
            Ok(records) => records,
            Err(error) => return fail_core(&error),
        };
        let count = match checked_u32(records.len()) {
            Ok(count) => count,
            Err(error) => return error,
        };
        unsafe { write_result(out_count, count) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_memories_read(
    simulation: *const PfSimulation,
    observer: u64,
    out_records: *mut PfMemoryRecord,
    record_capacity: u32,
    record_element_size: u32,
    out_count: *mut u32,
) -> PfResult {
    ffi_result(|| {
        if out_count.is_null() {
            return invalid_argument("out_count is null");
        }
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let records = match simulation
            .inner
            .memories_for(crate::types::member_id(observer))
        {
            Ok(records) => records,
            Err(error) => return fail_core(&error),
        };
        let required = match checked_u32(records.len()) {
            Ok(required) => required,
            Err(error) => return error,
        };
        unsafe { write_result(out_count, required) };
        if record_capacity < required {
            return buffer_too_small("memory record buffer is too small");
        }
        if required > 0 && out_records.is_null() {
            return invalid_argument("out_records is null for non-empty results");
        }
        if let Err(error) = validate_array_stride(
            required,
            record_element_size,
            expected_size::<PfMemoryRecord>(),
            std::mem::align_of::<PfMemoryRecord>() as u32,
        ) {
            return error;
        }
        for (index, record) in records.into_iter().enumerate() {
            // SAFETY: Capacity and element size were validated above.
            let destination = (out_records as *mut u8)
                .wrapping_add(index * record_element_size as usize)
                .cast::<PfMemoryRecord>();
            unsafe { write_result(destination, memory_to_c(record)) };
        }
        PF_OK
    })
}
