use super::Process;
use anyhow::Result;
use chrono::Local;
use write_fonts::{
    tables::{
        cmap::PlatformId,
        name::{Name, NameRecord},
    },
    types::NameId,
};

fn format_version(version: f64) -> String {
    format!("Version {:.2}", version)
}

fn generate_unique_id(family_name: &str, version: f64) -> String {
    let now = Local::now();
    format!(
        "{} : {} Regular : {}",
        format_version(version),
        family_name,
        now.format("%d-%m-%Y")
    )
}

pub fn push_name_table(process: &mut Process) -> Result<()> {
    let name = Name {
        name_record: vec![
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::COPYRIGHT_NOTICE,
                string: process.copyright.clone().into(),
            },
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
                name_id: NameId::UNIQUE_ID,
                string: generate_unique_id(&process.family_name, process.family_version).into(),
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
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::POSTSCRIPT_NAME,
                string: process.family_name.replace(" ", "").into(),
            },
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::MANUFACTURER,
                string: process.manufacturer.clone().into(),
            },
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::VENDOR_URL,
                string: process.vendor_url.clone().into(),
            },
            NameRecord {
                platform_id: PlatformId::Windows as u16,
                encoding_id: 1,
                language_id: 0x0409,
                name_id: NameId::LICENSE_DESCRIPTION,
                string: process.license_description.clone().into(),
            },
        ],
        ..Default::default()
    };
    process.builder.add_table(&name)?;
    Ok(())
}
