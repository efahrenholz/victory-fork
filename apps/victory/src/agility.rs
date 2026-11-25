//! D3D12 Agility SDK exports (`Microsoft.Direct3D.D3D12` 1.620.2).
//!
//! When `D3D12Core.dll` sits in `.\D3D12\` beside `victory.exe` and has a
//! newer SDK version than the in-box OS runtime, `d3d12.dll` loads it instead:
//! - On Windows 10, this unlocks SM 6.6 directly-indexed root signatures
//!   (`0xc00`).
//! - On Windows 11 21H2 (`10.0.22000.x`), this unlocks
//!   `ID3D12Device13::OpenExistingHeapFromAddress1` for in-place arenas.
//!
//! If `.\D3D12\` is absent, Windows falls back to the in-box runtime.
#![allow(non_upper_case_globals)]

#[repr(transparent)]
pub struct SyncPtr(pub *const u8);

// SAFETY: `D3D12SDKPath` points to a `'static` NUL-terminated byte literal.
unsafe impl Sync for SyncPtr {}

#[used]
#[unsafe(no_mangle)]
pub static D3D12SDKVersion: u32 = 620;

#[used]
#[unsafe(no_mangle)]
pub static D3D12SDKPath: SyncPtr = SyncPtr(c".\\D3D12\\".as_ptr().cast());
