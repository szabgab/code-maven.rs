use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use liquid_core::error::ResultLiquidReplaceExt as _;
use liquid_core::model::Scalar;
use liquid_core::parser::TryMatchToken;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::ValueView as _;
use liquid_core::{ParseTag, TagReflection, TagTokenIter};

use crate::read_languages;

#[derive(Copy, Clone, Debug, Default)]
pub struct IncludeTag;

#[allow(clippy::missing_trait_methods)]
impl TagReflection for IncludeTag {
    fn tag(&self) -> &'static str {
        "include"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for IncludeTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        //println!("Parsing include tag");
        arguments
            .expect_next("\"file\" expected.")?
            .expect_str("file")
            .into_result_custom_msg("\"file\" expected.")?;

        arguments
            .expect_next("Assignment operator \"=\" expected.")?
            .expect_str("=")
            .into_result_custom_msg("Assignment operator \"=\" expected.")?;

        let token = arguments.expect_next("Identifier or value expected")?;
        let file = match token.expect_literal() {
            TryMatchToken::Matches(name) => name.to_kstr().into_string(),
            TryMatchToken::Fails(name) => return name.raise_error().into_err(),
        };

        arguments.expect_nothing()?;
        //println!("Parsing done");

        Ok(Box::new(Include { file }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Include {
    file: String,
}

#[allow(clippy::missing_trait_methods)]
impl Renderable for Include {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let ext_to_language: HashMap<String, String> = read_languages();
        //println!("render_to");

        let root = match runtime.get(&[Scalar::new("root")]) {
            Ok(value) => value.to_kstr().into_string(),
            Err(_) => {
                return Err(liquid_core::error::Error::with_msg(
                    "No value called 'root' was passed to the render function.",
                ));
            }
        };

        let repo = match runtime.get(&[Scalar::new("repo")]) {
            Ok(value) => value.to_kstr().into_string(),
            Err(_) => {
                return Err(liquid_core::error::Error::with_msg(
                    "No value called 'repo' was passed to the render function.",
                ));
            }
        };

        let branch = match runtime.get(&[Scalar::new("branch")]) {
            Ok(value) => value.to_kstr().into_string(),
            Err(_) => {
                return Err(liquid_core::error::Error::with_msg(
                    "No value called 'branch' was passed to the render function.",
                ));
            }
        };

        let path = Path::new(&self.file);
        let include_path = Path::new(&root).join(path);

        let file_name = path
            .file_name()
            .ok_or(liquid_core::error::Error::with_msg(format!(
                "file_name not found in include tag '{}'",
                self.file
            )))?
            .to_str()
            .ok_or(liquid_core::error::Error::with_msg(format!(
                "file_name could not convert to string in include tag '{}'",
                self.file
            )))?;

        // TODO remove the hard coded mapping of .gitignore
        // TODO properly handle files that do not have an extension
        let language = if file_name == ".gitignore" {
            "gitignore"
        } else {
            let extension = path
                .extension()
                .ok_or(liquid_core::error::Error::with_msg(format!(
                    "extension not found in include tag '{}'",
                    self.file
                )))?
                .to_str()
                .ok_or(liquid_core::error::Error::with_msg(format!(
                    "extension could not convert to string in include tag {}",
                    self.file
                )))?;

            //println!("extension: {extension}");

            if ext_to_language.contains_key(extension) {
                ext_to_language[extension].as_str()
            } else {
                return Err(liquid_core::error::Error::with_msg(format!(
                    "Unhandled extension '{extension}' in {}",
                    self.file
                )));
            }
        };

        let file_content = std::fs::read_to_string(&include_path)
            .replace(format!("Failed to read file {include_path:?}"))?;
        write!(
            writer,
            "**[{}]({}/tree/{}/{})**\n```{}\n{}\n```\n",
            path.display(),
            &repo,
            &branch,
            path.display(),
            language,
            &file_content
        )
        .replace("Failed to render")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_core::parser;
    use liquid_core::runtime;
    use liquid_core::runtime::RuntimeBuilder;
    use liquid_core::Value;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("include".to_string(), IncludeTag.into());
        options
    }

    #[test]
    fn include_file() {
        let options = options();
        let template = parser::parse(
            r#"{% include file = "test_cases/demo/examples/demo.yaml" %}"#,
            &options,
        )
        .map(runtime::Template::new)
        .unwrap();

        let runtime = RuntimeBuilder::new().build();
        runtime.set_global("root".into(), Value::scalar("."));
        runtime.set_global(
            "repo".into(),
            Value::scalar("https://github.com/szabgab/code-maven.rs/"),
        );
        runtime.set_global("branch".into(), Value::scalar("main"));

        let output = template.render(&runtime).unwrap();
        assert_eq!(output, "**[test_cases/demo/examples/demo.yaml](https://github.com/szabgab/code-maven.rs//tree/main/test_cases/demo/examples/demo.yaml)**\n```yaml\nfield: value\n\n```\n");
    }

    #[test]
    fn missing_include_file() {
        let options = options();
        let template = parser::parse(r#"{% include file = "test_cases/other.txt" %}"#, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();
        runtime.set_global("root".into(), Value::scalar("."));
        runtime.set_global(
            "repo".into(),
            Value::scalar("https://github.com/szabgab/code-maven.rs/"),
        );
        runtime.set_global("branch".into(), Value::scalar("main"));

        let result = template.render(&runtime);
        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(
            result.to_string(),
            "liquid: Failed to read file \"./test_cases/other.txt\"\n"
        );
    }

    #[test]
    fn include_file_invalid_extension() {
        let options = options();
        let template = parser::parse(r#"{% include file = "test_cases/other.qqrq" %}"#, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();
        runtime.set_global("root".into(), Value::scalar("."));
        runtime.set_global(
            "repo".into(),
            Value::scalar("https://github.com/szabgab/code-maven.rs/"),
        );
        runtime.set_global("branch".into(), Value::scalar("main"));

        let result = template.render(&runtime);
        assert!(result.is_err());
        let result = result.err().unwrap();
        assert_eq!(
            result.to_string(),
            "liquid: Unhandled extension 'qqrq' in test_cases/other.qqrq\n"
        );
    }
}
