use std::io::Write;

use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::parser::TryMatchToken;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::ValueView;
use liquid_core::{ParseTag, TagReflection, TagTokenIter};

#[derive(Copy, Clone, Debug, Default)]
pub struct YoutubeTag;

#[allow(clippy::missing_trait_methods)]
impl TagReflection for YoutubeTag {
    fn tag(&self) -> &'static str {
        "youtube"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for YoutubeTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        arguments
            .expect_next("id expected")?
            .expect_str("id")
            .into_result_custom_msg("id expected.")?;

        arguments
            .expect_next("Assignment operator \"=\" expected.")?
            .expect_str("=")
            .into_result_custom_msg("Assignment operator \"=\" expected.")?;

        let id_token = arguments.expect_next("Identifier or value expected")?;
        let id = match id_token.expect_literal() {
            TryMatchToken::Matches(name) => name.to_kstr().to_string(),
            TryMatchToken::Fails(name) => return name.raise_error().into_err(),
        };

        let key = arguments.next();

        let file = match key {
            Some(file_token) => {
                file_token
                    .expect_str("file")
                    .into_result_custom_msg("expected file")?;
                arguments
                    .expect_next("Assignment operator \"=\" expected.")?
                    .expect_str("=")
                    .into_result_custom_msg("Assignment operator \"=\" expected.")?;

                let literal = arguments.expect_next("value of tag")?.expect_literal();

                let file_value = match literal {
                    TryMatchToken::Matches(name) => name.to_kstr().to_string(),
                    TryMatchToken::Fails(name) => return name.raise_error().into_err(),
                };
                Some(file_value)
            }
            None => None,
        };

        arguments.expect_nothing()?;

        Ok(Box::new(YouTube { id, file }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct YouTube {
    id: String,
    file: Option<String>,
}

#[allow(clippy::missing_trait_methods)]
impl Renderable for YouTube {
    fn render_to(&self, writer: &mut dyn Write, _runtime: &dyn Runtime) -> Result<()> {
        let id = self.id.clone();
        // The optional file field is not used internally, it is not displayed in the output
        // let file = self.file.clone().unwrap_or_default();
        writeln!(writer, r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/{id}" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>"#).replace("Failed to render")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use liquid_core::parser;
    use liquid_core::runtime;
    use liquid_core::runtime::RuntimeBuilder;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("youtube".to_string(), YoutubeTag.into());
        options
    }

    #[test]
    fn youtube_id_only() {
        let options = options();
        let template = parser::parse(r#"{% youtube id="R2_D2" %}"#, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();

        let output = template.render(&runtime).unwrap();
        assert_eq!(
            output,
            format!(
                "{}\n",
                r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/R2_D2" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>"#
            )
        );
    }
}
