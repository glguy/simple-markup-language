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

pub(crate) struct Decoder {
    cxt: Vec<Element>,
}

impl Decoder {
    pub(crate) fn new() -> Self {
        Self { cxt: vec![] }
    }

    pub(crate) fn add_row(
        &mut self,
        mut row: Vec<Option<String>>,
    ) -> Result<Option<Element>, Error> {
        if row.len() == 1 {
            // Singleton rows start and end elements
            if row[0].iter().all(|x| x.eq_ignore_ascii_case("end")) {
                // "end" and null are always element terminators

                // Current element ended, pop it off the context if it exists
                let elt = self.cxt.pop().ok_or(Error::BadRoot)?;

                // If the context is empty, the root element is complete
                if self.cxt.is_empty() {
                    return Ok(Some(elt));
                }

                // Context isn't empty, add this finished element to current element
                self.cxt
                    .last_mut()
                    .ok_or(Error::ExtraEnd)?
                    .children
                    .push(Node::Elt(elt));
            } else {
                let title = row.remove(0).ok_or(Error::NullTitle)?;
                self.cxt.push(Element {
                    title,
                    children: vec![],
                });
            }
        } else if row.len() > 1 {
            // Larger rows define attributes of the current element
            let key = row.remove(0).ok_or(Error::NullAttribute)?;
            match self.cxt.last_mut() {
                None => return Err(Error::BadRoot),
                Some(elt) => elt.children.push(Node::Attr(key, row)),
            }
        }
        Ok(None)
    }
}
