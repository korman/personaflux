#![allow(clippy::missing_safety_doc)]

use crate::PfSimulation;
use crate::error::{
    PF_OK, PfResult, buffer_too_small, checked_u32, ffi_result, invalid_argument, read_mut,
    read_ref, validate_array_stride, write_result,
};
use crate::types::{PfEvent, event_to_c, expected_size};

unsafe fn simulation_ref<'a>(pointer: *const PfSimulation) -> Result<&'a PfSimulation, PfResult> {
    // SAFETY: The caller must pass a live handle created by pf_simulation_create.
    unsafe { read_ref(pointer) }
}

unsafe fn simulation_mut<'a>(pointer: *mut PfSimulation) -> Result<&'a mut PfSimulation, PfResult> {
    // SAFETY: The caller must pass a live handle created by pf_simulation_create.
    unsafe { read_mut(pointer) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_events_count(
    simulation: *const PfSimulation,
    out_count: *mut u32,
) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_ref(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        let count = match checked_u32(simulation.inner.event_count()) {
            Ok(count) => count,
            Err(error) => return error,
        };
        unsafe { write_result(out_count, count) }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_events_read(
    simulation: *const PfSimulation,
    out_events: *mut PfEvent,
    event_capacity: u32,
    event_element_size: u32,
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
        let events = simulation.inner.events_snapshot();
        let required = match checked_u32(events.len()) {
            Ok(required) => required,
            Err(error) => return error,
        };
        unsafe { write_result(out_count, required) };
        if event_capacity < required {
            return buffer_too_small("event buffer is too small");
        }
        if required > 0 && out_events.is_null() {
            return invalid_argument("out_events is null for non-empty results");
        }
        if let Err(error) = validate_array_stride(
            required,
            event_element_size,
            expected_size::<PfEvent>(),
            std::mem::align_of::<PfEvent>() as u32,
        ) {
            return error;
        }
        for (index, event) in events.into_iter().enumerate() {
            // SAFETY: Capacity and element size were validated above.
            let destination = (out_events as *mut u8)
                .wrapping_add(index * event_element_size as usize)
                .cast::<PfEvent>();
            unsafe { write_result(destination, event_to_c(event)) };
        }
        PF_OK
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_events_clear(simulation: *mut PfSimulation) -> PfResult {
    ffi_result(|| {
        let simulation = match unsafe { simulation_mut(simulation) } {
            Ok(simulation) => simulation,
            Err(error) => return error,
        };
        simulation.inner.drain_events().for_each(drop);
        PF_OK
    })
}

/// Copies the thread-local diagnostic message without clearing it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_last_error_message_copy(
    out_message: *mut u8,
    message_capacity: u32,
    out_message_len: *mut u32,
) -> PfResult {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if out_message_len.is_null() {
            return invalid_argument("out_message_len is null");
        }
        let message = crate::error::last_error_bytes();
        let required = match checked_u32(message.len()) {
            Ok(required) => required,
            Err(error) => return error,
        };
        // SAFETY: The output counter was checked and points to writable storage.
        unsafe { *out_message_len = required };
        if message_capacity < required {
            // Do not replace the diagnostic being queried with a diagnostic about
            // the diagnostic buffer itself.
            return crate::error::PF_BUFFER_TOO_SMALL;
        }
        if required > 0 && out_message.is_null() {
            return crate::error::PF_INVALID_ARGUMENT;
        }
        if required > 0 {
            // SAFETY: The caller supplied a buffer at least as large as `required`.
            unsafe {
                std::ptr::copy_nonoverlapping(message.as_ptr(), out_message, required as usize)
            };
        }
        PF_OK
    }));
    match result {
        Ok(code) => code,
        Err(_) => {
            crate::error::set_last_error("panic contained at the C ABI boundary");
            crate::error::PF_INTERNAL_ERROR
        }
    }
}
