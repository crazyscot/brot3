//! A menu item (Button-like) that is also a checkbox
//!
//! This is basically a welding-together of the relevant parts of checkbox.rs and button.rs

use easy_shader_runner::egui::{
    epaint, pos2, Atom, AtomKind, AtomLayout, Frame, Id, IntoAtoms, NumExt as _, Response, Sense,
    Shape, TextStyle, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

/*
This is basically a Button
- whose contents are a checkbox
- with accelerator text
 */

pub struct CheckableButton<'a> {
    checked: &'a mut bool,
    atoms: AtomLayout<'a>,
    min_size: Vec2,
}

impl<'a> CheckableButton<'a> {
    pub fn new(checked: &'a mut bool, label: impl IntoAtoms<'a>) -> Self {
        let mut cb = CheckableButton {
            checked,
            atoms: AtomLayout::new(label.into_atoms())
                .sense(Sense::click())
                .fallback_font(TextStyle::Button),
            min_size: Vec2::ZERO,
        };
        cb.atoms.push_right(Atom::grow());
        cb
    }
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
    pub fn shortcut_text(mut self, shortcut_text: impl Into<Atom<'a>>) -> Self {
        let mut atom = shortcut_text.into();
        atom.kind = match atom.kind {
            AtomKind::Text(text) => AtomKind::Text(text.weak()),
            other => other,
        };
        self.atoms.push_right(Atom::grow());
        self.atoms.push_right(atom);
        self
    }
}

impl<'a> Widget for CheckableButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let CheckableButton {
            checked,
            mut atoms,
            mut min_size,
        } = self;

        let spacing = &ui.spacing();
        let icon_width = spacing.icon_width;
        min_size.x = min_size.x.at_least(spacing.interact_size.y);
        min_size.y = min_size.y.at_least(spacing.interact_size.y);
        min_size.y = min_size.y.at_least(icon_width);

        let mut icon_size = Vec2::splat(icon_width);
        icon_size.y = icon_size.y.at_least(min_size.y);
        let rect_id = Id::new("crazyscot::checkablebutton");
        atoms.push_left(Atom::custom(rect_id, icon_size));

        let text = atoms.text().map(String::from);

        let button_padding = ui.spacing().button_padding;

        let mut prepared = atoms
            .frame(Frame::new().inner_margin(button_padding))
            .min_size(min_size)
            .allocate(ui);

        if prepared.response.clicked() {
            *checked = !*checked;
            prepared.response.mark_changed();
        }

        prepared.response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::Checkbox,
                ui.is_enabled(),
                *checked,
                text.as_deref().unwrap_or(""),
            )
        });

        if ui.is_rect_visible(prepared.response.rect) {
            let visuals = ui.style().interact_selectable(&prepared.response, *checked);
            prepared.fallback_text_color = visuals.text_color();

            let visible_frame = prepared.response.hovered()
                || prepared.response.is_pointer_button_down_on()
                || prepared.response.has_focus();

            if visible_frame {
                let stroke = visuals.bg_stroke;
                let fill = visuals.weak_bg_fill;
                prepared.frame = prepared
                    .frame
                    .inner_margin(
                        button_padding + Vec2::splat(visuals.expansion) - Vec2::splat(stroke.width),
                    )
                    .outer_margin(-Vec2::splat(visuals.expansion))
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(visuals.corner_radius);
            }

            let response = prepared.paint(ui);

            if let Some(rect) = response.rect(rect_id) {
                let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
                ui.painter().add(epaint::RectShape::new(
                    big_icon_rect.expand(visuals.expansion),
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    epaint::StrokeKind::Inside,
                ));

                if *checked {
                    // Check mark:
                    ui.painter().add(Shape::line(
                        vec![
                            pos2(small_icon_rect.left(), small_icon_rect.center().y),
                            pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                            pos2(small_icon_rect.right(), small_icon_rect.top()),
                        ],
                        visuals.fg_stroke,
                    ));
                }
            }
            response.response
        } else {
            prepared.response
        }
    }
}
