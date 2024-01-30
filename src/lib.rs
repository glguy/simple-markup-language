use sml::Decoder;
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

pub fn parse(bytes: &[u8]) -> Result<Element, Error> {
    let txt = reliabletext::decode(bytes).map_err(Error::ReliableTextError)?;

    let mut rows = reliabletext::Lines::new(&txt).enumerate();
    let mut decoder = Decoder::new();
    let mut result = None;
    for (line_no, line) in &mut rows {
        let row = wsv::parse_row(&line).map_err(|e| Error::WsvError(line_no, e))?;
        if result.is_none() {
            result = decoder
                .add_row(row)
                .map_err(|e| Error::SmlError(line_no, e))?;
        } else if !row.is_empty() {
            return Err(Error::SmlError(line_no, sml::Error::TooManyRoots));
        }
    }
    match result {
        None => Err(Error::SmlError(0, sml::Error::MissingEnd)),
        Some(elt) => Ok(elt),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sml::{Element, Node};

    #[test]
    fn test() {
        let bytes = "\u{feff}Root\nEnd".as_bytes();
        let elt = parse(bytes).unwrap();
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
        let elt = parse(bytes).unwrap();
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
        let elt = parse(bytes).unwrap();
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
