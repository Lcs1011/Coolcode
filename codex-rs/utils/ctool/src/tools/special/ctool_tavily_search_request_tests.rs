use crate::command_request::CToolCommandRisk;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

fn input(action: CToolTavilyAction) -> CToolTavilySearchRequestInput {
    CToolTavilySearchRequestInput {
        action,
        query: Some("rust cargo workspace".to_string()),
        url: None,
        source_file: None,
        target: None,
        file_name_hint: None,
        yellow_confirmation: None,
        red_first_confirmation: None,
        red_second_confirmation: None,
    }
}

fn config_with_token() -> TavilySearchConfig {
    TavilySearchConfig {
        tavily_api_key: Some("tvly-test-token".to_string()),
        ..Default::default()
    }
}

#[test]
fn missing_system_token_blocks_network_actions() {
    let plan = classify_tavily_request(&input(CToolTavilyAction::Search), &TavilySearchConfig::default());

    assert_eq!(plan.risk, CToolCommandRisk::Blocked);
    assert_eq!(plan.reason, "missing Tavily token in system config");
}

#[test]
fn disabled_provider_is_blocked_request() {
    let config = TavilySearchConfig {
        provider: "other".to_string(),
        tavily_api_key: Some("tvly-test-token".to_string()),
        ..Default::default()
    };

    let plan = classify_tavily_request(&input(CToolTavilyAction::Search), &config);

    assert_eq!(plan.risk, CToolCommandRisk::Blocked);
    assert_eq!(plan.reason, "unsupported Tavily search provider: other");
}

#[test]
fn image_search_is_red_when_enabled() {
    let config = TavilySearchConfig {
        allow_image_search: true,
        ..config_with_token()
    };

    let plan = classify_tavily_request(&input(CToolTavilyAction::SearchWithImages), &config);

    assert_eq!(plan.risk, CToolCommandRisk::Red);
    assert_eq!(plan.reason, "image search requires red confirmation");
}

#[test]
fn extract_is_yellow_when_otherwise_allowed() {
    let plan = classify_tavily_request(&input(CToolTavilyAction::Extract), &config_with_token());

    assert_eq!(plan.risk, CToolCommandRisk::Yellow);
    assert_eq!(plan.reason, "extract fetches external page content");
}

#[test]
fn blocked_image_formats_are_blocked() {
    let config = TavilySearchConfig {
        allow_image_search: true,
        ..config_with_token()
    };
    let mut request = input(CToolTavilyAction::SearchWithImages);
    request.query = Some("rust logo .svg".to_string());

    let plan = classify_tavily_request(&request, &config);

    assert_eq!(plan.risk, CToolCommandRisk::Blocked);
    assert_eq!(plan.reason, "image search requested blocked image format: .svg");
}

#[test]
fn appends_deduplicated_nested_image_urls() {
    let response = json!({
        "images": [
            "https://example.test/a.png",
            { "url": "https://example.test/b.jpg" }
        ],
        "results": [
            {
                "image": { "image_url": "https://example.test/a.png" },
                "ignored": "https://example.test/not-collected.png"
            }
        ]
    });
    let mut markdown = String::new();

    append_tavily_images_section(&mut markdown, &response);

    assert_eq!(
        markdown,
        "\n## Images\n\n1. https://example.test/a.png\n2. https://example.test/b.jpg\n"
    );
}

#[test]
fn slug_falls_back_for_symbol_only_text() {
    assert_eq!(slugify("!!!"), "tavily_request");
}
