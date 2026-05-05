use super::Process;
use anyhow::Result;
use std::collections::BTreeMap;
use write_fonts::tables::{
    gsub::{Gsub, Ligature, LigatureSet, LigatureSubstFormat1, SubstitutionLookup},
    layout::{
        CoverageTable, Feature, FeatureList, FeatureRecord, LangSys, Lookup, LookupFlag,
        LookupList, Script, ScriptList, ScriptRecord,
    },
};
use write_fonts::types::{GlyphId16, Tag};

pub fn push_gsub_table(process: &mut Process) -> Result<()> {
    let mut ligatures_map: BTreeMap<GlyphId16, LigatureSet> = BTreeMap::new();

    for (key, _) in process.pixmap_pairs.iter() {
        // A ligature is needed if the string consists of multiple codepoints.
        // This catches sequences like "ff" as well as single grapheme clusters like emoji ZWJ sequences.
        if key.chars().count() > 1 {
            let ligature_glyph_id = GlyphId16::new(process.get_glyph_id(key).unwrap());
            let mut components: Vec<GlyphId16> = Vec::new();

            for ch in key.chars() {
                if let Some(gid) = process.get_glyph_id(&ch.to_string()) {
                    components.push(GlyphId16::new(gid));
                } else {
                    components.clear();
                    break;
                }
            }

            if components.len() > 1 {
                let first_component_gid = components.remove(0);
                let ligature = Ligature::new(ligature_glyph_id, components);
                ligatures_map
                    .entry(first_component_gid)
                    .or_insert_with(|| LigatureSet::new(vec![]))
                    .ligatures // This expects OffsetMarker<Ligature>
                    .push(ligature.into());
            }
        }
    }

    if ligatures_map.is_empty() {
        return Ok(());
    }

    let coverage: CoverageTable = ligatures_map.keys().copied().collect();

    // Ligatures within a LigatureSet must be sorted from longest to shortest.
    // If a shorter ligature is a prefix of a longer one, the shorter one will match first
    // unless the longer one appears earlier in the set.
    let mut ligature_sets = Vec::new();
    for mut set in ligatures_map.into_values() {
        set.ligatures.sort_by_key(|lig| std::cmp::Reverse(lig.component_glyph_ids.len()));
        ligature_sets.push(set);
    }

    let ligature_subst_format1 = LigatureSubstFormat1::new(coverage, ligature_sets);

    let ligature_lookup_table = Lookup::new(
        LookupFlag::empty(),
        vec![ligature_subst_format1],
    );

    let substitution_lookup_variant = SubstitutionLookup::Ligature(ligature_lookup_table);

    let lookup_list = LookupList::new(vec![substitution_lookup_variant]);

    let feature = Feature::new(None, vec![0]); // Points to the first lookup in lookup_list
    let feature_list = FeatureList::new(vec![FeatureRecord::new(Tag::new(b"liga"), feature)]);

    let lang_sys = LangSys::new(vec![0]); // Default lang sys uses the 'liga' feature at index 0
    let script = Script::new(Some(lang_sys), vec![]);

    // Register the ligature feature for both Default and Latin scripts to ensure shaper compatibility.
    let script_list = ScriptList::new(vec![
        ScriptRecord::new(Tag::new(b"DFLT"), script.clone()),
        ScriptRecord::new(Tag::new(b"latn"), script),
    ]);

    let gsub = Gsub::new(script_list, feature_list, lookup_list);

    process.builder.add_table(&gsub)?;
    Ok(())
}
