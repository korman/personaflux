#![allow(clippy::missing_safety_doc)]

use std::slice;
use std::str;

use personaflux_core::{Simulation, SimulationConfig};

use crate::PfSimulation;
use crate::error::{
    PF_OK, PfResult, buffer_too_small, fail_batch, fail_core, ffi_result, invalid_argument,
    read_mut, read_ref, validate_array_stride, write_result,
};
use crate::types::{
    PF_ABI_VERSION, PF_MODEL_VERSION, PfDirectWitnessDeed, PfSubmissionResult, expected_size,
    parse_deed, submission_to_c, validate_output_size,
};

unsafe fn simulation_mut<'a>(pointer: *mut PfSimulation) -> Result<&'a mut PfSimulation, PfResult> {
    // SAFETY: The caller must pass a live handle created by pf_simulation_create.
    unsafe { read_mut(pointer) }
}

unsafe fn read_bytes<'a>(pointer: *const u8, length: u32) -> Result<&'a [u8], PfResult> {
    if length == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err(invalid_argument(
            "byte pointer is null for a non-empty input",
        ));
    }
    // SAFETY: The caller must provide `length` readable bytes for this call.
    Ok(unsafe { slice::from_raw_parts(pointer, length as usize) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_api_version(out_version: *mut u32) -> PfResult {
    ffi_result(|| unsafe { write_result(out_version, PF_ABI_VERSION) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_model_version(out_version: *mut u32) -> PfResult {
    ffi_result(|| unsafe { write_result(out_version, PF_MODEL_VERSION) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_create(
    random_seed: u64,
    out_simulation: *mut *mut PfSimulation,
) -> PfResult {
    ffi_result(|| {
        if out_simulation.is_null() {
            return invalid_argument("out_simulation is null");
        }
        let simulation = Box::new(PfSimulation {
            inner: Simulation::new(SimulationConfig { random_seed }),
        });
        // SAFETY: The output pointer was checked and points to caller-owned writable storage.
        unsafe { write_result(out_simulation, Box::into_raw(simulation)) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_destroy(simulation: *mut PfSimulation) {
    crate::error::clear_last_error();
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if simulation.is_null() {
            return;
        }
        // SAFETY: The handle must originate from pf_simulation_create and be destroyed once.
        unsafe { drop(Box::from_raw(simulation)) };
    }))
    .is_err()
    {
        crate::error::set_last_error("panic contained at the C ABI boundary");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_random_seed(
    simulation: *const PfSimulation,
    out_random_seed: *mut u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { read_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        unsafe { write_result(out_random_seed, simulation.inner.config().random_seed) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_faction_add(
    simulation: *mut PfSimulation,
    name: *const u8,
    name_len: u32,
    out_faction_id: *mut u64,
) -> PfResult {
    ffi_result(|| {
        if out_faction_id.is_null() {
            return invalid_argument("out_faction_id is null");
        }
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let bytes = match unsafe { read_bytes(name, name_len) } {
            Ok(bytes) => bytes,
            Err(error) => return error,
        };
        let name = match str::from_utf8(bytes) {
            Ok(name) => name,
            Err(_) => return invalid_argument("faction name is not valid UTF-8"),
        };
        let faction = match simulation.inner.add_faction(name) {
            Ok(faction) => faction,
            Err(error) => return fail_core(&error),
        };
        unsafe { write_result(out_faction_id, faction.into_raw()) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_member_add(
    simulation: *mut PfSimulation,
    faction_id: u64,
    out_member_id: *mut u64,
) -> PfResult {
    ffi_result(|| {
        if out_member_id.is_null() {
            return invalid_argument("out_member_id is null");
        }
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let member = match simulation
            .inner
            .add_member(crate::types::faction_id(faction_id))
        {
            Ok(member) => member,
            Err(error) => return fail_core(&error),
        };
        unsafe { write_result(out_member_id, member.into_raw()) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_member_to_member_set(
    simulation: *mut PfSimulation,
    observer: u64,
    target: u64,
    affinity: f32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let affinity = match crate::types::affinity(affinity) {
            Ok(affinity) => affinity,
            Err(error) => return error,
        };
        simulation
            .inner
            .set_member_relationship(
                crate::types::member_id(observer),
                crate::types::member_id(target),
                affinity,
            )
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_member_to_member_clear(
    simulation: *mut PfSimulation,
    observer: u64,
    target: u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        simulation
            .inner
            .clear_member_relationship(
                crate::types::member_id(observer),
                crate::types::member_id(target),
            )
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_faction_to_member_set(
    simulation: *mut PfSimulation,
    faction: u64,
    member: u64,
    affinity: f32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let affinity = match crate::types::affinity(affinity) {
            Ok(affinity) => affinity,
            Err(error) => return error,
        };
        simulation
            .inner
            .set_faction_member_relationship(
                crate::types::faction_id(faction),
                crate::types::member_id(member),
                affinity,
            )
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_faction_to_member_clear(
    simulation: *mut PfSimulation,
    faction: u64,
    member: u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        simulation
            .inner
            .clear_faction_member_relationship(
                crate::types::faction_id(faction),
                crate::types::member_id(member),
            )
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_faction_to_faction_set(
    simulation: *mut PfSimulation,
    source: u64,
    target: u64,
    affinity: f32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let affinity = match crate::types::affinity(affinity) {
            Ok(affinity) => affinity,
            Err(error) => return error,
        };
        simulation
            .inner
            .set_faction_relationship(
                crate::types::faction_id(source),
                crate::types::faction_id(target),
                affinity,
            )
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_relationship_faction_to_faction_clear(
    simulation: *mut PfSimulation,
    source: u64,
    target: u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        simulation
            .inner
            .clear_faction_relationship(
                crate::types::faction_id(source),
                crate::types::faction_id(target),
            )
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_step(
    simulation: *mut PfSimulation,
    delta_ticks: u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        simulation
            .inner
            .step(delta_ticks)
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_advance_to(
    simulation: *mut PfSimulation,
    target_tick: u64,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        simulation
            .inner
            .advance_to(target_tick)
            .map_or_else(|error| fail_core(&error), |_| PF_OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_submit_direct_witness(
    simulation: *mut PfSimulation,
    deed: *const PfDirectWitnessDeed,
    out_submission: *mut PfSubmissionResult,
    out_submission_size: u32,
) -> PfResult {
    ffi_result(|| {
        if let Err(error) =
            validate_output_size(out_submission_size, expected_size::<PfSubmissionResult>())
        {
            return error;
        }
        if out_submission.is_null() {
            return invalid_argument("out_submission is null");
        }
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let deed = match unsafe { read_ref(deed) } {
            Ok(deed) => match crate::types::parse_deed(deed) {
                Ok(deed) => deed,
                Err(error) => return error,
            },
            Err(error) => return error,
        };
        let submission = match simulation.inner.submit_direct_witness(deed) {
            Ok(submission) => submission,
            Err(error) => return fail_core(&error),
        };
        unsafe { write_result(out_submission, submission_to_c(submission)) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_simulation_submit_direct_witness_batch(
    simulation: *mut PfSimulation,
    deeds: *const PfDirectWitnessDeed,
    deed_count: u32,
    out_results: *mut PfSubmissionResult,
    result_capacity: u32,
    result_element_size: u32,
    out_result_count: *mut u32,
    out_error_index: *mut u32,
) -> PfResult {
    ffi_result(|| {
        if out_result_count.is_null() || out_error_index.is_null() {
            return invalid_argument("batch output counters are null");
        }
        // SAFETY: The output counters were checked and point to writable storage.
        unsafe {
            *out_result_count = 0;
            *out_error_index = u32::MAX;
        }
        if deed_count > result_capacity {
            // A null results pointer is valid for a size query in this branch.
            unsafe { write_result(out_result_count, deed_count) };
            return buffer_too_small("result buffer is smaller than deed_count");
        }
        if deed_count > 0 && out_results.is_null() {
            return invalid_argument("out_results is null for a non-empty batch");
        }
        if let Err(error) = validate_array_stride(
            deed_count,
            result_element_size,
            expected_size::<PfSubmissionResult>(),
            std::mem::align_of::<PfSubmissionResult>() as u32,
        ) {
            return error;
        }
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let raw_deeds = if deed_count == 0 {
            &[][..]
        } else {
            // SAFETY: Non-null input with the declared number of elements is required.
            if deeds.is_null() {
                return invalid_argument("deeds is null for a non-empty batch");
            }
            unsafe { slice::from_raw_parts(deeds, deed_count as usize) }
        };
        let mut parsed = Vec::with_capacity(raw_deeds.len());
        for (index, raw_deed) in raw_deeds.iter().enumerate() {
            match parse_deed(raw_deed) {
                Ok(deed) => parsed.push(deed),
                Err(error) => {
                    unsafe { write_result(out_error_index, index as u32) };
                    return error;
                }
            }
        }
        let submissions = match simulation.inner.submit_direct_witness_batch(&parsed) {
            Ok(submissions) => submissions,
            Err(error) => {
                unsafe { write_result(out_error_index, error.index() as u32) };
                return fail_batch(&error);
            }
        };
        for (index, submission) in submissions.into_iter().enumerate() {
            let output = submission_to_c(submission);
            // SAFETY: Capacity and element size were validated above. The v0 ABI uses a
            // contiguous array whose stride may include append-only future fields.
            let destination = (out_results as *mut u8)
                .wrapping_add(index * result_element_size as usize)
                .cast::<PfSubmissionResult>();
            unsafe { write_result(destination, output) };
        }
        unsafe { write_result(out_result_count, deed_count) }
    })
}
