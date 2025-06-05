use std::collections::HashMap;


#[derive(Debug, PartialEq)]
pub enum TemplateToken {
    Literal(String), // Fixed literal, e.g., "mysql://"
    Variable(String), // Variable name, e.g., "{user}" => "user"
}

/// Analysis the template string and return a vector of TemplateToken.
/// Example:
///    template: "mysql://{user}:{password}@{host}:{port}/{database}"
///    return: [Literal("mysql://"), Variable("user"), Literal(":"), Variable("password"), Literal("@"),
/// Variable("host"), Literal(":"), Variable("port"), Literal("/"), Variable("database")]
pub fn analyze(template: &str) -> Vec<TemplateToken> {
    let mut tokens = Vec::new();
    let mut current_pos = 0;

    // The `variable_re` limits the variable name to start with a letter or underscore,
    // followed by any number of letters, digits, or underscores.
    // This ensures that variables are not confused with substrings in the template.
    let variable_re = regex::Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    // Iterate over all matches of the variable pattern in the template.
    for caps in variable_re.captures_iter(template) {
        let full_match = caps.get(0).unwrap(); // "{variable_name}"
        let var_name = caps.get(1).unwrap().as_str(); // "variable_name"

        if full_match.start() > current_pos {
            tokens.push(TemplateToken::Literal(
                template[current_pos..full_match.start()].to_string(),
            ));
        }

        tokens.push(TemplateToken::Variable(var_name.to_string()));
        current_pos = full_match.end();
    }

    // Add the remaining literal part if any.
    if current_pos < template.len() {
        tokens.push(TemplateToken::Literal(template[current_pos..].to_string()));
    }

    tokens
}


#[cfg(test)]
mod parse_template_tests {
    use super::*;

    #[test]
    fn test_parse_template_with_variables() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Literal("mysql://".to_string()),
            TemplateToken::Variable("user".to_string()),
            TemplateToken::Literal(":".to_string()),
            TemplateToken::Variable("password".to_string()),
            TemplateToken::Literal("@".to_string()),
            TemplateToken::Variable("host".to_string()),
            TemplateToken::Literal(":".to_string()),
            TemplateToken::Variable("port".to_string()),
            TemplateToken::Literal("/".to_string()),
            TemplateToken::Variable("database".to_string()),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_only_literals() {
        let template = "just/a/regular/string";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Literal("just/a/regular/string".to_string()),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_only_variables() {
        let template = "{user}{password}{host}";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Variable("user".to_string()),
            TemplateToken::Variable("password".to_string()),
            TemplateToken::Variable("host".to_string()),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_empty_string() {
        let template = "";
        let result = analyze(template);

        let expected: Vec<TemplateToken> = vec![];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_variable_at_start() {
        let template = "{user}@example.com";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Variable("user".to_string()),
            TemplateToken::Literal("@example.com".to_string()),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_variable_at_end() {
        let template = "prefix_{user}";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Literal("prefix_".to_string()),
            TemplateToken::Variable("user".to_string()),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_consecutive_variables() {
        let template = "{user}{password}";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Variable("user".to_string()),
            TemplateToken::Variable("password".to_string()),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_template_with_complex_pattern() {
        let template = "http://{host}:{port}/api/{version}/{endpoint}?{query}";
        let result = analyze(template);

        let expected = vec![
            TemplateToken::Literal("http://".to_string()),
            TemplateToken::Variable("host".to_string()),
            TemplateToken::Literal(":".to_string()),
            TemplateToken::Variable("port".to_string()),
            TemplateToken::Literal("/api/".to_string()),
            TemplateToken::Variable("version".to_string()),
            TemplateToken::Literal("/".to_string()),
            TemplateToken::Variable("endpoint".to_string()),
            TemplateToken::Literal("?".to_string()),
            TemplateToken::Variable("query".to_string()),
        ];

        assert_eq!(result, expected);
    }
}

/// dynamic_parse_template function to parse the input string and extract the variables.
/// Example:
///    template: "mysql://{user}:{password}@{host}:{port}/{database}"
///    input: "mysql://root:root@localhost:3306/test_db"
///    return: { "user": "root", "password": "root", "host": "localhost", "port": "3306", "database": "test_db" }
///
/// The function will return None if the input string does not match the template.
pub fn parse_variables(
    template: &str,
    input_string: &str,
) -> Option<HashMap<String, String>> {
    let tokens = analyze(template);

    let mut variables = HashMap::new();
    let mut input_cursor = 0;

    for (i, part) in tokens.iter().enumerate() {
        if input_cursor >= input_string.len() {
            break;
        }

        match part {
            // literal part must match the input string exactly.
            // if not match, return None.
            TemplateToken::Literal(literal) => {
                if input_string[input_cursor..].starts_with(literal) {
                    input_cursor += literal.len();
                } else {
                    return None;
                }
            }
            // variable part must match the input string after the literal part and before the next literal part.
            // if not match, return None.
            TemplateToken::Variable(var_name) => {
                // try to match the variable with the current input string
                // if not match, return None
                let mut end_index_for_var = input_string.len();

                let j = i + 1;

                // If there's no more token, then we can use the end of the input string as the end index.
                // Otherwise, we need to locate the next literal after the variable.
                if j >= tokens.len() {
                    variables.insert(var_name.clone(), input_string[input_cursor..].to_string());
                    break;
                }

                // If the next token is NOT a literal, returns None since we can't find the end index for the variable.
                // e.g. we CANNOT split "abcde" as {user}{password} because we don't know the end index for the user variable.
                if let TemplateToken::Literal(next_literal) = &tokens[j] {
                    if !next_literal.is_empty() {
                        if let Some(idx) = input_string[input_cursor..].find(next_literal) {
                            end_index_for_var = input_cursor + idx;
                        } else {
                            return None;
                        }
                    }
                } else {
                    return None;
                }

                // extract the variable value
                if end_index_for_var >= input_cursor {
                    let var_value = &input_string[input_cursor..end_index_for_var];
                    variables.insert(var_name.clone(), var_value.to_string());
                }

                input_cursor = end_index_for_var;
            }
        }
    }

    Some(variables)
}


#[cfg(test)]
mod dynamic_parse_template_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_mysql_connection_string() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "mysql://root:root@localhost:3306/test_db";

        let result = parse_variables(template, input).unwrap();
        let mut expected = HashMap::new();
        expected.insert("user".to_string(), "root".to_string());
        expected.insert("password".to_string(), "root".to_string());
        expected.insert("host".to_string(), "localhost".to_string());
        expected.insert("port".to_string(), "3306".to_string());
        expected.insert("database".to_string(), "test_db".to_string());

        assert_eq!(result, expected);
    }

    #[test]
    fn test_with_query_params() {
        let template = "mysql://{user}:{password}@({host}:{port})/{database}?{options}";
        let input = "mysql://admin:p%40ssword@(db.example.com:3306)/prod?ssl=true";

        let result = parse_variables(template, input).unwrap();
        assert_eq!(result.get("user"), Some(&"admin".to_string()));
        assert_eq!(result.get("password"), Some(&"p%40ssword".to_string()));
        assert_eq!(result.get("database"), Some(&"prod".to_string()));
    }

    #[test]
    fn test_with_special_chars_in_password() {
        let template = "mysql://{user}:{password}@({host}:{port})/{database}";
        let input = "mysql://user:p@ssw:ord@(localhost:3306)/db";

        let result = parse_variables(template, input).unwrap();
        assert_eq!(result.get("password"), Some(&"p@ssw:ord".to_string()));
    }

    #[test]
    fn test_invalid_input() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "postgres://user:pass@host:port/db";

        assert!(parse_variables(template, input).is_none());
    }

    #[test]
    fn test_missing_component() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "mysql://root@localhost:3306/test_db";

        assert!(parse_variables(template, input).is_none());
    }

    #[test]
    fn test_custom_template() {
        let template = "{protocol}://{user}@{host}/{path}?{query}";
        let input = "https://john@example.com/api/v1?debug=true";

        let result = parse_variables(template, input).unwrap();
        assert_eq!(result.get("protocol"), Some(&"https".to_string()));
        assert_eq!(result.get("user"), Some(&"john".to_string()));
        assert_eq!(result.get("path"), Some(&"api/v1".to_string()));
        assert_eq!(result.get("query"), Some(&"debug=true".to_string()));
    }

    #[test]
    fn test_empty_variables() {
        let template = "static/path/with/no/variables";
        let input = "static/path/with/no/variables";

        let result = parse_variables(template, input).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_multiple_variables_with_same_name() {
        let template = "{var}/{var}/{var}";
        let input = "a/b/c";

        let result = parse_variables(template, input).unwrap();
        // Should only capture the last occurrence
        assert_eq!(result.get("var"), Some(&"c".to_string()));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_complex_url_with_multiple_components() {
        let template = "{scheme}://{sub}.{domain}.{tld}:{port}/{path}?{query}#{fragment}";
        let input = "https://api.example.com:443/v1/users?page=1#section";

        let result = parse_variables(template, input).unwrap();
        assert_eq!(result.get("scheme"), Some(&"https".to_string()));
        assert_eq!(result.get("sub"), Some(&"api".to_string()));
        assert_eq!(result.get("domain"), Some(&"example".to_string()));
        assert_eq!(result.get("tld"), Some(&"com".to_string()));
        assert_eq!(result.get("port"), Some(&"443".to_string()));
        assert_eq!(result.get("path"), Some(&"v1/users".to_string()));
        assert_eq!(result.get("query"), Some(&"page=1".to_string()));
        assert_eq!(result.get("fragment"), Some(&"section".to_string()));
    }
}

pub fn fill_template(template: &str, variables: &HashMap<String, String>) -> Option<String> {
    let tokens = analyze(template);
    let mut result = String::new();

    for part in tokens {
        match part {
            TemplateToken::Literal(literal) => {
                result.push_str(&literal);
            }
            TemplateToken::Variable(var_name) => {
                if let Some(value) = variables.get(&var_name) {
                    result.push_str(value);
                }
            }
        }
    }

    Some(result)
}

#[cfg(test)]
mod fill_template_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_fill_template_with_variables() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let mut variables = HashMap::new();
        variables.insert("user".to_string(), "root".to_string());
        variables.insert("password".to_string(), "root".to_string());
        variables.insert("host".to_string(), "localhost".to_string());
        variables.insert("port".to_string(), "3306".to_string());
        variables.insert("database".to_string(), "test_db".to_string());
        let result = fill_template(template, &variables).unwrap();
        assert_eq!(result, "mysql://root:root@localhost:3306/test_db");
    }

    #[test]
    fn test_fill_template_with_missing_variables() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let mut variables = HashMap::new();
        variables.insert("user".to_string(), "root".to_string());
        variables.insert("password".to_string(), "root".to_string());
        variables.insert("host".to_string(), "localhost".to_string());
        let result = fill_template(template, &variables).unwrap();
        assert_eq!(result, "mysql://root:root@localhost:/");
    }

    #[test]
    fn test_fill_template_with_empty_variables() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let variables = HashMap::new();
        let result = fill_template(template, &variables).unwrap();
        assert_eq!(result, "mysql://:@:/");
    }

    #[test]
    fn test_fill_template_with_custom_template() {
        let template = "{protocol}://{user}@{host}/{path}?{query}#{fragment}";
        let mut variables = HashMap::new();
        variables.insert("protocol".to_string(), "https".to_string());
        variables.insert("user".to_string(), "john".to_string());
        variables.insert("host".to_string(), "example.com".to_string());
        variables.insert("path".to_string(), "api/v1".to_string());
        variables.insert("query".to_string(), "debug=true".to_string());
        variables.insert("fragment".to_string(), "section".to_string());

        let result = fill_template(template, &variables).unwrap();
        assert_eq!(result, "https://john@example.com/api/v1?debug=true#section")
    }

    #[test]
    fn test_fill_template_with_special_characters() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let mut variables = HashMap::new();
        variables.insert("user".to_string(), "admin".to_string());
        variables.insert("password".to_string(), "p%40ssw:ord".to_string());
        variables.insert("host".to_string(), "localhost".to_string());
        variables.insert("port".to_string(), "3306".to_string());
        variables.insert("database".to_string(), "prod".to_string());
        let result = fill_template(template, &variables).unwrap();
        assert_eq!(result, "mysql://admin:p%40ssw:ord@localhost:3306/prod");
    }
}