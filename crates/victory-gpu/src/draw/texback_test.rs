use super::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R32_FLOAT, DXGI_SAMPLE_DESC};
use windows::core::IUnknown;

fn texture(slices: u16) -> IUnknown {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Width: 8,
        Height: 8,
        DepthOrArraySize: slices,
        MipLevels: 3,
        Format: DXGI_FORMAT_R32_FLOAT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        ..Default::default()
    };
    let mut texture = std::ptr::null();
    assert_eq!(
        crate::resource::create_committed(
            std::ptr::null(),
            (&raw const heap).cast(),
            0,
            (&raw const desc).cast(),
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS.0 as u32,
            std::ptr::null(),
            std::ptr::from_ref(&ID3D12Resource::IID).cast(),
            &mut texture,
        ),
        crate::object::S_OK
    );
    // SAFETY: create_committed handed us an owned COM reference.
    unsafe { IUnknown::from_raw(texture.cast_mut().cast()) }
}

fn array_view(mip: u32, first: u32, count: u32) -> [u32; 10] {
    let dimension = D3D12_UAV_DIMENSION_TEXTURE2DARRAY.0 as u32;
    [41, dimension, mip, first, count, 0, 0, 0, 0, 0]
}

fn two_d_view(mip: u32) -> [u32; 10] {
    let dimension = D3D12_UAV_DIMENSION_TEXTURE2D.0 as u32;
    [41, dimension, mip, 0, 0, 0, 0, 0, 0, 0]
}

#[test]
fn water_readback_preserves_texels_outside_bound_uavs() {
    const CHILD: &str = "VICTORY_TEST_TEXBACK_UAVS";
    if std::env::var_os(CHILD).is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                std::thread::current().name().unwrap(),
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("VICTORY_D3D12_WARP", "1")
            .env("VICTORY_TEXTURE_BACK", "8x8f41")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    let Some(host) = crate::host::host() else {
        eprintln!("texture readback test skipped: no D3D12 WARP device");
        return;
    };

    verify_readback(host, 4);
    verify_readback(host, 1);
}

fn verify_readback(host: &crate::host::Host, slices: u16) {
    let owner = texture(slices);
    let title = owner.as_raw().cast();
    let texture = crate::resource::host_copy(title).unwrap();
    let (layout, at, size) = crate::resource::title_texture(title).unwrap();
    let initialize = (0..3)
        .map(|mip| {
            (
                Some(if slices == 1 {
                    two_d_view(mip)
                } else {
                    array_view(mip, 0, u32::from(slices))
                }),
                1.0,
            )
        })
        .collect::<Vec<_>>();
    let bound = if slices == 1 {
        vec![(None, 4.0), (Some(two_d_view(1)), 5.0)]
    } else {
        vec![
            (None, 4.0),
            (Some(array_view(1, 1, 2)), 5.0),
            (Some(array_view(1, 2, u32::MAX)), 5.0),
            (Some(two_d_view(2)), 6.0),
        ]
    };
    // The first list sets every host texel to 1, the second binds the default
    // view and two overlapping array UAVs through the shader binding path.
    for (pass, views) in [initialize, bound].into_iter().enumerate() {
        if pass == 1 {
            // SAFETY: the owned title allocation has size bytes, with no GPU work pending.
            unsafe { (at as *mut u8).write_bytes(0x4a, size) };
        }
        let list = host.new_list().unwrap();
        let mut heaps = Vec::new();
        for (words, value) in &views {
            if pass == 1 {
                readable(
                    &list,
                    ResourceType::Uav,
                    &View::Unordered(title as usize, 0, *words),
                );
            }
            // SAFETY: the views describe this live UAV texture. Keep both
            // descriptor heaps alive through completion of the list.
            unsafe {
                let heap = |flags| -> ID3D12DescriptorHeap {
                    host.device
                        .CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                            NumDescriptors: 1,
                            Flags: flags,
                            NodeMask: 0,
                        })
                        .unwrap()
                };
                let cpu = heap(D3D12_DESCRIPTOR_HEAP_FLAG_NONE);
                let gpu = heap(D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE);
                let native = words.as_ref().map(|w| {
                    w.as_ptr()
                        .cast::<D3D12_UNORDERED_ACCESS_VIEW_DESC>()
                        .read_unaligned()
                });
                let view = native.as_ref().map(std::ptr::from_ref);
                let handle = cpu.GetCPUDescriptorHandleForHeapStart();
                host.device.CreateUnorderedAccessView(
                    &texture.texture,
                    None::<&ID3D12Resource>,
                    view,
                    handle,
                );
                host.device.CopyDescriptorsSimple(
                    1,
                    gpu.GetCPUDescriptorHandleForHeapStart(),
                    handle,
                    D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                );
                list.list.SetDescriptorHeaps(&[Some(gpu.clone())]);
                list.list.ClearUnorderedAccessViewFloat(
                    gpu.GetGPUDescriptorHandleForHeapStart(),
                    handle,
                    &texture.texture,
                    &[*value; 4],
                    &[],
                );
                heaps.push((cpu, gpu));
            }
        }
        list.close();
        host.execute(&[&list]);
    }
    let mut expected_bytes = vec![0x4a; size];
    for mip in 0..3 {
        for slice in 0..u32::from(slices) {
            let expected = match (mip, slice) {
                (0, _) => 4f32.to_bits(),
                (1, _) if slices == 1 || slice >= 1 => 5f32.to_bits(),
                (2, 0) if slices > 1 => 6f32.to_bits(),
                _ => 0x4a4a4a4a,
            };
            for y in 0..(8 >> mip) {
                for x in 0..(8 >> mip) {
                    let offset = layout.offset(mip, x, y, slice).unwrap() as usize;
                    expected_bytes[offset..offset + 4].copy_from_slice(&expected.to_le_bytes());
                }
            }
        }
    }
    // SAFETY: the owned allocation has size bytes and execute waited for its readback.
    let actual = unsafe { std::slice::from_raw_parts(at as *const u8, size) };
    if let Some(offset) = actual.iter().zip(&expected_bytes).position(|(a, b)| a != b) {
        panic!(
            "texture with {slices} slices: byte {offset} was {}, expected {}",
            actual[offset], expected_bytes[offset]
        );
    }
}
