use html5ever::serialize::TraversalScope::*;
use html5ever::serialize::{serialize, Serialize, Serializer};
pub use html5ever::serialize::{SerializeOpts, TraversalScope};
use html5ever::QualName;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;
use std::string::ToString;

use crate::tree::{NodeData, NodeRef};

impl Serialize for NodeRef {
    fn serialize<S: Serializer>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope,
    ) -> Result<()> {
        match (traversal_scope, self.data()) {
            (ref scope, &NodeData::Element(ref element)) => {
                if *scope == IncludeNode {
                    let attrs = element.attributes.borrow();

                    // Unfortunately we need to allocate something to hold these &'a QualName
                    let attrs = attrs
                        .map
                        .iter()
                        .map(|(name, attr)| {
                            (
                                QualName::new(
                                    attr.prefix.clone(),
                                    name.ns.clone(),
                                    name.local.clone(),
                                ),
                                &attr.value,
                            )
                        })
                        .collect::<Vec<_>>();

                    serializer.start_elem(
                        element.name.clone(),
                        attrs.iter().map(|&(ref name, value)| (name, &**value)),
                    )?
                }

                for child in self.children() {
                    Serialize::serialize(&child, serializer, IncludeNode)?
                }

                if *scope == IncludeNode {
                    serializer.end_elem(element.name.clone())?
                }
                Ok(())
            }

            (_, &NodeData::DocumentFragment) | (_, &NodeData::Document(_)) => {
                for child in self.children() {
                    Serialize::serialize(&child, serializer, IncludeNode)?
                }
                Ok(())
            }

            (ChildrenOnly(_), _) => Ok(()),

            (IncludeNode, &NodeData::Doctype(ref doctype)) => {
                serializer.write_doctype(&doctype.name)
            }
            (IncludeNode, &NodeData::Text(ref text)) => serializer.write_text(&text.borrow()),
            (IncludeNode, &NodeData::Comment(ref text)) => serializer.write_comment(&text.borrow()),
            (IncludeNode, &NodeData::ProcessingInstruction(ref contents)) => {
                let contents = contents.borrow();
                serializer.write_processing_instruction(&contents.0, &contents.1)
            }
        }
    }
}

impl ToString for NodeRef {
    #[inline]
    fn to_string(&self) -> String {
        let mut u8_vec = Vec::new();
        self.serialize(&mut u8_vec).unwrap();
        String::from_utf8(u8_vec).unwrap()
    }
}

impl NodeRef {
    /// Serialize this node and its descendants in HTML syntax to the given stream.
    #[inline]
    pub fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        serialize(
            writer,
            self,
            SerializeOpts {
                traversal_scope: IncludeNode,
                ..Default::default()
            },
        )
    }

    /// Serialize in HTML syntax to the given stream using the supplied options.
    #[inline]
    pub fn serialize_with_options<W: Write>(&self, writer: &mut W, opts: SerializeOpts) -> Result<()> {
        serialize(
            writer,
            self,
            opts
        )
    }

    /// Serialize this node and its descendants in HTML syntax to a new file at the given path.
    #[inline]
    pub fn serialize_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(&path)?;
        self.serialize(&mut file)
    }
}
