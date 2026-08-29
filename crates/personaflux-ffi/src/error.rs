use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};

use personaflux_core::{BatchError, Error};

pub type PfResult = i32;

pub const PF_OK: PfResult = 0;
pub const PF_INVALID_ARGUMENT: PfResult = 1;
pub const PF_NOT_FOUND: PfResult = 2;
pub const PF_INVALID_STATE: PfResult = 3;
pub const PF_BUFFER_TOO_SMALL: PfResult = 4;
pub const PF_SERIALIZATION_ERROR: PfResult = 5;
pub const PF_VERSION_MISMATCH: PfResult = 6;
pub const PF_INTERNAL_ERROR: PfResult = 255;

thread_local! {
    static LAST_ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

pub fn clear_last_error() {
    LAST_ERROR.with(|message| message.borrow_mut().clear());
}

pub fn set_last_error(message: impl Into<String>) {
    let bytes = message.into().into_bytes();
    LAST_ERROR.with(|current| *current.borrow_mut() = bytes);
}

pub fn last_error_bytes() -> Vec<u8> {
    LAST_ERROR.with(|message| message.borrow().clone())
}

pub fn invalid_argument(message: impl Into<String>) -> PfResult {
    set_last_error(message);
    PF_INVALID_ARGUMENT
}

pub fn not_found(message: impl Into<String>) -> PfResult {
    set_last_error(message);
    PF_NOT_FOUND
}

pub fn buffer_too_small(message: impl Into<String>) -> PfResult {
    set_last_error(message);
    PF_BUFFER_TOO_SMALL
}

pub fn version_mismatch() -> PfResult {
    set_last_error("unsupported ABI version");
    PF_VERSION_MISMATCH
}

pub fn map_core_error(error: &Error) -> PfResult {
    match error {
        Error::EmptyName | Error::ActorTargetSame(_) | Error::Value(_) => PF_INVALID_ARGUMENT,
        Error::FactionNotFound(_) | Error::MemberNotFound(_) => PF_NOT_FOUND,
        Error::TimeWentBackwards { .. } | Error::TimeOverflow => PF_INVALID_STATE,
    }
}

pub fn fail_core(error: &Error) -> PfResult {
    let result = map_core_error(error);
    set_last_error(format!("{error:?}"));
    result
}

pub fn fail_batch(error: &BatchError) -> PfResult {
    let result = map_core_error(error.error());
    set_last_error(format!(
        "batch index {}: {:?}",
        error.index(),
        error.error()
    ));
    result
}

pub fn ffi_result(operation: impl FnOnce() -> PfResult) -> PfResult {
    clear_last_error();
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(result) => {
            if result != PF_OK && last_error_bytes().is_empty() {
                set_last_error("operation failed");
            }
            result
        }
        Err(_) => {
            set_last_error("panic contained at the C ABI boundary");
            PF_INTERNAL_ERROR
        }
    }
}

pub fn checked_u32(value: usize) -> Result<u32, PfResult> {
    u32::try_from(value).map_err(|_| {
        set_last_error("result count exceeds the C ABI uint32 range");
        PF_INTERNAL_ERROR
    })
}

pub fn validate_array_stride(
    count: u32,
    element_size: u32,
    expected_size: u32,
    alignment: u32,
) -> Result<(), PfResult> {
    if count == 0 {
        return Ok(());
    }
    if element_size < expected_size {
        return Err(invalid_argument(
            "array element size is smaller than the v0 type",
        ));
    }
    if alignment == 0 || element_size % alignment != 0 {
        return Err(invalid_argument(
            "array element size is not correctly aligned",
        ));
    }
    (count as usize)
        .checked_mul(element_size as usize)
        .ok_or_else(|| invalid_argument("array byte size overflows the host address space"))?;
    Ok(())
}

pub unsafe fn write_value<T: Copy>(destination: *mut T, value: T) -> Result<(), PfResult> {
    if destination.is_null() {
        return Err(invalid_argument("output pointer is null"));
    }
    if (destination as usize) % std::mem::align_of::<T>() != 0 {
        return Err(invalid_argument("output pointer is not correctly aligned"));
    }
    // SAFETY: The caller contract requires destination to point to writable storage.
    unsafe { std::ptr::write(destination, value) };
    Ok(())
}

pub unsafe fn write_result<T: Copy>(destination: *mut T, value: T) -> PfResult {
    // SAFETY: The caller passes the same pointer governed by `write_value`'s contract.
    unsafe { write_value(destination, value) }.map_or_else(|error| error, |_| PF_OK)
}

pub unsafe fn read_ref<'a, T>(source: *const T) -> Result<&'a T, PfResult> {
    if source.is_null() {
        return Err(invalid_argument("input pointer is null"));
    }
    if (source as usize) % std::mem::align_of::<T>() != 0 {
        return Err(invalid_argument("input pointer is not correctly aligned"));
    }
    // SAFETY: The caller contract requires source to point to a live value for this call.
    // The null case was handled above.
    Ok(unsafe { &*source })
}

pub unsafe fn read_mut<'a, T>(source: *mut T) -> Result<&'a mut T, PfResult> {
    if source.is_null() {
        return Err(invalid_argument("input pointer is null"));
    }
    if (source as usize) % std::mem::align_of::<T>() != 0 {
        return Err(invalid_argument("input pointer is not correctly aligned"));
    }
    // SAFETY: The caller contract requires source to point to a live, uniquely borrowed value.
    Ok(unsafe { &mut *source })
}
