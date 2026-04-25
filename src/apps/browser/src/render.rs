use crate::apps::browser::src::parser::{Dom, NodeId, NodeType};
use crate::apps::browser::src::regions::{HitAction, HitRegion, MAX_REGIONS};
use crate::graphics::GridTarget;
use crate::input::Rect;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::WebColors;
use heapless::Vec;

#[derive(Copy, Clone)]
struct RenderStyle {
    pub fg: Rgb565,
    pub bg: Rgb565,
    pub uppercase: bool,
    pub indent: u16,
}

impl RenderStyle {
    fn light() -> Self {
        Self {
            fg: Rgb565::CSS_BLACK,
            bg: Rgb565::CSS_WHITE,
            uppercase: false,
            indent: 0,
        }
    }
    fn dark() -> Self {
        Self {
            fg: Rgb565::CSS_WHITE,
            bg: Rgb565::CSS_BLACK,
            uppercase: false,
            indent: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Pos {
    x: u16,
    y: u16,
}

pub struct Renderer {
    pub dark_mode: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self { dark_mode: false }
    }
    pub fn render(
        &self,
        dom: &Dom,
        regions: &mut Vec<HitRegion, MAX_REGIONS>,
        grid: &mut dyn GridTarget,
    ) {
        let mut pos = Pos { x: 0, y: 0 };

        let base_style = if self.dark_mode {
            RenderStyle::dark()
        } else {
            RenderStyle::light()
        };

        grid.draw_box(0, 0, grid.cols(), grid.rows(), base_style.bg);

        self.render_node(0, &mut pos, dom, regions, base_style, grid);
    }

    fn render_text(
        &self,
        text: &str,
        pos: &mut Pos,
        style: RenderStyle,
        grid: &mut dyn GridTarget,
    ) {
        for ch in text.chars() {
            let ch = if style.uppercase {
                ch.to_ascii_uppercase()
            } else {
                ch
            };

            if ch == '\n' {
                pos.x = style.indent;
                pos.y += 1;
                continue;
            }

            if pos.x >= grid.cols() {
                pos.x = style.indent;
                pos.y += 1;
            }

            if pos.y >= grid.rows() {
                return;
            }

            grid.put_char(pos.x, pos.y, ch, style.fg, style.bg);
            pos.x += 1;
        }
    }

    fn render_children(
        &self,
        parent_id: NodeId,
        pos: &mut Pos,
        dom: &Dom,
        regions: &mut Vec<HitRegion, MAX_REGIONS>,
        style: RenderStyle,
        grid: &mut dyn GridTarget,
    ) {
        let mut child = dom.get_node(parent_id).first_child;

        while let Some(child_id) = child {
            self.render_node(child_id, pos, dom, regions, style, grid);
            child = dom.get_node(child_id).next_sibling;
        }
    }

    fn render_node(
        &self,
        node_id: NodeId,
        pos: &mut Pos,
        dom: &Dom,
        regions: &mut Vec<HitRegion, MAX_REGIONS>,
        style: RenderStyle,
        grid: &mut dyn GridTarget,
    ) {
        let node = dom.get_node(node_id);

        match &node.node_type {
            NodeType::Text(slice) => {
                let text = dom.resolve(*slice);
                self.render_text(text, pos, style, grid);
            }
            NodeType::Element(element) => {
                let tag = dom.resolve(element.tag_name);

                match tag {
                    "ROOT" => {
                        self.render_children(node_id, pos, dom, regions, style, grid);
                    }
                    "h1" => {
                        if pos.x != 0 {
                            pos.x = 0;
                            pos.y += 1;
                        }

                        let h1_style = RenderStyle {
                            fg: if self.dark_mode {
                                Rgb565::CSS_YELLOW
                            } else {
                                Rgb565::CSS_DARK_GOLDENROD
                            },
                            bg: style.bg,
                            uppercase: true,
                            indent: 0,
                        };

                        self.render_children(node_id, pos, dom, regions, h1_style, grid);

                        pos.x = 0;
                        pos.y += 2;
                    }
                    "p" => {
                        if pos.x != 0 {
                            pos.x = 0;
                            pos.y += 1;
                        }

                        self.render_children(node_id, pos, dom, regions, style, grid);

                        pos.x = 0;
                        pos.y += 1;
                    }
                    "a" => {
                        let link_style = RenderStyle {
                            fg: Rgb565::CSS_CYAN,
                            bg: style.bg,
                            uppercase: style.uppercase,
                            indent: style.indent,
                        };
                        let hit_target_start = pos.clone();
                        self.render_children(node_id, pos, dom, regions, link_style, grid);
                        let hit_target_end = pos.clone();

                        if let Some(href) = dom.get_attr_value(element, "href") {
                            // TODO: Handle edge case with too many regions.
                            regions
                                .push(HitRegion {
                                    rect: Rect {
                                        x_min: hit_target_start.x,
                                        y_min: hit_target_start.y,
                                        x_max: (hit_target_end.x).max(1),
                                        y_max: (hit_target_end.y + 1).max(1),
                                    },
                                    action: HitAction::Link { href },
                                })
                                .unwrap_or_default();
                        }
                    }
                    "br" => {
                        pos.x = 0;
                        pos.y += 1;
                    }
                    "b" | "strong" => {
                        let bold_style = RenderStyle {
                            fg: style.fg,
                            bg: style.bg,
                            uppercase: true,
                            indent: style.indent,
                        };
                        self.render_children(node_id, pos, dom, regions, bold_style, grid);
                    }
                    "div" => {
                        self.render_children(node_id, pos, dom, regions, style, grid);
                    }
                    "ul" => {
                        if pos.x != 0 {
                            pos.x = 0;
                            pos.y += 1;
                        }

                        let list_style = RenderStyle {
                            fg: style.fg,
                            bg: style.bg,
                            uppercase: style.uppercase,
                            indent: style.indent + 1,
                        };

                        self.render_children(node_id, pos, dom, regions, list_style, grid);
                    }
                    "li" => {
                        pos.x = style.indent;

                        self.render_text("- ", pos, style, grid);
                        self.render_children(node_id, pos, dom, regions, style, grid);

                        pos.x = 0;
                        pos.y += 1;
                    }
                    "blockquote" => {
                        if pos.x != 0 {
                            pos.x = 0;
                            pos.y += 1;
                        }

                        let quote_style = RenderStyle {
                            fg: Rgb565::CSS_LIME_GREEN,
                            bg: style.bg,
                            uppercase: style.uppercase,
                            indent: style.indent + 1,
                        };

                        self.render_text("> ", pos, quote_style, grid);
                        self.render_children(node_id, pos, dom, regions, quote_style, grid);

                        pos.x = 0;
                        pos.y += 1;
                    }
                    _ => {
                        self.render_children(node_id, pos, dom, regions, style, grid);
                    }
                }
            }
        }
    }
}
