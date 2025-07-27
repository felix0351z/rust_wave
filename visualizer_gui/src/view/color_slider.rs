use egui::{lerp, pos2, remap_clamp, vec2, Color32, Mesh, Response, Sense, Shape, Stroke, TextEdit, Ui, WidgetInfo, WidgetType};
use std::ops::RangeInclusive;
use eframe::epaint::StrokeKind;

const N: u32 = 6 * 6 * 10;

pub fn color_slider(
    ui: &mut Ui,
    label: &str,
    value: &mut u16,
    range: RangeInclusive<u16>,
    color_at: impl Fn(f32) -> Color32,
) -> (Response, bool) {
    let mut changed = false;

    let response = ui.horizontal(|ui| {

        // Get the default size for a slider
        let desired_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
        // Allocate the space and react for click and drag events
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

        // 1. Handle the user input
        if let Some(pointer_position) = response.interact_pointer_pos() {
            // Update the pointer position with the current selected value
            let new_val = remap_clamp(
                pointer_position.x,
                rect.left()..=rect.right(),
                *range.start() as f32..=*range.end() as f32
            ).round() as u16;
            
            if new_val != *value {
                *value = new_val;
                changed = true;
            }
        }

        // 2. Provide widget information
        response.widget_info(|| WidgetInfo::selected(
            WidgetType::Slider,
            ui.is_enabled(),
            response.drag_started(),
            label
        ));

        // 3. Render the slider if visible
        if ui.is_rect_visible(rect) {

            let visuals = ui.style().interact(&response);

            // Draw the gradient mesh
            {
                let mut mesh = Mesh::default();
                for i in 0..=N {
                    // Between 0 and 1
                    let t = i as f32 / N as f32;

                    let color = color_at(t);
                    let x = lerp(rect.left()..=rect.right(), t);
                    let y_offset = ui.spacing().slider_rail_height / 2.0;
                    mesh.colored_vertex(pos2(x, rect.center().y + y_offset), color);
                    mesh.colored_vertex(pos2(x, rect.center().y - y_offset), color);

                    // Not finished
                    if i < N {
                        let idx = 2 * i;
                        mesh.add_triangle(idx, idx + 1, idx + 2);
                        mesh.add_triangle(idx + 1, idx + 2, idx + 3);
                    }
                }
                ui.painter().add(Shape::mesh(mesh));
            }

            // Draw the slider outline
            ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke, StrokeKind::Middle);

            // Render the slider handle
            {
                let x = lerp(
                    rect.left()..=rect.right(),
                    remap_clamp(
                        *value as f32,
                        *range.start() as f32..=*range.end() as f32,
                        0.0..=1.0
                    )
                );

                let radius = ui.spacing().slider_rail_height / 1.3;
                let picked_color = value_to_color(*value, &range, &color_at);

                ui.painter().circle(
                    pos2(x, rect.center().y),
                    radius,
                    picked_color,
                    Stroke::new(visuals.fg_stroke.width, picked_color)
                );
            }
        }

        let mut text = value.to_string();
        let text_response = ui.add(TextEdit::singleline(&mut text).desired_width(30.0));
        if text_response.changed() {
            if let Ok(v) = text.parse::<u16>() {
                *value = v.clamp(*range.start(), *range.end());
                changed = true
            }
        }

        response
    }).inner;

    (response, changed)
}

fn value_to_color(value: u16, range: &RangeInclusive<u16>, color_at: &impl Fn(f32) -> Color32) -> Color32 {
    color_at(remap_clamp(
        value as f32,
        *range.start() as f32..=*range.end() as f32,
        0.0..=1.0,
    ))
}