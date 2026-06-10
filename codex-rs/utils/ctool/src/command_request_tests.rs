use pretty_assertions::assert_eq;

use super::*;

#[test]
fn blocked_icon_uses_three_red_markers() {
    assert_eq!(CToolCommandRisk::Blocked.icon(), "🔴🔴🔴");
}

#[test]
fn red_second_confirmation_accepts_y_prefix() {
    assert_eq!(
        parse_red_second_confirmation_input("Y"),
        CToolCommandUserDecision::Approved
    );
    assert_eq!(
        parse_red_second_confirmation_input("yes run it"),
        CToolCommandUserDecision::Approved
    );
}

#[test]
fn python_environment_creation_is_blocked() {
    let classification = classify_command("python -m venv .venv", &default_command_config());

    assert_eq!(
        classification,
        CToolCommandClassification {
            command: "python -m venv .venv".to_string(),
            risk: CToolCommandRisk::Blocked,
            reason: "matched blocked contains rule: venv".to_string(),
        }
    );
}

#[test]
fn blocked_banner_is_visible() {
    let preview = build_command_request_preview(
        "C:\\CodexLab\\codex",
        vec!["python -m venv .venv".to_string()],
        &default_command_config(),
        /*ai_risk_upgrade*/ None,
    )
    .unwrap();

    assert_eq!(preview.final_risk, CToolCommandRisk::Blocked);
    assert!(render_command_request_banner(&preview).contains("🔴🔴🔴 COMMAND REQUEST: BLOCKED"));
}
