#[derive(Debug)]
enum TemplateToken {
    Literal(String), // 固定文本，例如 "mysql://"
    Variable(String), // 变量名，例如 "user"
}

/// 解析模板字符串，将其分解为 Literal 和 Variable 部分
fn parse_template(template: &str) -> Vec<TemplateToken> {
    let mut parts = Vec::new();
    let mut current_pos = 0;

    // 假设变量名只包含字母、数字和下划线
    // 匹配 {variable_name} 格式的变量
    let variable_re = regex::Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    for caps in variable_re.captures_iter(template) {
        let full_match = caps.get(0).unwrap(); // "{variable_name}"
        let var_name = caps.get(1).unwrap().as_str(); // "variable_name"

        // 添加当前变量之前的字面量
        if full_match.start() > current_pos {
            parts.push(TemplateToken::Literal(
                template[current_pos..full_match.start()].to_string(),
            ));
        }

        // 添加变量部分
        parts.push(TemplateToken::Variable(var_name.to_string()));
        current_pos = full_match.end();
    }

    // 添加最后一个变量之后的字面量（如果有的话）
    if current_pos < template.len() {
        parts.push(TemplateToken::Literal(template[current_pos..].to_string()));
    }

    parts
}

use std::collections::HashMap;

pub fn dynamic_parse_template(
    template: &str,
    input_string: &str,
) -> Option<HashMap<String, String>> {

    let template_parts = parse_template(template);

    let mut result = HashMap::new();
    let mut current_input_index = 0; // 当前在 input_string 中匹配到的位置

    for (i, part) in template_parts.iter().enumerate() {
        match part {
            TemplateToken::Literal(literal) => {
                // 尝试匹配字面量
                if input_string[current_input_index..].starts_with(literal) {
                    current_input_index += literal.len();
                } else {
                    // 字面量不匹配，解析失败
                    return None;
                }
            }
            TemplateToken::Variable(var_name) => {
                // 找到下一个字面量作为当前变量的结束标记
                let mut end_index_for_var = input_string.len(); // 默认变量匹配到字符串末尾

                // 查找下一个 TemplatePart 中的字面量
                for j in (i + 1)..template_parts.len() {
                    if let TemplateToken::Literal(next_literal) = &template_parts[j] {
                        if !next_literal.is_empty() {
                            // 在剩余的 input_string 中查找下一个字面量
                            if let Some(idx) = input_string[current_input_index..].find(next_literal) {
                                end_index_for_var = current_input_index + idx;
                                break;
                            } else {
                                // 理论上应该匹配到的下一个字面量没找到，说明不符合模板
                                return None;
                            }
                        }
                    }
                }

                // 提取变量的值
                if end_index_for_var >= current_input_index {
                    let var_value = &input_string[current_input_index..end_index_for_var];
                    result.insert(var_name.clone(), var_value.to_string());
                    current_input_index = end_index_for_var; // 更新当前输入字符串的索引
                } else {
                    // 变量的结束索引不合理，解析失败
                    return None;
                }
            }
        }
    }

    // 检查是否所有输入字符串都被消耗
    if current_input_index == input_string.len() {
        Some(result)
    } else {
        // 如果输入字符串还有剩余，但模板已经解析完，可能意味着不完全匹配
        // 比如模板是 "abc{x}"，输入是 "abcde"，那么 "de" 未被匹配
        // 根据需求决定这里是返回 Some(result) 还是 None
        // 在此例中，我们认为如果输入字符串没有完全被消耗，则视为不完全匹配
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_mysql_connection_string() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "mysql://root:root@localhost:3306/test_db";

        let result = dynamic_parse_template(template, input).unwrap();
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
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "mysql://admin:p%40ssword@db.example.com:3306/prod?ssl=true";

        let result = dynamic_parse_template(template, input).unwrap();
        assert_eq!(result.get("user"), Some(&"admin".to_string()));
        assert_eq!(result.get("password"), Some(&"p%40ssword".to_string()));
        assert_eq!(result.get("database"), Some(&"prod".to_string()));
    }

    #[test]
    fn test_with_special_chars_in_password() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "mysql://user:p@ssw:ord@localhost:3306/db";

        let result = dynamic_parse_template(template, input).unwrap();
        assert_eq!(result.get("password"), Some(&"p@ssw:ord".to_string()));
    }

    #[test]
    fn test_invalid_input() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "postgres://user:pass@host:port/db";

        assert!(dynamic_parse_template(template, input).is_none());
    }

    #[test]
    fn test_missing_component() {
        let template = "mysql://{user}:{password}@{host}:{port}/{database}";
        let input = "mysql://root@localhost:3306/test_db";

        assert!(dynamic_parse_template(template, input).is_none());
    }

    #[test]
    fn test_custom_template() {
        let template = "{protocol}://{user}@{host}/{path}?{query}";
        let input = "https://john@example.com/api/v1?debug=true";

        let result = dynamic_parse_template(template, input).unwrap();
        assert_eq!(result.get("protocol"), Some(&"https".to_string()));
        assert_eq!(result.get("user"), Some(&"john".to_string()));
        assert_eq!(result.get("path"), Some(&"api/v1".to_string()));
        assert_eq!(result.get("query"), Some(&"debug=true".to_string()));
    }

    #[test]
    fn test_empty_variables() {
        let template = "static/path/with/no/variables";
        let input = "static/path/with/no/variables";

        let result = dynamic_parse_template(template, input).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_multiple_variables_with_same_name() {
        let template = "{var}/{var}/{var}";
        let input = "a/b/c";

        let result = dynamic_parse_template(template, input).unwrap();
        // Should only capture the last occurrence
        assert_eq!(result.get("var"), Some(&"c".to_string()));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_complex_url_with_multiple_components() {
        let template = "{scheme}://{sub}.{domain}.{tld}:{port}/{path}?{query}#{fragment}";
        let input = "https://api.example.com:443/v1/users?page=1#section";

        let result = dynamic_parse_template(template, input).unwrap();
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