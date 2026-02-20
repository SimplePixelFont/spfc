use super::Process;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use write_fonts::{
    tables::head::{Flags, Head, MacStyle},
    types::{Fixed, LongDateTime},
};

fn unix_to_mac_epoch(unix_time: u64) -> i64 {
    const MAC_EPOCH_OFFSET: u64 = 2082844800;
    (unix_time + MAC_EPOCH_OFFSET) as i64
}

pub fn push_head_table(process: &mut Process) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mac_timestamp = unix_to_mac_epoch(now);

    let head = Head {
        units_per_em: process.units_per_em,
        x_min: 0,
        y_min: -(process.descender_pixels * process.target_pixel_size),
        x_max: process.max_pixel_width * process.target_pixel_size,
        y_max: ((process.max_pixel_height - process.descender_pixels) * process.target_pixel_size)
            as i16,
        mac_style: MacStyle::empty(),
        font_revision: Fixed::from_f64(process.family_version),
        created: LongDateTime::new(mac_timestamp),
        modified: LongDateTime::new(mac_timestamp),
        lowest_rec_ppem: 8,
        flags: Flags::BASELINE_AT_Y_0 | Flags::LSB_AT_X_0 | Flags::FORCE_INTEGER_PPEM,
        ..Default::default()
    };
    process.builder.add_table(&head)?;
    Ok(())
}
