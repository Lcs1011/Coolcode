use pretty_assertions::assert_eq;

use super::*;

#[test]
fn yaml_front_matter_requires_marker_line() {
    let ranges = markdown_protected_ranges("--- heading\nbody\n").unwrap();

    assert_eq!(ranges, Vec::<(usize, usize)>::new());
}

#[test]
fn protected_ranges_cover_markdown_regions() {
    let text = "---\ntitle: Demo\n---\n\nvisible `inline code` text\n<!-- comment -->\n```rust\ncode\n```\nvisible\n";
    let ranges = markdown_protected_ranges(text).unwrap();

    let inline_start = text.find("`inline code`").unwrap();
    let inline_end = inline_start + "`inline code`".len();
    let comment_start = text.find("<!-- comment -->").unwrap();
    let comment_end = comment_start + "<!-- comment -->".len();
    let fence_start = text.find("```rust").unwrap();
    let fence_end = text.find("visible\n").unwrap();

    assert!(range_is_protected(0, "---\ntitle: Demo\n---\n".len(), &ranges));
    assert!(range_is_protected(inline_start, inline_end, &ranges));
    assert!(range_is_protected(comment_start, comment_end, &ranges));
    assert!(range_is_protected(fence_start, fence_end, &ranges));
}

#[test]
fn duplicate_targets_require_occurrence_choice() {
    let matches = find_target_matches("alpha beta alpha", "alpha");

    assert_eq!(matches, vec![(0, 5), (11, 16)]);
}

#[test]
fn annotation_direction_prefixes_use_arrows() {
    assert_eq!(
        annotation_direction_prefix(CToolMarkdownAnnotationDirection::Up),
        "↑"
    );
    assert_eq!(
        annotation_direction_prefix(CToolMarkdownAnnotationDirection::Down),
        "↓"
    );
}

fn range_is_protected(start: usize, end: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|&(protected_start, protected_end)| start >= protected_start && end <= protected_end)
}
