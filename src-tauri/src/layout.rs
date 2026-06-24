use std::sync::OnceLock;

pub const NLED: usize = 556;

const LAYOUT_JSON: &str = include_str!("../../reference/layout.json");

static LAYOUT_DATA: OnceLock<[u16; 1024]> = OnceLock::new();
static CELL_DATA: OnceLock<[usize; NLED]> = OnceLock::new();

/// 1024 lamp-index values (row-major, 32 wide). 0 = no LED; 1..=556 = 1-based LED index.
pub fn layout() -> &'static [u16; 1024] {
    LAYOUT_DATA.get_or_init(|| {
        let raw: Vec<u16> = serde_json::from_str(LAYOUT_JSON).expect("bad layout.json");
        raw.try_into().expect("layout.json must have exactly 1024 entries")
    })
}

/// For each LED index (0-based), its flat grid cell (y*32+x).
pub fn cell_of_led() -> &'static [usize; NLED] {
    CELL_DATA.get_or_init(|| {
        let layout = layout();
        let mut cells = [0usize; NLED];
        for (cell, &lamp) in layout.iter().enumerate() {
            if lamp != 0 {
                cells[(lamp - 1) as usize] = cell;
            }
        }
        cells
    })
}
