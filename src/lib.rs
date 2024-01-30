pub use sml::Element;

pub mod reliabletext;
pub mod sml;
pub mod wsv;

#[derive(Debug)]
pub enum Error {
    ReliableTextError(reliabletext::Error),
    WsvError(usize, usize),
    SmlError(usize, sml::Error),
}

pub fn parse_bytes(bytes: &[u8]) -> Result<Element, Error> {
    let txt = reliabletext::decode(bytes).map_err(Error::ReliableTextError)?;
    parse_lines(reliabletext::lines(&txt))
}

pub fn parse_lines<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Element, Error> {
    let rows = lines
        .enumerate()
        .map(|(line_no, line)| wsv::parse_row(line).map_err(|e| Error::WsvError(line_no, e)))
        .collect::<Result<Vec<Vec<Option<String>>>, Error>>()?;
    sml::parse_rows(rows).map_err(|(line_no, err)| Error::SmlError(line_no, err))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sml::{Element, Node};

    #[test]
    fn test() {
        let bytes = "\u{feff}Root\nEnd".as_bytes();
        let elt = parse_bytes(bytes).unwrap();
        assert_eq!(
            elt,
            sml::Element {
                title: "Root".to_string(),
                children: vec![]
            }
        )
    }

    #[test]
    fn test1() {
        let bytes = "\u{feff}Root\nx y -#Comment\nEnd".as_bytes();
        let elt = parse_bytes(bytes).unwrap();
        assert_eq!(
            elt,
            sml::Element {
                title: "Root".to_string(),
                children: vec![Node::Attr(
                    "x".to_string(),
                    vec![Some("y".to_string()), None]
                )]
            }
        )
    }

    #[test]
    fn game_cfg_example() {
        let bytes = "\u{feff}Configuration
        Video
          Resolution 1280 720
          RefreshRate 60
          Fullscreen true
        End
        Audio
          Volume 100
          Music 80
        End
        Player
          Name \"Hero 123\"
        End
      End"
        .as_bytes();
        let elt = parse_bytes(bytes).unwrap();
        assert_eq!(
            elt,
            Element {
                title: "Configuration".to_string(),
                children: vec![
                    Node::Elt(Element {
                        title: "Video".to_string(),
                        children: vec![
                            Node::Attr(
                                "Resolution".to_string(),
                                vec![Some("1280".to_string()), Some("720".to_string())]
                            ),
                            Node::Attr("RefreshRate".to_string(), vec![Some("60".to_string())]),
                            Node::Attr("Fullscreen".to_string(), vec![Some("true".to_string())])
                        ]
                    }),
                    Node::Elt(Element {
                        title: "Audio".to_string(),
                        children: vec![
                            Node::Attr("Volume".to_string(), vec![Some("100".to_string())]),
                            Node::Attr("Music".to_string(), vec![Some("80".to_string())])
                        ]
                    }),
                    Node::Elt(Element {
                        title: "Player".to_string(),
                        children: vec![Node::Attr(
                            "Name".to_string(),
                            vec![Some("Hero 123".to_string())]
                        )]
                    })
                ]
            }
        )
    }
}
