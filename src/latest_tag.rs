use std::io::Write;

use liquid_core::error::ResultLiquidReplaceExt;
use liquid_core::model::Scalar;
use liquid_core::parser::TryMatchToken;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::ValueView;
use liquid_core::{ParseTag, TagReflection, TagTokenIter};
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default)]
pub struct LatestTag;

#[allow(clippy::missing_trait_methods)]
impl TagReflection for LatestTag {
    fn tag(&self) -> &'static str {
        "latest"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for LatestTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        arguments
            .expect_next("limit expected")?
            .expect_str("limit")
            .into_result_custom_msg("limit expected.")?;

        arguments
            .expect_next("Assignment operator \"=\" expected.")?
            .expect_str("=")
            .into_result_custom_msg("Assignment operator \"=\" expected.")?;

        let limit_token = arguments.expect_next("Identifier or value expected")?;
        let limit_value = match limit_token.expect_literal() {
            TryMatchToken::Matches(name) => name.to_kstr().to_string(),
            TryMatchToken::Fails(name) => return name.raise_error().into_err(),
        };
        let Ok(limit) = limit_value.parse::<u8>() else {
            return Err(liquid_core::error::Error::with_msg("Expected number"));
        };

        let key = arguments.next();

        let tag = match key {
            Some(tag_token) => {
                tag_token
                    .expect_str("tag")
                    .into_result_custom_msg("expected tag")?;
                arguments
                    .expect_next("Assignment operator \"=\" expected.")?
                    .expect_str("=")
                    .into_result_custom_msg("Assignment operator \"=\" expected.")?;

                let literal = arguments.expect_next("value of tag")?.expect_literal();

                let tag_value = match literal {
                    TryMatchToken::Matches(name) => name.to_kstr().to_string(),
                    TryMatchToken::Fails(name) => return name.raise_error().into_err(),
                };
                Some(tag_value)
            }
            None => None,
        };

        arguments.expect_nothing()?;

        Ok(Box::new(Latest { limit, tag }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}

#[derive(Debug)]
struct Latest {
    limit: u8,
    tag: Option<String>,
}

#[allow(clippy::missing_trait_methods)]
impl Renderable for Latest {
    fn render_to(&self, writer: &mut dyn Write, runtime: &dyn Runtime) -> Result<()> {
        let mut count = 0;

        let selected_tag = self.tag.clone().unwrap_or_default();

        match runtime.get(&[Scalar::new("items")]) {
            // Ok(values) => values.as_array().unwrap().values().collect::<Vec<_>>(),
            Ok(values) => {
                for value in values
                    .as_array()
                    .ok_or(liquid_core::error::Error::with_msg(
                        "Expected to be an array",
                    ))?
                    .values()
                {
                    let obj = value
                        .as_object()
                        .ok_or(liquid_core::error::Error::with_msg("Expected arrayObject"))?;
                    let title = obj
                        .get("title")
                        .ok_or(liquid_core::error::Error::with_msg("title is missing"))?
                        .to_kstr()
                        .to_string();
                    let url_path = obj
                        .get("url_path")
                        .ok_or(liquid_core::error::Error::with_msg("url_path is missing"))?
                        .to_kstr()
                        .to_string();
                    let tags = obj
                        .get("tags")
                        .ok_or(liquid_core::error::Error::with_msg("Expected tags"))?
                        .as_array()
                        .ok_or(liquid_core::error::Error::with_msg("Expected array"))?
                        .values()
                        .map(|val| val.to_kstr().to_string())
                        .collect::<Vec<_>>();
                    if self.tag.is_some() && !tags.contains(&selected_tag) {
                        continue;
                    }
                    if url_path == "archive" {
                        continue;
                    }

                    writeln!(writer, "* [{title}](/{url_path})").replace("Failed to render")?;
                    count += 1;
                    if count >= self.limit {
                        break;
                    }
                }
            }
            Err(_) => return Err(liquid_core::error::Error::with_msg("Expected number")),
        };

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct Item<'aa> {
    title: &'aa str,
    url_path: &'aa str,
    tags: &'aa [&'aa str],
}

impl<'aa> Item<'aa> {
    pub fn new(title: &'aa str, url_path: &'aa str, tags: &'aa [&'aa str]) -> Self {
        Self {
            title,
            url_path,
            tags,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_items() -> &'static [Item<'static>] {
        let items = &[
            Item {
                title: "one",
                url_path: "1",
                tags: &["web"],
            },
            Item {
                title: "two",
                url_path: "2",
                tags: &["programming"],
            },
            Item {
                title: "three",
                url_path: "3",
                tags: &["web"],
            },
            Item {
                title: "four",
                url_path: "4",
                tags: &["programming"],
            },
            Item {
                title: "five",
                url_path: "5",
                tags: &["web"],
            },
            Item {
                title: "six",
                url_path: "6",
                tags: &["programming"],
            },
            Item {
                title: "seven",
                url_path: "7",
                tags: &["web"],
            },
            Item {
                title: "eight",
                url_path: "8",
                tags: &["programming"],
            },
            Item {
                title: "nine",
                url_path: "9",
                tags: &["web"],
            },
            Item {
                title: "ten",
                url_path: "10",
                tags: &["web"],
            },
        ];
        items
    }

    use liquid_core::object;
    use liquid_core::parser;
    use liquid_core::runtime;
    use liquid_core::runtime::RuntimeBuilder;
    use liquid_core::Value;

    fn options() -> Language {
        let mut options = Language::default();
        options
            .tags
            .register("latest".to_string(), LatestTag.into());
        options
    }

    #[test]
    fn latest_5_none() {
        let options = options();
        let template = parser::parse(r#"{% latest limit=5 %}"#, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();

        let objects = get_items()
            .iter()
            .map(|item| {
                let obj =
                    object!({"title": item.title, "url_path": item.url_path, "tags": item.tags});
                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        runtime.set_global("items".into(), Value::Array(objects));

        let output = template.render(&runtime).unwrap();
        assert_eq!(
            output,
            "* [one](/1)\n* [two](/2)\n* [three](/3)\n* [four](/4)\n* [five](/5)\n"
        );
    }

    #[test]
    fn latest_5_web() {
        let options = options();
        let template = parser::parse(r#"{% latest limit=5   tag="web" %}"#, &options)
            .map(runtime::Template::new)
            .unwrap();

        let runtime = RuntimeBuilder::new().build();

        let objects = get_items()
            .iter()
            .map(|item| {
                let obj =
                    object!({"title": item.title, "url_path": item.url_path, "tags": item.tags});
                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        runtime.set_global("items".into(), Value::Array(objects));

        let output = template.render(&runtime).unwrap();
        assert_eq!(
            output,
            "* [one](/1)\n* [three](/3)\n* [five](/5)\n* [seven](/7)\n* [nine](/9)\n"
        );
    }
}
