use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use std::path::PathBuf;

use liquid_core::{
    Display_filter, Filter, FilterReflection, ParseFilter, Result, Runtime, Value, ValueView,
};

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "topath",
    description = "Convert a string to something we can use as a path in the URL",
    parsed(ToPathFilter)
)]
pub struct ToPath;

#[derive(Debug, Default, Display_filter)]
#[name = "topath"]
pub struct ToPathFilter;

impl Filter for ToPathFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        let text = input.to_kstr();
        Ok(Value::scalar(topath(&text)))
    }
}

pub fn topath(text: &str) -> String {
    match text {
        "#" => "number-sign".to_string(),
        "/" => "forward-slash".to_string(),
        "\\" => "back-slash".to_string(),
        "." => "full-stop".to_string(),
        ";" => "semi-colon".to_string(),
        ":" => "colon".to_string(),
        "'" => "single-quote".to_string(),
        "\"" => "double-quote".to_string(),
        _ => text.replace(' ', "_").to_lowercase(),
    }
}

pub fn draw_image(path: &PathBuf, text: &str) -> bool {
    let width = 1000;
    let height = 500;

    let limit = 40;
    if text.len() > limit {
        return false;
    }

    // create image
    let mut image = RgbImage::new(width, height);
    // set white background
    for x in 0..width {
        for y in 0..height {
            *image.get_pixel_mut(x, y) = image::Rgb([255, 255, 255]);
        }
    }

    //let font = Vec::from(include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf") as &[u8]);
    //let font = Vec::from(include_bytes!("/snap/cups/980/usr/share/fonts/truetype/freefont/FreeSans.ttf") as &[u8]);
    let font = Vec::from(include_bytes!("../FreeSans.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let inteded_text_height = 24.4;
    let scale = Scale {
        x: inteded_text_height * 2.0,
        y: inteded_text_height,
    };

    // color of the text
    let red = 50_u8;
    let green = 50;
    let blue = 0;

    // get the size of the text and calculate the x, y coordinate where to start to be center aligned
    // both horizontally and vertically
    let (text_width, text_height) = text_size(scale, &font, text);
    println!("Text size: {}x{}", text_width, text_height);
    let text_start_x = ((width - text_width as u32) / 2) as i32;
    let text_start_y = ((height - text_height as u32) / 2) as i32;

    draw_text_mut(
        &mut image,
        Rgb([red, green, blue]),
        text_start_x,
        text_start_y,
        scale,
        &font,
        text,
    );

    image.save(path).unwrap();

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_topath() {
        let cases = vec![("hello", "hello"), ("#", "number-sign")];

        for entry in cases {
            let text = "{{ text | topath}}";
            let globals = liquid::object!({
                "text": entry.0,
            });
            let template = liquid::ParserBuilder::with_stdlib()
                .filter(ToPath)
                .build()
                .unwrap()
                .parse(text)
                .unwrap();
            let output = template.render(&globals).unwrap();
            assert_eq!(output, entry.1.to_string());
        }
    }
}
