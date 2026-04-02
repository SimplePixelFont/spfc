use anyhow::Result;
use write_fonts::tables::gasp::{Gasp, GaspRange, GaspRangeBehavior};

use crate::builders::Process;

pub fn push_gasp_table(process: &mut Process) -> Result<()> {
    let gasp = Gasp {
        version: 1,
        num_ranges: 1,
        gasp_ranges: vec![GaspRange {
            range_max_ppem: 65535,
            range_gasp_behavior: GaspRangeBehavior::GASP_GRIDFIT
                | GaspRangeBehavior::GASP_DOGRAY,
        }],
    };
    process.builder.add_table(&gasp)?;
    Ok(())
}
