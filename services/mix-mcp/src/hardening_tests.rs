use super::*;
use std::os::unix::fs::symlink;

#[test]
fn indexing_refuses_directory_links_and_confines_markdown_links() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("workspace");
    let outside = tmp.path().join("outside");
    std::fs::create_dir_all(root.join("_doc")).unwrap();
    std::fs::create_dir_all(outside.join("_plan")).unwrap();
    std::fs::write(outside.join("secret.md"), "secret").unwrap();
    std::fs::write(root.join("_doc/real.md"), "real").unwrap();
    symlink(&outside, root.join("linked")).unwrap();
    symlink(&outside, root.join("_memory")).unwrap();
    symlink(&outside, root.join("_doc/subdir")).unwrap();
    symlink(outside.join("secret.md"), root.join("_notes.md")).unwrap();
    symlink(outside.join("secret.md"), root.join("_doc/escape.md")).unwrap();
    symlink(root.join("_doc/real.md"), root.join("_doc/inside.md")).unwrap();
    let walked = walkdir(&root);
    assert!(!walked.contains(&root.join("linked/secret.md")));
    assert!(!walked.contains(&root.join("_doc/subdir/secret.md")));
    assert_eq!(find_content_dirs(&root), vec![(root.join("_doc"), "doc")]);
    let mut files = collect_md_files(&root.join("_doc"));
    files.sort();
    assert_eq!(
        files,
        vec![root.join("_doc/inside.md"), root.join("_doc/real.md")]
    );
    assert!(walkdir(&root.join("linked")).is_empty());
    assert!(collect_md_files(&root.join("linked")).is_empty());
    symlink(root.join("_doc/real.md"), root.join("nested_notes.md")).unwrap();
    std::fs::remove_file(root.join("_notes.md")).unwrap();
    symlink(root.join("_doc/real.md"), root.join("_notes.md")).unwrap();
    assert!(find_content_dirs(&root).contains(&(root.join("_notes.md"), "notes")));

    let domains = knowledge_settings::DomainsSettings {
        map: std::collections::BTreeMap::from([(
            root.to_string_lossy().into_owned(),
            "fixture".into(),
        )]),
    };
    assert_eq!(checked_workspace(&root, &domains).unwrap(), root);
    assert!(checked_workspace(&root.join("_doc"), &domains).is_ok());
    for path in [&outside, &root.join("linked"), &root.join("../outside")] {
        assert!(
            checked_workspace(path, &domains)
                .unwrap_err()
                .contains("refused")
        );
    }
}
