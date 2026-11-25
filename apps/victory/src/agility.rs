//! D3D12 Agility SDK exports (`Microsoft.Direct3D.D3D12` 1.620.2).
//!
//! Exported via `build.rs` (`agility.def` on GNU, `/EXPORT` on MSVC) so
//! `d3d12.dll` loads `.\D3D12\D3D12Core.dll` beside the executable, unlocking
//! `ID3D12Device13::OpenExistingHeapFromAddress1` on Win11 21H2/Win10.
#![allow(non_upper_case_globals)]

#[repr(transparent)]
pub struct SyncPtr(pub *const u8);
// SAFETY: points to a `'static` NUL-terminated byte literal.
unsafe impl Sync for SyncPtr {}

#[used]
#[unsafe(no_mangle)]
pub static D3D12SDKVersion: u32 = 620;

#[used]
#[unsafe(no_mangle)]
pub static D3D12SDKPath: SyncPtr = SyncPtr(c".\\D3D12\\".as_ptr().cast());
