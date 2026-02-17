use super::Process;
use anyhow::Result;
use write_fonts::{
    tables::{
        cmap::PlatformId,
        name::{Name, NameRecord},
    },
    types::NameId,
};

fn format_version(version: f64) -> String {
    if version == version.trunc() {
        format!("Version {:.1}", version)
    } else {
        format!("Version {}", version)
    }
}

pub fn push_name_table(process: &mut Process) -> Result<()> {
    let name = Name {
        name_record: vec![
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409, // English US
                name_id: NameId::FAMILY_NAME,
                string: process.family_name.clone().into(),
            },
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::SUBFAMILY_NAME,
                string: "Regular".to_string().into(),
            },
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::FULL_NAME,
                string: format!("{} Regular", process.family_name).into(),
            },
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::VERSION_STRING,
                string: format_version(process.family_version).into(),
            },
        ],
        ..Default::default()
    };
    process.builder.add_table(&name)?;
    Ok(())
}
