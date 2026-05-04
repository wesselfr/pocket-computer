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

pub struct Dom {
    nodes: heapless::Vec<Node, 512>,
    attrs: heapless::Vec<Attr, 128>,
    strings: StringArena,
}

impl Default for Dom {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            attrs: Vec::new(),
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

    pub fn get_attrs(&self, slice: AttrSlice) -> &[Attr] {
        let start = slice.start as usize;
        let end = start + slice.len as usize;
        &self.attrs[start..end]
    }
    pub fn get_attr_value(&self, element: &ElementData, key: &str) -> Option<StrSlice> {
        self.get_attrs(element.attrs)
            .iter()
            .find(|attr| self.resolve(attr.key).eq_ignore_ascii_case(key))
            .and_then(|attr| attr.value)
    }
    pub fn get_attr_value_as_str(&self, element: &ElementData, key: &str) -> Option<&str> {
        self.get_attrs(element.attrs)
            .iter()
            .find(|attr| self.resolve(attr.key).eq_ignore_ascii_case(key))
            .and_then(|attr| attr.value)
            .map(|slice| self.resolve(slice))
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
    pub attrs: AttrSlice,
}

#[derive(Copy, Clone, Debug)]
pub struct StrSlice {
    start: u16,
    len: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct AttrSlice {
    start: u16,
    len: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct Attr {
    pub key: StrSlice,
    pub value: Option<StrSlice>,
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
            attrs: AttrSlice { start: 0, len: 0 },
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

                let mut tag_end = start;
                while tag_end < i && !bytes[tag_end].is_ascii_whitespace() {
                    tag_end += 1;
                }

                let mut attr_start = tag_end;
                while attr_start < i && bytes[attr_start].is_ascii_whitespace() {
                    attr_start += 1;
                }

                let attrs = if attr_start < i {
                    self.parse_attrs(&bytes[attr_start..i])
                } else {
                    AttrSlice { start: 0, len: 0 }
                };

                let tag = self
                    .dom
                    .strings
                    .push(str::from_utf8(&bytes[start..tag_end]).unwrap_or_default());
                let void_tag = self.is_void_tag(&tag);

                if !is_closing {
                    let node_id = self.dom.nodes.len() as NodeId;
                    self.dom
                        .nodes
                        .push(Node::element(ElementData {
                            tag_name: tag,
                            attrs,
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

    fn parse_attrs(&mut self, bytes: &[u8]) -> AttrSlice {
        let start_index = self.dom.attrs.len();
        let mut i = 0;

        while i < bytes.len() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if i >= bytes.len() {
                break;
            }

            let key_start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' {
                i += 1;
            }

            let key = self
                .dom
                .strings
                .push(str::from_utf8(&bytes[key_start..i]).unwrap_or_default());

            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            let value = if i < bytes.len() && bytes[i] == b'=' {
                i += 1;

                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }

                if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                    let quote = bytes[i];
                    i += 1;

                    let value_start = i;

                    while i < bytes.len() && bytes[i] != quote {
                        i += 1;
                    }

                    let value = self
                        .dom
                        .strings
                        .push(str::from_utf8(&bytes[value_start..i]).unwrap_or_default());

                    if i < bytes.len() {
                        i += 1;
                    }

                    Some(value)
                } else {
                    let value_start = i;

                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }

                    Some(
                        self.dom
                            .strings
                            .push(str::from_utf8(&bytes[value_start..i]).unwrap_or_default()),
                    )
                }
            } else {
                None
            };

            let _ = self.dom.attrs.push(Attr { key, value });
        }

        AttrSlice {
            start: start_index as u16,
            len: (self.dom.attrs.len() - start_index) as u16,
        }
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
