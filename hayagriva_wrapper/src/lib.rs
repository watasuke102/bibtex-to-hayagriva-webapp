use hayagriva::io::from_biblatex_str;
use hayagriva::io::to_yaml_str;
use hayagriva::lang::TitleCase;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn convert_biblatex_to_hayagriva(bib_str: &str) -> String {
    let result = from_biblatex_str(bib_str);
    match result {
        Ok(library) => {
            if library.is_empty() {
                "Error parsing Bibtex".to_string()
            } else {
                let formatted_library = library
                    .into_iter()
                    .map(|mut entry| {
                        if let Some(mut title) = entry.title().cloned() {
                            title.value = title.format_title_case(TitleCase::new()).into();
                            entry.set_title(title);
                        }
                        entry
                    })
                    .collect();
                to_yaml_str(&formatted_library).unwrap_or("Error converting to YAML".to_string())
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
        assert!(result.contains("example"));
        assert!(result.contains("Test Article"));
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
