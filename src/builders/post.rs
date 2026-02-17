use super::Process;
use anyhow::Result;
use write_fonts::{
    tables::post::Post,
    types::{FWord, Version16Dot16},
};

pub fn push_post_table(process: &mut Process) -> Result<()> {
    let post = Post {
        version: Version16Dot16::VERSION_3_0,
        underline_position: FWord::new(0),
        underline_thickness: FWord::new(process.target_pixel_size),
        is_fixed_pitch: 0,
        min_mem_type42: 0,
        max_mem_type42: 0,
        min_mem_type1: 0,
        max_mem_type1: 0,
        ..Default::default()
    };
    process.builder.add_table(&post)?;
    Ok(())
}
