//! Lifted whole out of `builder.rs`. See that module for what these check.

use super::*;

fn a_part(at: [f32; 3], stage: &str) -> Placed {
    Placed {
        part: "wall-2".to_string(),
        at,
        yaw: 0.0,
        tilt: 0.0,
        ramp: None,
        shade: 0.7,
        stage: stage.to_string(),
        flip: false,
        group: None,
        loose: false,
    }
}

/// All three shapes a work has ever had open, and become levels.
///
/// A maker's buildings outlive the format they were written in, so this is the
/// promise that matters most in the whole file.
#[test]
fn every_shape_a_work_was_ever_saved_in_opens() {
    // Format 2: levels, said outright.
    let now = Workbench {
        format: 2,
        name: "now".into(),
        parts: Vec::new(),
        stages: Vec::new(),
        levels: vec![
            Level {
                name: "base".into(),
                phases: vec![vec![a_part([0.0; 3], "footing")]],
            },
            Level {
                name: "forge".into(),
                phases: vec![vec![a_part([0.0; 3], "walls")]],
            },
        ],
    };
    let read = levels_of(&now);
    assert_eq!(read.len(), 2);
    assert_eq!(read[1].name, "forge");

    // Format 1: phases with no levels, which become one level's phases.
    let older = Workbench {
        format: 1,
        name: "older".into(),
        parts: Vec::new(),
        stages: vec![
            vec![a_part([0.0; 3], "footing")],
            vec![a_part([0.0; 3], "walls")],
        ],
        levels: Vec::new(),
    };
    let read = levels_of(&older);
    assert_eq!(read.len(), 1, "an older work is one level");
    assert_eq!(read[0].phases.len(), 2, "and keeps both its phases");

    // Older still: one flat list, whose phases are inferred from the tags.
    let eldest = Workbench {
        format: 1,
        name: "eldest".into(),
        parts: vec![a_part([0.0; 3], "footing"), a_part([1.0, 0.0, 0.0], "roof")],
        stages: Vec::new(),
        levels: Vec::new(),
    };
    let read = levels_of(&eldest);
    assert_eq!(read.len(), 1);
    assert!(
        read[0].phases.len() > 1,
        "a flat work should come back with the steps it already rose in"
    );
    // The last phase is the whole finished building.
    assert_eq!(read[0].phases.last().unwrap().len(), 2);
}

/// Every level is measured from the FIRST level's middle.
///
/// An upgrade lands on the building it upgrades. Recentring each level on its
/// own bounds would shunt the whole thing sideways the day a wing was added,
/// which is the one thing about levels that cannot be seen by looking at one.
#[test]
fn an_upgrade_stands_where_the_building_stands() {
    let palette = crate::look::bench_palette();
    // A base centred on the origin, and an upgrade that only adds to +X - so
    // its own middle is well away from the base's.
    let base = vec![a_part([0.0, 0.0, 0.0], "walls")];
    let upgraded = vec![
        a_part([0.0, 0.0, 0.0], "walls"),
        a_part([8.0, 0.0, 0.0], "walls"),
    ];
    let work = Workbench {
        format: 2,
        name: "blacksmith".into(),
        parts: Vec::new(),
        stages: Vec::new(),
        levels: vec![
            Level {
                name: "base".into(),
                phases: vec![base.clone()],
            },
            Level {
                name: "forge".into(),
                phases: vec![upgraded],
            },
        ],
    };
    let (json, boxes, _) = bake_a_work(&work, &palette, "blacksmith");

    // The top level is the BASE finished, so a format 1 reader is unaffected by
    // the existence of an upgrade.
    let alone = Workbench {
        format: 2,
        name: "blacksmith".into(),
        parts: Vec::new(),
        stages: Vec::new(),
        levels: vec![Level {
            name: "base".into(),
            phases: vec![base],
        }],
    };
    let (_, only, _) = bake_a_work(&alone, &palette, "blacksmith");
    assert_eq!(
        boxes, only,
        "adding an upgrade changed what a format 1 reader sees"
    );

    // The wall the two levels share is written at the SAME place in both.
    let at_zero = json.matches("\"at\": [0.0000, 1.2500, 0.0000]").count();
    assert!(
        at_zero >= 2,
        "the shared wall moved between levels - the origin is not shared:\n{json}"
    );
    assert!(json.contains("\"format\": 2"));
    assert!(json.contains("\"levels\""));
}
