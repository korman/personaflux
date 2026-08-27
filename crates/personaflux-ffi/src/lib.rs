use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

use personaflux_core::{Simulation, SimulationConfig};

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PfResult {
    Ok = 0,
    InvalidArgument = 1,
    InternalError = 255,
}

#[repr(C)]
pub struct PfSimulation {
    inner: Simulation,
}

#[unsafe(no_mangle)]
/// Creates a simulation and writes its opaque handle to `out_simulation`.
///
/// # Safety
///
/// `out_simulation` must be null or point to writable storage for one pointer.
pub unsafe extern "C" fn pf_simulation_create(
    random_seed: u64,
    out_simulation: *mut *mut PfSimulation,
) -> PfResult {
    ffi_result(|| {
        if out_simulation.is_null() {
            return PfResult::InvalidArgument;
        }

        let simulation = Box::new(PfSimulation {
            inner: Simulation::new(SimulationConfig { random_seed }),
        });

        // SAFETY: The pointer is non-null and points to caller-owned writable storage.
        unsafe { ptr::write(out_simulation, Box::into_raw(simulation)) };
        PfResult::Ok
    })
}

#[unsafe(no_mangle)]
/// Destroys a simulation created by [`pf_simulation_create`].
///
/// # Safety
///
/// `simulation` must be null or a live handle returned by
/// [`pf_simulation_create`] that has not already been destroyed.
pub unsafe extern "C" fn pf_simulation_destroy(simulation: *mut PfSimulation) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if simulation.is_null() {
            return;
        }

        // SAFETY: This pointer must come from pf_simulation_create and be destroyed once.
        unsafe { drop(Box::from_raw(simulation)) };
    }));
}

#[unsafe(no_mangle)]
/// Writes the simulation's configured random seed.
///
/// # Safety
///
/// `simulation` must be a live handle returned by [`pf_simulation_create`], and
/// `out_random_seed` must point to writable storage for one `u64`.
pub unsafe extern "C" fn pf_simulation_random_seed(
    simulation: *const PfSimulation,
    out_random_seed: *mut u64,
) -> PfResult {
    ffi_result(|| {
        if simulation.is_null() || out_random_seed.is_null() {
            return PfResult::InvalidArgument;
        }

        // SAFETY: Both pointers were checked for null and are only used for this call.
        let random_seed = unsafe { (*simulation).inner.config().random_seed };
        unsafe { ptr::write(out_random_seed, random_seed) };
        PfResult::Ok
    })
}

fn ffi_result(operation: impl FnOnce() -> PfResult) -> PfResult {
    catch_unwind(AssertUnwindSafe(operation)).unwrap_or(PfResult::InternalError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_reads_and_destroys_a_simulation() {
        let mut simulation = ptr::null_mut();
        // SAFETY: The output and result pointers remain valid for each call.
        unsafe {
            assert_eq!(pf_simulation_create(42, &mut simulation), PfResult::Ok);

            let mut random_seed = 0;
            assert_eq!(
                pf_simulation_random_seed(simulation, &mut random_seed),
                PfResult::Ok
            );
            assert_eq!(random_seed, 42);

            pf_simulation_destroy(simulation);
        }
    }

    #[test]
    fn rejects_null_output_pointers() {
        // SAFETY: A null output pointer is explicitly accepted and rejected.
        unsafe {
            assert_eq!(
                pf_simulation_create(0, ptr::null_mut()),
                PfResult::InvalidArgument
            );
        }
    }
}
