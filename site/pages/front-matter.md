---
title: Front matter (per page meta data)
timestamp: 2024-02-14T17:40:01
published: true
description: Each page must have some meta-data at the top. Some fields are optional.
---

At the top of each page there must be a section for the front-matter that is in YAML format.

Some fields are required.

Some are optional.


```
---
title: The title of the article
timestamp: 2015-10-11T12:30:01       # The timestamp is used for the ordering and therefor it must be unique. We assume it is in UTC.
description: Longer text             # This will be used in the `description` meta field of the page for SEO and in the Open Graph field.
author:                              # The nicname of the author. Each author must be listed in the config.yaml file (this helps you avoid typos)
published: false                     # If it is false the page is generated but not linked from anywhere so only people with the URL can see it.
                                     # Set `published` to `true` to link it from the archive, tags, atom, sitemap.xml.
                                     # The `code-maven drafts` command will list all the articles that are set to `false`.

tags:                                # tags are free text, but remember it is markdown, so some characters might need to be in quotes.
  - web
  - rust

todo:                                # As I write articles I often have ideas for related articles I put them in the todo list
                                     # The `code-maven todo` command will list all the TODO items
  - Add another article extending on the topic
  - Add an article describing a prerequisite

redirect: /some-other-page           # If included (and if the page is published) we will generate a HTML-base redirection.

show_related: true                   # If the `show_related` field in the config.yaml is `false` then this optional field has no impact
                                     # If the `show_related` field in the config.yaml is `true` then this defaults to `true`, but you can set it to fals to not sow the related fields of a specific page.
---
```


* Moving a page to a new location? No problem. You can add a `redirect` field to the front-matter of the old file and generate and HTML-based redirection pages.




