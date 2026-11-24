use embedded_graphics::geometry::Point;

use crate::display::{Color, HEIGHT, Row, WIDTH, rgb};

const BACKGROUND: Color = rgb(20, 14, 25);
const GRID: Color = rgb(68, 46, 60);
const SURFACE: Color = rgb(54, 32, 49);
const TEXT: Color = rgb(255, 242, 214);
const HINT: Color = rgb(221, 197, 174);

struct Node {
    label: &'static str,
    x: i32,
    y: i32,
    parent: Option<usize>,
    note: &'static str,
}

impl Node {
    fn width(&self) -> i32 {
        (self.label.chars().count() as i32 * 10 + 52).max(218)
    }

    const fn topic(label: &'static str, x: i32, note: &'static str) -> Self {
        Self {
            label,
            x,
            y: 96,
            parent: Some(0),
            note,
        }
    }

    const fn root(label: &'static str) -> Self {
        Self {
            label,
            x: 0,
            y: 0,
            parent: None,
            note: "",
        }
    }
}

struct Tree {
    y: i32,
    accent: Color,
    link: Color,
    root_fill: Color,
    nodes: &'static [Node],
}

// Grade-specific sources: MCKO maths, profile grade 11 (September 2023)
// and advanced grade 10 (April-May 2026), content tables 1 and 2.
const TREES: &[Tree] = &[
    Tree {
        y: 0,
        accent: rgb(238, 202, 151),
        link: rgb(173, 125, 91),
        root_fill: rgb(83, 39, 56),
        nodes: &[
            Node::root("МЦКО: 11 класс"),
            Node::topic("Планиметрия", -816, "Площадь по углу\nS = ab sin(C)/2"),
            Node::topic(
                "Стереометрия",
                -544,
                "Прямые и плоскости\nУгол с проекцией\nПризма: V = Sосн*h\nПирамида: V = Sосн*h/3",
            ),
            Node::topic(
                "Уравнения",
                -272,
                "Показательные и с корнем\nЛогарифмические: ОДЗ\nТригонометрические\nСистемы и задачи",
            ),
            Node::topic(
                "Производная",
                0,
                "Касательная: k = f'(x)\nЗнак плюс: рост\nЗнак минус: убывание",
            ),
            Node::topic(
                "Функции",
                272,
                "Значение по графику\nМаксимум и минимум\nЧисло решений:\nпересечения графиков",
            ),
            Node::topic(
                "Выражения",
                544,
                "Степени и корни\nПри a>0, b>0:\nlg(ab) = lg(a)+lg(b)",
            ),
            Node::topic(
                "Вероятность",
                816,
                "Совместный исход\nпри независимости:\nP(A и B) = P(A)*P(B)",
            ),
        ],
    },
    Tree {
        y: 480,
        accent: rgb(229, 178, 204),
        link: rgb(158, 106, 142),
        root_fill: rgb(64, 35, 65),
        nodes: &[
            Node::root("МЦКО: 10 класс"),
            Node::topic(
                "Множества",
                -816,
                "Объединение: хотя бы одно\nПересечение: оба\nДиаграммы Эйлера",
            ),
            Node::topic(
                "Геометрия",
                -544,
                "Расстояния и углы\nПрямые и плоскости\nПлощади и объёмы",
            ),
            Node::topic(
                "Вычисления",
                -272,
                "Степени и корни\nПрикидка и округление\nСтандартный вид числа",
            ),
            Node::topic(
                "Прогрессии",
                0,
                "a(n) = a1 + (n-1)d\nb(n) = b1*q^(n-1)\nПри |q|<1: S = b1/(1-q)",
            ),
            Node::topic(
                "Уравнения",
                272,
                "Рациональные и с корнем\nНеравенства: интервалы\nsin(x), cos(x), tg(x)\nУчитывай период",
            ),
            Node::topic(
                "Графики",
                544,
                "Степенные функции\nРешить f(x)=g(x):\nнайти абсциссы\nточек пересечения",
            ),
            Node::topic(
                "Случайные события",
                816,
                "Исходы и комбинаторика\nУсловная вероятность\nНезависимые испытания\nРаспределения",
            ),
        ],
    },
];

pub struct Topics {
    offset: Point,
    previous_touch: Option<Point>,
    origin: Option<Point>,
    dragging: bool,
    show_hint: bool,
    selected: Option<(&'static Tree, &'static Node)>,
}

impl Topics {
    pub const fn new() -> Self {
        Self {
            offset: Point::new(WIDTH / 2, 122),
            previous_touch: None,
            origin: None,
            dragging: false,
            show_hint: true,
            selected: None,
        }
    }

    pub fn cancel_contact(&mut self) {
        self.previous_touch = None;
        self.origin = None;
        self.dragging = false;
    }
    pub fn close_note(&mut self) {
        self.selected = None;
    }

    pub fn touch(&mut self, position: Option<Point>) -> bool {
        let Some(point) = position else {
            let tap = self.origin.filter(|_| !self.dragging);
            self.cancel_contact();
            return tap.is_some_and(|point| self.activate(point));
        };
        let origin = *self.origin.get_or_insert(point);
        let previous = self.previous_touch.replace(point).unwrap_or(point);
        let delta = point - origin;
        if !self.dragging && delta.x.abs().max(delta.y.abs()) <= 24 {
            return false;
        }
        let from = if self.dragging { previous } else { origin };
        self.dragging = true;
        if self.selected.is_some() {
            return false;
        }
        let mut next = self.offset + point - from;
        // Keep every outer node reachable at the center of the round screen.
        next.x = next.x.clamp(WIDTH / 2 - 816, WIDTH / 2 + 816);
        next.y = next.y.clamp(HEIGHT / 2 - 576, HEIGHT / 2);
        if next == self.offset {
            return false;
        }
        self.offset = next;
        self.show_hint = false;
        true
    }

    fn activate(&mut self, point: Point) -> bool {
        if self.selected.is_some() {
            if (84..382).contains(&point.x) && (302..406).contains(&point.y) {
                self.close_note();
                return true;
            }
        } else {
            for tree in TREES {
                for node in &tree.nodes[1..] {
                    let dx = point.x - self.offset.x - node.x;
                    let dy = point.y - self.offset.y - tree.y - node.y;
                    if dx.abs() <= node.width() / 2 + 12 && dy.abs() <= 48 {
                        self.selected = Some((tree, node));
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn draw_row(&self, y: i32, row: &mut Row<'_>) {
        row.fill(BACKGROUND);
        row.arch(y);
        if let Some((tree, node)) = self.selected {
            let center = WIDTH / 2;
            row.label(y - 91, center, tree.nodes[0].label, tree.accent);
            row.label(y - 137, center, node.label, TEXT);
            for (i, line) in node.note.lines().enumerate() {
                row.label(y - 195 - 25 * i as i32, center, line, TEXT);
            }
            if (314..394).contains(&y) {
                let bevel = (7 - (y - 314).min(393 - y)).max(0);
                row.span(96 + bevel, 370 - bevel, tree.accent);
                if (316..392).contains(&y) {
                    row.span(99 + bevel, 367 - bevel, tree.root_fill);
                }
                row.label(y - 345, center, "К карте", TEXT);
            }
            return;
        }
        let map_y = y - self.offset.y;
        if map_y.rem_euclid(24) == 0 {
            for x in (self.offset.x.rem_euclid(24)..WIDTH).step_by(24) {
                row.span(x, x + 1, GRID);
            }
        }

        for tree in TREES {
            let x_offset = self.offset.x;
            let map_y = map_y - tree.y;
            if !(-24..=265).contains(&map_y) {
                continue;
            }
            for child in tree.nodes.iter().filter(|node| node.parent.is_some()) {
                let parent = &tree.nodes[child.parent.unwrap()];
                draw_link(row, map_y, x_offset, parent, child, tree.link);
            }
            for node in tree.nodes {
                draw_node(row, map_y, x_offset, node, tree);
            }
        }

        if self.show_hint && (355..411).contains(&y) {
            row.span(104, 362, SURFACE);
            row.label(y - 361, WIDTH / 2, "Нажми на тему", TEXT);
            row.label(y - 384, WIDTH / 2, "Ветви по бокам, 10 кл. ниже", HINT);
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
    let half_height = if root { 24 } else { 37 };
    let distance_from_edge = half_height - (y - node.y).abs();
    if distance_from_edge <= 0 {
        return;
    }

    let center = offset + node.x;
    let width = node.width();
    let inset = (10 - distance_from_edge).max(0);
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
    row.label(y - node.y + 9, center, node.label, TEXT);
}
