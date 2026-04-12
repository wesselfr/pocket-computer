use core::str;

use heapless::{String, Vec};
use log::info;

pub type NodeId = u16;

struct StringArena {
    last: usize,
    buff: heapless::String<8192>,
}

impl StringArena {
    fn new() -> Self {
        Self {
            last: 0,
            buff: String::new(),
        }
    }
    fn clear(&mut self) {
        self.buff.clear();
        self.last = 0;
    }
    fn push(&mut self, str: &str) -> StrSlice {
        let start = self.last;

        self.buff.insert_str(start, str);
        self.last += str.len();

        StrSlice {
            start: start as u16,
            len: str.len() as u16,
        }
    }
}

// struct Attr {
//     key: StrSlice,
//     value: StrSlice,
// }

pub struct Dom {
    nodes: heapless::Vec<Node, 256>,
    // attrs: heapless::Vec<Attr, 128>,
    strings: StringArena,
}

impl Default for Dom {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            // attrs: Vec::new(),
            strings: StringArena::new(),
        }
    }
}

impl Dom {
    fn clear(&mut self) {
        self.nodes.clear();
        // self.attrs.clear();
        self.strings.clear();
    }
    pub fn resolve(&self, slice: StrSlice) -> &str {
        &self.strings.buff[slice.start as usize..(slice.start + slice.len) as usize]
    }
    pub fn get_node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }
    fn get_node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id as usize]
    }
}

#[derive(Debug)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub node_type: NodeType,
}

impl Node {
    fn element(element: ElementData) -> Self {
        Node {
            parent: None,
            first_child: None,
            next_sibling: None,
            node_type: NodeType::Element(element),
        }
    }
    fn text(text: StrSlice) -> Self {
        Node {
            parent: None,
            first_child: None,
            next_sibling: None,
            node_type: NodeType::Text(text),
        }
    }
}

#[derive(Debug)]
pub enum NodeType {
    Element(ElementData),
    Text(StrSlice),
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: StrSlice,
    attrs: Option<StrSlice>,
}

#[derive(Copy, Clone, Debug)]
pub struct StrSlice {
    start: u16,
    len: u16,
}

pub enum ParseError {
    Invalid,
}

pub struct Parser {
    dom: Dom,
}

fn debug_node(node: &Node, dom: &Dom) {
    match &node.node_type {
        NodeType::Element(e) => {
            info!("Node: {}", dom.resolve(e.tag_name));
        }
        NodeType::Text(slice) => {
            info!("{}", dom.resolve(*slice));
        }
    }
}

fn append_child(parent_id: NodeId, child_id: NodeId, dom: &mut Dom) {
    dom.get_node_mut(child_id).parent = Some(parent_id);

    if dom.get_node(parent_id).first_child.is_none() {
        dom.get_node_mut(parent_id).first_child = Some(child_id);
        return;
    }

    let mut current_id = dom
        .get_node(parent_id)
        .first_child
        .expect("First child must be assigned by now.");

    while let Some(next) = dom.get_node(current_id).next_sibling {
        current_id = next;
    }

    dom.get_node_mut(current_id).next_sibling = Some(child_id);
}

impl Parser {
    pub fn new() -> Self {
        Self {
            dom: Dom::default(),
        }
    }
    pub fn get_dom(&self) -> &Dom {
        &self.dom
    }
    pub fn parse(&mut self, bytes: &[u8]) -> Result<(), ParseError> {
        self.dom.clear();

        let mut stack: Vec<NodeId, 256> = Vec::new();

        // HACK: Always add a root node.
        let root_node = Node::element(ElementData {
            tag_name: self.dom.strings.push("ROOT"),
            attrs: None,
        });
        let _ = self.dom.nodes.push(root_node);
        let _ = stack.push(0);

        let mut i = 0;
        let len = bytes.len();

        while i < len {
            if bytes[i].is_ascii_control() {
                i += 1;
                continue;
            }

            // Start Tag
            if bytes[i] == b'<' {
                i += 1;
                let is_closing = bytes[i] == b'/';
                if is_closing {
                    i += 1;
                }
                let start = i;

                while i < len && bytes[i] != b'>' {
                    i += 1;
                }

                // Sanity check
                if i >= len {
                    return Err(ParseError::Invalid);
                }

                let mut tag_end = i;
                let mut attr_start = i;
                while attr_start > start && !bytes[attr_start].is_ascii_whitespace() {
                    attr_start -= 1;
                }

                // Has attrs
                if attr_start != start {
                    tag_end = attr_start;
                }

                let val = &bytes[start..tag_end];
                let tag = self
                    .dom
                    .strings
                    .push(str::from_utf8(val).unwrap_or_default());
                let void_tag = self.is_void_tag(&tag);

                info!("TAG: {} is_closing: {}", self.dom.resolve(tag), is_closing);

                let attr = if attr_start != start {
                    let val = &bytes[attr_start..i];
                    let attr = self
                        .dom
                        .strings
                        .push(str::from_utf8(val).unwrap_or_default());
                    Some(attr)
                } else {
                    None
                };

                if !is_closing {
                    let node_id = self.dom.nodes.len() as NodeId;
                    self.dom
                        .nodes
                        .push(Node::element(ElementData {
                            tag_name: tag,
                            attrs: attr,
                        }))
                        .unwrap_or_default();

                    if let Some(parent) = stack.last().copied() {
                        append_child(parent, node_id, &mut self.dom);
                    }

                    if !void_tag {
                        stack.push(node_id).unwrap_or_default();
                    }
                } else {
                    if !void_tag {
                        let Some(last_tag) = stack.last().copied() else {
                            return Err(ParseError::Invalid);
                        };

                        let NodeType::Element(element) = &self.dom.get_node(last_tag).node_type
                        else {
                            return Err(ParseError::Invalid);
                        };

                        if self.dom.resolve(element.tag_name) != self.dom.resolve(tag) {
                            return Err(ParseError::Invalid);
                        }

                        stack.pop();
                    }
                }

                i += 1;
            } else {
                // Text
                let start = i;
                while i < len && bytes[i] != b'<' {
                    i += 1;
                }

                let val = &bytes[start..i];
                let text = self
                    .dom
                    .strings
                    .push(str::from_utf8(val).unwrap_or_default());

                let node_id = self.dom.nodes.len() as NodeId;
                self.dom.nodes.push(Node::text(text)).unwrap_or_default();
                if let Some(parent) = stack.last().copied() {
                    append_child(parent, node_id, &mut self.dom);
                }
            }
        }

        Ok(())
    }

    fn is_void_tag(&self, tag: &StrSlice) -> bool {
        matches!(
            self.dom.resolve(*tag),
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
    }
}
