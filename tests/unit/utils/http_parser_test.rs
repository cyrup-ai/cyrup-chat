// Unit tests for HTTP header parsing utilities
// Extracted from src/utils/http_parser.rs for proper test organization

use cyrup::utils::http_parser::*;

#[test]
fn test_parse_simple_link() {
    let header = r#"<https://api.example.com/users?page=2>; rel="next""#;
    let links = parse_link_header(header).expect("Failed to parse valid link header");
    
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].url, "https://api.example.com/users?page=2");
    assert_eq!(links[0].rel(), Some("next"));
}

#[test]
fn test_parse_multiple_links() {
    let header = r#"<https://api.example.com/users?page=2>; rel="next", <https://api.example.com/users?page=1>; rel="prev""#;
    let links = parse_link_header(header).expect("Failed to parse multiple links");
    
    assert_eq!(links.len(), 2);
    assert!(links.iter().any(|l| l.has_rel("next")));
    assert!(links.iter().any(|l| l.has_rel("prev")));
}

#[test]
fn test_extract_max_id() {
    let url = "https://api.example.com/users?max_id=12345&limit=20";
    let max_id = extract_max_id_from_url(url);
    
    assert_eq!(max_id, Some("12345".to_string()));
}

#[test]
fn test_extract_max_id_no_param() {
    let url = "https://api.example.com/users?limit=20";
    let max_id = extract_max_id_from_url(url);
    
    assert_eq!(max_id, None);
}

#[test]
fn test_parse_empty_header() {
    let header = "";
    let links = parse_link_header(header).expect("Should handle empty header");
    
    assert_eq!(links.len(), 0);
}

#[test]
fn test_parse_malformed_header() {
    let header = "not a valid link header";
    let result = parse_link_header(header);
    
    // Should either return empty vec or error gracefully
    assert!(result.is_ok() || result.is_err());
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_extract_max_id_property(
            max_id in "[0-9]{1,10}",
            base_url in "https://[a-z]+\\.[a-z]{2,}/[a-z]+"
        ) {
            let url = format!("{}?max_id={}&other=value", base_url, max_id);
            let extracted = extract_max_id_from_url(&url);
            
            prop_assert_eq!(extracted, Some(max_id));
        }
    }
}