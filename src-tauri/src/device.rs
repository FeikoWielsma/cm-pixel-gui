use anyhow::{anyhow, Result};
use hidapi::HidApi;

use crate::canvas::Canvas;

const VID: u16 = 0x2516;
const PID: u16 = 0x021C;
const FRAME_USAGE_PAGE: u16 = 0xFF01;
const PACKETS: usize = 28;
const PAYLOAD: usize = 60;

pub struct PixelDevice {
    dev: hidapi::HidDevice,
}

impl PixelDevice {
    /// Open the frame interface (usage_page 0xFF01).
    pub fn open() -> Result<Self> {
        let api = HidApi::new()?;
        let info = api
            .device_list()
            .find(|d| d.vendor_id() == VID && d.product_id() == PID && d.usage_page() == FRAME_USAGE_PAGE)
            .ok_or_else(|| anyhow!("device not found (usage_page 0xFF01 — is MasterCTRL closed?)"))?;
        let dev = info.open_device(&api)?;
        Ok(PixelDevice { dev })
    }

    /// Non-opening probe: enumerate HID to check presence without claiming the device.
    pub fn is_present() -> bool {
        HidApi::new()
            .map(|api| {
                api.device_list()
                    .any(|d| d.vendor_id() == VID && d.product_id() == PID)
            })
            .unwrap_or(false)
    }

    /// Encode `frame` and write 28 × 65-byte reports to the device.
    /// Report format: [0x00, 0x80, 0xDD, idx_hi, idx_lo] + 60 data bytes.
    /// Pixel order: 556 LEDs × RGB in cell_of_led order, padded to 1680 bytes.
    pub fn send(&mut self, frame: &Canvas) -> Result<()> {
        use crate::layout::cell_of_led;

        let cells = cell_of_led();
        let mut stream = [0u8; PACKETS * PAYLOAD]; // 1680 bytes, last 12 = padding
        for (led, &cell) in cells.iter().enumerate() {
            let y = cell / crate::canvas::W;
            let x = cell % crate::canvas::W;
            let rgb = frame.px[y][x];
            let o = led * 3;
            stream[o] = rgb[0];
            stream[o + 1] = rgb[1];
            stream[o + 2] = rgb[2];
        }

        for i in 0..PACKETS {
            let chunk = &stream[i * PAYLOAD..(i + 1) * PAYLOAD];
            let mut report = [0u8; 65];
            report[0] = 0x00; // report id
            report[1] = 0x80;
            report[2] = 0xDD;
            report[3] = ((i >> 8) & 0xFF) as u8;
            report[4] = (i & 0xFF) as u8;
            report[5..65].copy_from_slice(chunk);
            self.dev.write(&report)?;
        }
        Ok(())
    }
}
