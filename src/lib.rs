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
        _ => text.to_lowercase(),
    }
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
