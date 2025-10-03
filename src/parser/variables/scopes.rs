#[cfg(test)]
mod tests {
    use crate::PowerShellSession;

    #[test]
    fn test_new_scope() {
        let mut p = PowerShellSession::new();
        let input = r#"$v = 5;& { $v = 10};$v"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(
            s.result().to_string(),
            "5".to_string()
        );
    }

    #[test]
    fn test_current_scope() {
        let mut p = PowerShellSession::new();
        let input = r#"$v = 5;. { $v = 10};$v"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(
            s.result().to_string(),
            "10".to_string()
        );
    }
}