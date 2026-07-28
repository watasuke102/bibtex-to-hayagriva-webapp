use hayagriva::io::from_biblatex_str;
use hayagriva::io::to_yaml_str;
use hayagriva::lang::TitleCase;
use wasm_bindgen::prelude::*;

fn citation_key_from_title(title: &str) -> String {
    let words = title
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .take(6)
        .map(str::to_lowercase)
        .collect::<Vec<_>>();

    format!("bib_{}", words.join("_"))
}

fn replace_citation_keys(yaml: &str, citation_keys: &[String]) -> String {
    let mut citation_keys = citation_keys.iter();
    let mut result = yaml
        .lines()
        .map(|line| {
            if !line.starts_with(char::is_whitespace) && line.ends_with(':') {
                if let Some(key) = citation_keys.next() {
                    return format!("{key}:");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");

    if yaml.ends_with('\n') {
        result.push('\n');
    }

    result
}

#[wasm_bindgen]
pub fn convert_biblatex_to_hayagriva(bib_str: &str) -> String {
    let result = from_biblatex_str(bib_str);
    match result {
        Ok(library) => {
            if library.is_empty() {
                "Error parsing Bibtex".to_string()
            } else {
                let mut citation_keys = Vec::with_capacity(library.len());
                let formatted_library = library
                    .into_iter()
                    .map(|mut entry| {
                        if let Some(mut title) = entry.title().cloned() {
                            citation_keys.push(citation_key_from_title(&title.value.to_string()));
                            title.value = title.format_title_case(TitleCase::new()).into();
                            entry.set_title(title);
                        } else {
                            citation_keys.push(entry.key().to_string());
                        }
                        entry
                    })
                    .collect();
                to_yaml_str(&formatted_library)
                    .map(|yaml| replace_citation_keys(&yaml, &citation_keys))
                    .unwrap_or("Error converting to YAML".to_string())
            }
        }
        Err(errors) => {
            let mut error_str = String::new();
            error_str.push_str("Error parsing Bibtex: \n");
            for error in errors {
                error_str.push_str("* ");
                error_str.push_str(&error.to_string());
                error_str.push('\n');
            }
            error_str
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_bibtex() {
        let bibtex = r#"
@article{example,
    title={Test Article},
    author={John Doe},
    journal={Test Journal},
    year={2023},
}
"#;

        let result = convert_biblatex_to_hayagriva(bibtex);

        // Should not be an error message
        assert!(!result.starts_with("Error parsing Bibtex"));
        assert!(!result.starts_with("Error converting to YAML"));

        // Should contain YAML-like content
        assert!(result.contains("bib_test_article:"));
        assert!(result.contains("Test Article"));
    }

    #[test]
    fn test_citation_key_from_title() {
        assert_eq!(
            citation_key_from_title("MidAir-Focus: A Novel Approach for Spatial Interaction"),
            "bib_midair_focus_a_novel_approach_for"
        );
    }

    #[test]
    fn test_convert_invalid_bibtex() {
        let invalid_bibtex = "this is not valid bibtex";

        let result = convert_biblatex_to_hayagriva(invalid_bibtex);

        // Should be an empty library since invalid bibtex just results in no entries
        assert_eq!(result, "Error parsing Bibtex");
    }

    // This verifies that the title-case settings passed to Hayagriva produce the expected academic title format.
    #[test]
    fn test_convert_capitalizes_only_title() {
        let bibtex = r#"
@article{example,
    title={a revolutional method for mid-air interaction in virtual reality},
    journal={journal of virtual reality},
}
"#;

        let result = convert_biblatex_to_hayagriva(bibtex);

        assert!(result
            .contains("title: A Revolutional Method for Mid-Air Interaction in Virtual Reality"));
        assert!(result.contains("title: journal of virtual reality"));
        assert!(!result.contains("title: Journal of Virtual Reality"));
    }
}
