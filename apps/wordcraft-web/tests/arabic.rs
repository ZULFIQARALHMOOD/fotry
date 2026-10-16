use serde_json::json;
use wordcraft_doc::{Document, ListKind, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_fonts::{FontDb, shape_run};

fn clusters(glyphs: &[wordcraft_fonts::ShapedGlyph]) -> Vec<Vec<u32>> {
    let mut out: Vec<(usize, Vec<u32>)> = Vec::new();
    for glyph in glyphs {
        match out.last_mut() {
            Some((cluster, ids)) if *cluster == glyph.cluster => ids.push(glyph.gid),
            _ => out.push((glyph.cluster, vec![glyph.gid])),
        }
    }
    out.into_iter().map(|(_, ids)| ids).collect()
}

#[test]
fn arabic_fonts_shape_and_mixed_rtl_document_edits_cleanly() {
    let db = FontDb::global();
    for (name, bytes, family) in [
        ("NotoNaskhArabic.ttf", include_bytes!("../fonts/NotoNaskhArabic.ttf").as_slice(), "Noto Naskh Arabic"),
        ("Amiri-Regular.ttf", include_bytes!("../fonts/Amiri-Regular.ttf").as_slice(), "Amiri"),
        ("Cairo.ttf", include_bytes!("../fonts/Cairo.ttf").as_slice(), "Cairo"),
    ] {
        db.install_font(name, bytes).expect("subset Arabic font installs");
        let face = db.face(family, "Regular");
        for ch in ['س', 'م', 'َ', 'ـ', 'ا', 'a', 'Z', '9'] {
            assert!(face.covers(ch), "{family} should cover {ch:?}");
        }
    }

    let naskh = db.face("Noto Naskh Arabic", "Regular");
    let isolated = clusters(&shape_run(&naskh, "ب", &[], |c| c, true));
    let joined = shape_run(&naskh, "ببب", &[], |c| c, true);
    let joined_clusters = clusters(&joined);
    assert_eq!(joined_clusters.len(), 3, "each Arabic letter keeps a logical cluster");
    assert!(joined.iter().all(|g| g.gid != 0), "Arabic letters have real glyphs");
    assert!(joined_clusters.iter().any(|cluster| cluster != &isolated[0]), "joining selects contextual glyph forms");
    assert!(joined.windows(2).all(|pair| pair[0].cluster <= pair[1].cluster), "shaped clusters return in logical order");
    let marked = shape_run(&naskh, "السَّلَامُ ـ عليكم", &[], |c| c, true);
    assert!(marked.iter().all(|g| g.gid != 0), "diacritics and tatweel shape without missing glyphs");
    let fallback = db.fallback_for('س', 0).expect("Arabic has a loaded fallback font");
    assert_eq!(fallback.family, "Noto Naskh Arabic");

    let first = "السَّلَامُ عَلَيْكُمْ";
    let sample = format!("{first}\nمرحبًا بالعالم - English 123 و ١٤٤٨\nالبند الأول\nالبند الثاني");
    let mut doc = Document::from_text(&sample);
    let start = Pos::body(0, 0);
    let end = Pos::body(3, "البند الثاني".len());
    doc.format_paragraphs(&start, &end, &|p| p.bidi = Some(true)).expect("set RTL paragraph direction");
    let list_id = doc.numbering.add_list(ListKind::Numbered);
    doc.format_paragraphs(&Pos::body(2, 0), &end, &|p| {
        p.numbering = Some(wordcraft_doc::props::NumRef { num: list_id, level: 0 });
    })
    .expect("number the final Arabic paragraphs");

    let mut session = Session::new(doc);
    let layout = session.layout();
    let caret_start = layout.caret(&start).expect("RTL paragraph start has a caret");
    let caret_next = match layout.visual_step(&start, true, 0).expect("RTL caret can move visually") {
        wordcraft_engine::layout::hit::VisualStep::Moved(pos) => layout.caret(&pos).expect("moved caret is laid out"),
        wordcraft_engine::layout::hit::VisualStep::Edge { .. } => panic!("RTL paragraph start must have a visual next position"),
    };
    assert!(caret_start.x > caret_next.x, "the caret moves left through Arabic text");
    assert!(!layout.selection_rects(&session.doc, &start, &Pos::body(0, first.len()), 0).is_empty(), "Arabic selection has visible geometry");
    assert!(session.doc.para(StoryRef::Body, &wordcraft_doc::Path::top(2)).expect("first list paragraph").props.numbering.is_some());
    assert_eq!(session.doc.para(StoryRef::Body, &wordcraft_doc::Path::top(2)).expect("first list paragraph").props.bidi, Some(true));

    session.run("select.all", &json!({})).expect("select mixed-text document");
    assert_eq!(session.selected_text(), sample, "Arabic, Latin and both digit styles remain in logical order");
}
