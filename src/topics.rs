use embedded_graphics::{
    geometry::{OriginDimensions, Point},
    image::GetPixel,
    mono_font::iso_8859_5::FONT_9X18_BOLD,
    pixelcolor::BinaryColor,
};

use crate::display::{Color, HEIGHT, Row, WIDTH, rgb};

const BACKGROUND: Color = rgb(8, 12, 17);
const GRID: Color = rgb(27, 39, 51);
const SURFACE: Color = rgb(20, 32, 45);
const TEXT: Color = rgb(236, 245, 255);
const HINT: Color = rgb(154, 176, 199);

struct Node {
    label: &'static str,
    x: i32,
    y: i32,
    parent: Option<usize>,
}

impl Node {
    const fn new(label: &'static str, x: i32, y: i32, parent: usize) -> Self {
        Self {
            label,
            x,
            y,
            parent: Some(parent),
        }
    }

    const fn root(label: &'static str) -> Self {
        Self {
            label,
            x: 0,
            y: 0,
            parent: None,
        }
    }
}

struct Tree {
    x: i32,
    accent: Color,
    link: Color,
    root_fill: Color,
    nodes: &'static [Node],
}

const TREES: &[Tree] = &[
    Tree {
        x: 0,
        accent: rgb(112, 223, 199),
        link: rgb(52, 146, 134),
        root_fill: rgb(19, 57, 56),
        nodes: &[
            Node::root("Геометрия"),
            Node::new("Планиметрия", -156, 105, 0),
            Node::new("Стереометрия", 0, 105, 0),
            Node::new("Координаты", 156, 105, 0),
            Node::new("Треугольники", -232, 214, 1),
            Node::new("Окружности", -103, 214, 1),
            Node::new("Многогранники", 34, 214, 2),
            Node::new("Векторы", 174, 214, 3),
            Node::new("Теорема Пифагора", -232, 320, 4),
            Node::new("Объёмы", 34, 320, 6),
        ],
    },
    Tree {
        x: 620,
        accent: rgb(255, 192, 116),
        link: rgb(177, 117, 65),
        root_fill: rgb(70, 45, 29),
        nodes: &[
            Node::root("Алгебра"),
            Node::new("Уравнения", -156, 105, 0),
            Node::new("Функции", 0, 105, 0),
            Node::new("Числа", 156, 105, 0),
            Node::new("Линейные", -223, 214, 1),
            Node::new("Квадратные", -97, 214, 1),
            Node::new("Графики", 30, 214, 2),
            Node::new("Дроби", 155, 214, 3),
            Node::new("Дискриминант", -97, 320, 5),
        ],
    },
];

pub struct Topics {
    offset: Point,
    previous_touch: Option<Point>,
    show_hint: bool,
}

impl Topics {
    pub const fn new() -> Self {
        Self {
            offset: Point::new(WIDTH / 2, 122),
            previous_touch: None,
            show_hint: true,
        }
    }

    pub fn drag(&mut self, position: Option<Point>) -> bool {
        let previous = core::mem::replace(&mut self.previous_touch, position);
        let (Some(previous), Some(position)) = (previous, position) else {
            return false;
        };
        let mut next = self.offset + position - previous;
        // Keep every outer node reachable at the center of the round screen.
        next.x = next.x.clamp(WIDTH / 2 - (620 + 156), WIDTH / 2 + 232);
        next.y = next.y.clamp(HEIGHT / 2 - 320, HEIGHT / 2);
        if next == self.offset {
            return false;
        }
        self.offset = next;
        self.show_hint = false;
        true
    }

    pub fn draw_row(&self, y: i32, row: &mut Row<'_>) {
        row.fill(BACKGROUND);
        let map_y = y - self.offset.y;
        if map_y.rem_euclid(24) == 0 {
            for x in (self.offset.x.rem_euclid(24)..WIDTH).step_by(24) {
                row.span(x, x + 1, GRID);
            }
        }

        for tree in TREES {
            let x_offset = tree.x + self.offset.x;
            for child in tree.nodes.iter().filter(|node| node.parent.is_some()) {
                let parent = &tree.nodes[child.parent.unwrap()];
                draw_link(row, map_y, x_offset, parent, child, tree.link);
            }
            for node in tree.nodes {
                draw_node(row, map_y, x_offset, node, tree);
            }
        }

        if self.show_hint && (397..435).contains(&y) {
            row.span(140, 326, SURFACE);
            label_row(row, y - 407, WIDTH / 2, "Перетащи карту", HINT);
        }
    }
}

fn draw_link(row: &mut Row<'_>, y: i32, offset: i32, parent: &Node, child: &Node, color: Color) {
    let middle = (parent.y + child.y) / 2;
    let from = parent.x + offset;
    let to = child.x + offset;
    if (parent.y..middle).contains(&y) {
        row.span(from - 1, from + 1, color);
    } else if (middle..middle + 2).contains(&y) {
        row.span(from.min(to) - 1, from.max(to) + 1, color);
    } else if (middle..child.y).contains(&y) {
        row.span(to - 1, to + 1, color);
    }
}

fn draw_node(row: &mut Row<'_>, y: i32, offset: i32, node: &Node, tree: &Tree) {
    let root = node.parent.is_none();
    let half_height = if root { 24 } else { 19 };
    let distance_from_edge = half_height - (y - node.y).abs();
    if distance_from_edge <= 0 {
        return;
    }

    let center = offset + node.x;
    let width = (node.label.chars().count() as i32 * 9 + 24).max(if root { 148 } else { 104 });
    let inset = (4 - distance_from_edge).max(0);
    let left = center - width / 2 + inset;
    let right = center + width / 2 - inset;
    if right <= 0 || left >= WIDTH {
        return;
    }

    row.span(left, right, if root { tree.accent } else { tree.link });
    if distance_from_edge >= 3 {
        row.span(
            left + 2,
            right - 2,
            if root { tree.root_fill } else { SURFACE },
        );
    }
    label_row(row, y - node.y + 9, center, node.label, TEXT);
}

fn label_row(row: &mut Row<'_>, y: i32, center: i32, text: &str, color: Color) {
    let font = &FONT_9X18_BOLD;
    let glyph_width = font.character_size.width as i32;
    let glyph_height = font.character_size.height as i32;
    if !(0..glyph_height).contains(&y) {
        return;
    }

    let pitch = glyph_width + font.character_spacing as i32;
    let columns = (font.image.size().width / font.character_size.width) as i32;
    let mut left = center - (text.chars().count() as i32 * pitch) / 2;
    for character in text.chars() {
        if left < WIDTH && left + glyph_width > 0 {
            let index = font.glyph_mapping.index(character) as i32;
            let glyph_x = (index % columns) * glyph_width;
            let glyph_y = (index / columns) * glyph_height;
            for x in 0..glyph_width {
                if font.image.pixel(Point::new(glyph_x + x, glyph_y + y)) == Some(BinaryColor::On) {
                    row.span(left + x, left + x + 1, color);
                }
            }
        }
        left += pitch;
    }
}
