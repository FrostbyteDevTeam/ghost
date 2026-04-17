use crate::error::CoreError;

/// Capture the primary monitor as PNG bytes.
pub fn capture_screen() -> Result<Vec<u8>, CoreError> {
    unsafe {
        use windows::Win32::Graphics::Dxgi::*;
        use windows::Win32::Graphics::Direct3D11::*;
        use windows::Win32::Graphics::Direct3D::*;
        use windows::core::Interface;

        // 1. Create D3D11 device
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
        .map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "D3D11CreateDevice",
        })?;

        let device = device.ok_or(CoreError::Win32 {
            code: 0,
            context: "D3D11 device null",
        })?;
        let context = context.ok_or(CoreError::Win32 {
            code: 0,
            context: "D3D11 context null",
        })?;

        // 2. Get DXGI device/adapter/output
        let dxgi_device: IDXGIDevice = device.cast().map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "IDXGIDevice cast",
        })?;
        let adapter = dxgi_device.GetAdapter().map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "GetAdapter",
        })?;
        let output: IDXGIOutput = adapter.EnumOutputs(0).map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "EnumOutputs",
        })?;
        let output1: IDXGIOutput1 = output.cast().map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "IDXGIOutput1 cast",
        })?;

        // 3. Create desktop duplication
        let duplication = output1.DuplicateOutput(&device).map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "DuplicateOutput",
        })?;

        // 4. Acquire a frame (timeout 500ms)
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        duplication
            .AcquireNextFrame(500, &mut frame_info, &mut resource)
            .map_err(|e| CoreError::Win32 {
                code: e.code().0 as u32,
                context: "AcquireNextFrame",
            })?;

        let resource = resource.ok_or(CoreError::Win32 {
            code: 0,
            context: "frame resource null",
        })?;
        let texture: ID3D11Texture2D = resource.cast().map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "texture cast",
        })?;

        // 5. Get dimensions
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);
        let width = desc.Width as usize;
        let height = desc.Height as usize;

        // 6. Create CPU-readable staging texture
        let staging_desc = D3D11_TEXTURE2D_DESC {
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
            ..desc
        };
        let mut staging: Option<ID3D11Texture2D> = None;
        device
            .CreateTexture2D(&staging_desc, None, Some(&mut staging))
            .map_err(|e| CoreError::Win32 {
                code: e.code().0 as u32,
                context: "CreateTexture2D staging",
            })?;
        let staging = staging.ok_or(CoreError::Win32 {
            code: 0,
            context: "staging texture null",
        })?;

        // 7. Copy frame to staging texture
        let resource_view: ID3D11Resource = texture.cast().map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "texture resource cast",
        })?;
        let staging_view: ID3D11Resource = staging.cast().map_err(|e| CoreError::Win32 {
            code: e.code().0 as u32,
            context: "staging resource cast",
        })?;
        context.CopyResource(&staging_view, &resource_view);

        // 8. Map and read pixels
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        context
            .Map(&staging_view, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .map_err(|e| CoreError::Win32 {
                code: e.code().0 as u32,
                context: "Map",
            })?;

        let pitch = mapped.RowPitch as usize;
        let data = std::slice::from_raw_parts(mapped.pData as *const u8, pitch * height);

        // 9. Convert BGRA to RGBA
        let mut rgba = vec![0u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let src = y * pitch + x * 4;
                let dst = (y * width + x) * 4;
                rgba[dst] = data[src + 2]; // R (from B)
                rgba[dst + 1] = data[src + 1]; // G
                rgba[dst + 2] = data[src]; // B (from R)
                rgba[dst + 3] = 255; // A
            }
        }

        context.Unmap(&staging_view, 0);
        duplication
            .ReleaseFrame()
            .map_err(|e| CoreError::Win32 {
                code: e.code().0 as u32,
                context: "ReleaseFrame",
            })?;

        // 10. Encode to PNG
        let png_bytes = encode_png_rgba(&rgba, width as u32, height as u32)?;

        Ok(png_bytes)
    }
}

fn encode_png_rgba(
    rgba_data: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, CoreError> {
    use image::RgbaImage;

    let img = RgbaImage::from_raw(width, height, rgba_data.to_vec())
        .ok_or(CoreError::Win32 {
            code: 0,
            context: "RgbaImage from_raw",
        })?;

    let mut png_bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )
    .map_err(|_| CoreError::Win32 {
        code: 0,
        context: "PNG encode",
    })?;

    Ok(png_bytes)
}
