# Basic site example

This example shows the minimal project layout: a site config, a single content page, and a base template that renders it.

## Folder structure

```
basic/
  config.yaml
  content/
    index.md
  static/
  templates/
    base.html
```

### What each folder is for

* `config.yaml`: global site settings and arbitrary site-wide data.
* `content/`: markdown pages (each file becomes a page).
* `templates/`: HTML templates used to render pages.
* `static/`: static assets copied as-is (images, CSS, JS, etc.). Empty in this example.

## Site config (`config.yaml`)

Global variables are exposed to templates under `site`.

```yaml
title: My Site
locale: en-US
year: 2025 # arbitrary data
```

## Content page (`content/index.md`)

A page can define front matter (YAML between `---`) and markdown body content.

```md
---
title: Home
---
# Hello, world!
```

* Front matter fields are exposed under `page` (e.g., `page.title`).
* The markdown body is rendered to HTML and exposed as `page.html_content`.

## Base template (`templates/base.html`)

The template reads from:

* `site.*` for global config (from `config.yaml`)
* `page.*` for the current page (from `content/index.md`)

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="locale" content="{{ site.locale }}">
  <title>{{ page.title }}</title>
</head>

<body>
  <header>
    <h1>{{ page.title }}</h1>
  </header>

  <main>
    {{ page.html_content | safe }}
  </main>

  <footer>
    <p>&copy; {{ site.year }} {{ site.title }}</p>
  </footer>
</body>
</html>
```

### Notes

* `{{ page.html_content | safe }}` inserts the rendered HTML from the markdown body.
* `site.year` is just an example of arbitrary config data being used in templates.
