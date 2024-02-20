---
title: Special syntax
timestamp: 2024-02-14T08:00:01
published: true
description: Special syntax beyond the regular Markdown
---


## Show the N [most recently published articles](/recent)

```
{% latest limit=5 %}
```

Some people might want to put this in the main `index.md` file, others might create a separate page called `recent.md`
and then link to it from the menu.


## Show the N most recently published article with the given tag

```
{% latest limit=3 tag=programming  %}
```

## [Embed YouTube videos](/youtube)


```
{% youtube id="K6EvVvYnjrY" %}
```

## Embed text file (code)

```
{% include file="" %}
```

