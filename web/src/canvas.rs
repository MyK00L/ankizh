use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

type Point = (f64, f64);

struct Stroke {
    p: Vec<Point>,
}
impl Stroke {
    fn push(&mut self, p: Point) {
        self.p.push(p);
    }
    fn to_hex(&self) -> String {
        self.p
            .iter()
            .map(|x| {
                format!(
                    "{:x}{:x}",
                    u64::from_be_bytes(x.0.to_be_bytes()),
                    u64::from_be_bytes(x.1.to_be_bytes())
                )
            })
            .fold(String::new(), |a, b| a + &b)
    }
    fn from_hex(s: &str) -> Self {
        let mut p = Vec::with_capacity(s.len() / 32);
        for i in (0..p.len()).step_by(32) {
            let xu = u64::from_str_radix(&s[i..(i + 16)], 16).unwrap();
            let yu = u64::from_str_radix(&s[(i + 16)..(i + 32)], 16).unwrap();

            let x = f64::from_be_bytes(xu.to_be_bytes());
            let y = f64::from_be_bytes(yu.to_be_bytes());

            p.push((x, y));
        }
        Self { p }
    }
}

struct DrawState {
    s: Vec<Stroke>,
}
impl DrawState {
    fn new() -> Self {
        Self { s: vec![] }
    }
    fn to_hex(&self) -> String {
        let mut ans = self
            .s
            .iter()
            .map(|x| x.to_hex())
            .fold(String::new(), |a, b| a + &b + "|");
        ans.pop();
        ans
    }
    fn from_hex(s: &str) -> Self {
        let s: Vec<Stroke> = s.split('|').map(Stroke::from_hex).collect();
        Self { s }
    }
    fn start_new_stroke(&mut self, p: Point) {
        self.s.push(Stroke { p: vec![p] });
    }
    fn continue_stroke(&mut self, p: Point) {
        self.s.last_mut().unwrap().push(p);
    }
    fn undo_stroke(&mut self) {
        self.s.pop();
    }
    fn clear(&mut self) {
        self.s.clear();
    }
}

struct MultiDrawState {
    s: Vec<DrawState>,
}
impl MultiDrawState {
    fn new() -> Self {
        Self {
            s: vec![DrawState::new()],
        }
    }
    fn to_hex(&self) -> String {
        let mut ans = self
            .s
            .iter()
            .map(|x| x.to_hex())
            .fold(String::new(), |a, b| a + &b + ";");
        ans.pop();
        ans
    }
    fn from_hex(s: &str) -> Self {
        let s: Vec<DrawState> = s.split(';').map(DrawState::from_hex).collect();
        Self { s }
    }
    fn start_new_stroke(&mut self, p: Point) {
        self.s.last_mut().unwrap().start_new_stroke(p);
    }
    fn continue_stroke(&mut self, p: Point) {
        self.s.last_mut().unwrap().continue_stroke(p);
    }
    fn undo_stroke(&mut self) {
        self.s.last_mut().unwrap().undo_stroke();
    }
    fn clear(&mut self) {
        self.s.last_mut().unwrap().clear();
    }
    fn add(&mut self) {
        self.s.push(DrawState::new());
    }
    fn rem(&mut self) {
        if self.s.len() == 1 {
            self.clear();
        } else {
            self.s.pop();
        }
    }
}

fn point_from_event(c: &web_sys::HtmlCanvasElement, e: &MouseEvent) -> Point {
    (
        (e.client_x() - c.offset_left()) as f64,
        (e.client_y() - c.offset_top()) as f64,
    )
}

pub struct CanvasManager {
    s: MultiDrawState,
    canvas: web_sys::HtmlCanvasElement,
    ctx: web_sys::CanvasRenderingContext2d,
    is_down: bool,
}
//⎌
//⌧
//+
//-
//✓
use crate::utils::add_callback;
impl CanvasManager {
    pub fn new(size: u32) -> (Rc<RefCell<Self>>, web_sys::Element) {
        let dom = web_sys::window().unwrap().document().unwrap();
        let canvas = dom
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas.set_height(size);
        canvas.set_width(size);
        let canvas_et = canvas.clone().dyn_into::<web_sys::EventTarget>().unwrap();
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let button_div = dom.create_element("div").unwrap();
        button_div.class_list().add_1("btn-group").unwrap();
        let button_clear = dom.create_element("button").unwrap();
        button_clear
            .append_child(&dom.create_text_node("⌧"))
            .unwrap();
        button_clear.class_list().add_1("control-button").unwrap();
        let button_undo = dom.create_element("button").unwrap();
        button_undo
            .append_child(&dom.create_text_node("⎌"))
            .unwrap();
        button_undo.class_list().add_1("control-button").unwrap();
        let button_add = dom.create_element("button").unwrap();
        button_add.append_child(&dom.create_text_node("+")).unwrap();
        button_add.class_list().add_1("control-button").unwrap();
        let button_rem = dom.create_element("button").unwrap();
        button_rem.append_child(&dom.create_text_node("-")).unwrap();
        button_rem.class_list().add_1("control-button").unwrap();
        button_div.append_child(&button_clear).unwrap();
        button_div.append_child(&button_undo).unwrap();
        button_div.append_child(&button_add).unwrap();
        button_div.append_child(&button_rem).unwrap();

        let div = dom.create_element("div").unwrap();
        div.append_child(&canvas).unwrap();
        div.append_child(&dom.create_element("br").unwrap())
            .unwrap();
        div.append_child(&button_div).unwrap();

        let s = MultiDrawState::new();
        let res = Rc::new(RefCell::new(Self {
            s,
            canvas,
            ctx,
            is_down: false,
        }));

        add_callback!("click", button_clear, res, || {
            res.borrow_mut().clear();
        });
        add_callback!("click", button_undo, res, || {
            res.borrow_mut().undo_stroke();
        });
        add_callback!("click", button_add, res, || {
            res.borrow_mut().add();
        });
        add_callback!("click", button_rem, res, || {
            res.borrow_mut().rem();
        });

        add_callback!(
            "mousedown",
            "touchstart",
            canvas_et,
            res,
            |me: MouseEvent| {
                me.prevent_default();
                let p = point_from_event(&res.borrow().canvas, &me);
                res.borrow_mut().mouse_down(p);
            }
        );
        add_callback!(
            "mousemove",
            "touchmove",
            canvas_et,
            res,
            |me: MouseEvent| {
                me.prevent_default();
                let p = point_from_event(&res.borrow().canvas, &me);
                res.borrow_mut().mouse_move(p);
            }
        );
        add_callback!(
            "mouseup",
            "mouseleave",
            "touchend",
            "touchcancel",
            canvas_et,
            res,
            |me: MouseEvent| {
                me.prevent_default();
                res.borrow_mut().mouse_up();
            }
        );

        (res, div)
    }
    pub fn get_normalized_strokes(&self) -> Vec<Vec<(f64, f64)>> {
        let mul = 1.0f64 / self.canvas.height() as f64;
        self.s
            .s
            .last()
            .unwrap()
            .s
            .iter()
            .map(|s| s.p.iter().map(|p| (p.0 * mul, p.1 * mul)).collect())
            .collect()
    }
    fn mouse_down(&mut self, p: Point) {
        self.is_down = true;
        self.s.start_new_stroke(p);
        self.ctx.begin_path();
        self.ctx.move_to(p.0, p.1);
    }
    fn mouse_move(&mut self, p: Point) {
        if self.is_down {
            self.s.continue_stroke(p);
            self.ctx.line_to(p.0, p.1);
            self.ctx.stroke();
        }
    }
    fn mouse_up(&mut self) {
        self.is_down = false;
    }
    fn undo_stroke(&mut self) {
        self.s.undo_stroke();
        self.redraw();
    }
    fn clear(&mut self) {
        self.s.clear();
        self.redraw();
    }
    fn add(&mut self) {
        self.s.add();
        self.redraw();
    }
    fn rem(&mut self) {
        self.s.rem();
        self.redraw();
    }
    pub fn redraw(&self) {
        self.ctx.reset();
        for s in self.s.s.last().unwrap().s.iter() {
            let mut i = s.p.iter();
            if let Some(p) = i.next() {
                self.ctx.begin_path();
                self.ctx.move_to(p.0, p.1);
                for p in i {
                    self.ctx.line_to(p.0, p.1);
                }
                self.ctx.stroke();
            }
        }
    }
}
