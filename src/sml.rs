#[derive(Eq, PartialEq, Debug)]
pub enum Node {
    Elt(Element),
    Attr(String, Vec<Option<String>>),
}

#[derive(Eq, PartialEq, Debug)]
pub struct Element {
    pub title: String,
    pub children: Vec<Node>,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    MissingEnd,
    ExtraEnd,
    BadRoot,
    NullTitle,
    NullAttribute,
    EmptySml,
    TooManyRoots,
}

pub fn parse_rows<'a>(rows: Vec<Vec<Option<String>>>) -> Result<Element, (usize, Error)> {
    let mut cxt: Vec<Element> = vec![];
    let mut result = None;
    for (line_no, mut row) in rows.into_iter().enumerate() {
        if result.is_none() {
            if row.len() == 1 {
                // Singleton rows start and end elements
                if row[0].iter().all(|x| x.eq_ignore_ascii_case("end")) {
                    // "end" and null are always element terminators

                    // Current element ended, pop it off the context if it exists
                    let elt = cxt.pop().ok_or((line_no, Error::BadRoot))?;

                    // If the context is empty, the root element is complete
                    if cxt.is_empty() {
                        result = Some(elt);
                    } else {
                        // Context isn't empty, add this finished element to current element
                        cxt.last_mut()
                            .ok_or((line_no, Error::ExtraEnd))?
                            .children
                            .push(Node::Elt(elt));
                    }
                } else {
                    let title = row.remove(0).ok_or((line_no, Error::NullTitle))?;
                    cxt.push(Element {
                        title,
                        children: vec![],
                    });
                }
            } else if row.len() > 1 {
                // Larger rows define attributes of the current element
                let key = row.remove(0).ok_or((line_no, Error::NullAttribute))?;
                cxt.last_mut()
                    .ok_or((line_no, Error::BadRoot))?
                    .children
                    .push(Node::Attr(key, row))
            }
        } else if !row.is_empty() {
            return Err((line_no, Error::TooManyRoots));
        }
    }
    result.ok_or((0, Error::MissingEnd))
}
