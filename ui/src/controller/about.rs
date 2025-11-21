//! About dialog
// (c) 2025 Ross Younger

use easy_shader_runner::egui;

impl super::Controller {
    pub(crate) fn about_modal(&mut self, ctx: &egui::Context) {
        if egui::Modal::new("about".into())
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("About brot3").size(18.));

                ui.label(egui::RichText::new(crate::version_string("Version: ")).italics());

                ui.add_space(12.);
                ui.image(egui::include_image!("../../../icons/original,origin=-1.259742+0.377104i,axes=0.01+0.01i,max=512,col=lch-gradient.png"));
                ui.add_space(6.);

                ui.label(
                    egui::RichText::new("Dedicated to the memory of Benoît B. Mandelbrot.")
                );
                ui.add_space(12.);
                if ui.button("License").clicked() {
                    self.show_license = true;
                }
            })
            .should_close()
        {
            self.show_about = false;
        }
    }

    pub(crate) fn license_modal(&mut self, ctx: &egui::Context) {
        if egui::Modal::new("license".into())
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("brot3 terms of use").size(18.));
                ui.label(egui::RichText::new("The MIT License").size(12.));
                ui.label(
                    r"
Copyright (C) 2025 Ross Younger

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
            ",
                );
            })
            .should_close()
        {
            self.show_license = false;
        }
    }
}
