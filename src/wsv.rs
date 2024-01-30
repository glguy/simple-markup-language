pub fn parse_row(line: &str) -> Result<Vec<Option<String>>, usize> {
    enum Context {
        Ready,
        Null,
        Quoted,
        Quoted2,
        QuotedNL,
        Simple,
    }

    let mut row = vec![];
    let mut cell = String::new();
    let mut cxt = Context::Ready;

    for (i, ch) in line.char_indices() {
        match cxt {
            Context::Ready if ch == '"' => cxt = Context::Quoted,
            Context::Ready if ch == '-' => cxt = Context::Null,
            Context::Ready if ch == '#' => break,
            Context::Ready if ch.is_whitespace() => {}
            Context::Ready => {
                cxt = Context::Simple;
                cell.push(ch)
            }

            Context::Null if ch == '"' => return Err(i),
            Context::Null if ch == '#' => break,
            Context::Null if ch.is_whitespace() => {
                cxt = Context::Ready;
                row.push(None)
            }
            Context::Null => {
                cxt = Context::Simple;
                cell.extend(['-', ch]);
            }

            Context::Simple if ch == '"' => return Err(i),
            Context::Simple if ch == '#' => break,
            Context::Simple if ch.is_whitespace() => {
                cxt = Context::Ready;
                row.push(Some(cell));
                cell = String::new();
            }
            Context::Simple => cell.push(ch),

            Context::Quoted if ch == '"' => cxt = Context::Quoted2,
            Context::Quoted => cell.push(ch),

            Context::Quoted2 if ch == '"' => {
                cxt = Context::Quoted;
                cell.push('"');
            }
            Context::Quoted2 if ch == '/' => cxt = Context::QuotedNL,
            Context::Quoted2 if ch == '#' => break,
            Context::Quoted2 if ch.is_whitespace() => {
                cxt = Context::Ready;
                row.push(Some(cell));
                cell = String::new();
            }
            Context::Quoted2 => return Err(i),

            Context::QuotedNL if ch == '"' => {
                cxt = Context::Quoted;
                cell.push('\n');
            }
            Context::QuotedNL => return Err(i),
        }
    }

    match cxt {
        Context::Ready => {}
        Context::Quoted | Context::QuotedNL => return Err(line.len()),
        Context::Null => row.push(None),
        Context::Simple | Context::Quoted2 => row.push(Some(cell)),
    }

    Ok(row)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let row = parse_row("x y z").unwrap();
        assert_eq!(
            row,
            vec![
                Some("x".to_string()),
                Some("y".to_string()),
                Some("z".to_string())
            ]
        )
    }

    #[test]
    fn test_empty() {
        assert_eq!(parse_row("").unwrap(), vec![]);
        assert_eq!(parse_row("    ").unwrap(), vec![]);
    }

    #[test]
    fn test_null() {
        assert_eq!(parse_row("-").unwrap(), vec![None]);
        assert_eq!(parse_row("  -  ").unwrap(), vec![None]);
        assert_eq!(parse_row("- -").unwrap(), vec![None, None]);
        assert_eq!(
            parse_row("- -1").unwrap(),
            vec![None, Some("-1".to_string())]
        );
        assert_eq!(
            parse_row("\"-\" -").unwrap(),
            vec![Some("-".to_string()), None]
        );
    }

    #[test]
    fn parse_quotes() {
        assert_eq!(
            parse_row("\"one\"\"two\" \"\" \"\"/\"\"").unwrap(),
            vec![
                Some("one\"two".to_string()),
                Some("".to_string()),
                Some("\n".to_string())
            ]
        );
    }

    #[test]
    fn quotes_end_by_comment() {
        assert_eq!(
            parse_row("\"one\"#").unwrap(),
            vec![
                Some("one".to_string()),
            ]
        );
    }
}
