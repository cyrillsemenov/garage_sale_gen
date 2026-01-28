# Static site builder (Haven't come up with a name yet)

This is rudimentary, small and simple static site builder written over a couple of weekends for fun and learning.

It does not have fancy stuff:
- No themes
- No pagination
- No live preview server (but I want to try to add one at some point)
- No state or lockfiles
- I dont't even know all the fancy stuff they usually do

But it does have a nice cli, supports environment variables, and has jinja-style templates.

I also plan to add a scaffolding command.

## Usage

The CLI tool provides several commands to help you manage your site.

### Commands

```sh
# Print the help message to see all available commands and options.
garage_sale_gen help

# List available examples
garage_sale_gen scaffold --list
# Create a new site from an example template.
garage_sale_gen scaffold --example basic my-new-site

# Print the help message to see all available arguments.
garage_sale_gen build --help
# Build the static site from your markdown content.
garage_sale_gen build --base-path ./content --output-path ./public
```

### GitHub Actions & GitHub Pages

You can easily deploy your site to GitHub Pages using the provided GitHub Action.

1.  **Enable GitHub Pages** in your repository settings (Settings > Pages). Set the source to **GitHub Actions**.
2.  Create a workflow file `.github/workflows/deploy.yml`:

```yaml
name: Deploy to GitHub Pages

on:
  push:
    branches: ["main"]

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Build Site
        uses: cyrillsemenov/garage_sale_gen@main
        with:
          source: 'examples/garage_sale' # Path to your site source
          output: 'public'

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: 'public'

      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

## Theory

The site builder first collects all pages and static files, then builds a dependency graph between pages.

Each page named `index.md`, or any page that has children, is a collection root.

Then we compute a topological order of pages (children first, dfs, kahn's algorithm - i dnot know a shit in your computer science, but whatever). After that, we update each child context based on its parent’s context (`children_attrs` value from parent front matter is used to pass attributes to children, children can override parent values), and pass children’s contexts back up to parent pages. A few passes back and forth and we end up with decent contexts that let us build things like databases, blogs, portfolios, garage sales, etc.

We can even skip building children and build a single parent page only (useful for "database"). By database I don’t mean shit like SQL, Mongo, or whatever - just lists of things you need or like.

Every page supports arbitrary values in front matter, so you are responsible for your data structures and templates.

## Why?

There are plenty of static site generators, of course. My goal is not to make something new, but to learn how it is made and to design something i would personally use.

If you need a static website, please use Hugo, Jekyll, or whatever: you already know them, and there is already plenty of info about them.

Although, I would be really happy if someone finds this project useful. It is simple enough that you can actually read through the source code and get the vibe (TW!). You can also contribute or ask for features/fixes—just don’t be ridiculous and please don't demand too much.
