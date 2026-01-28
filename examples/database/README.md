# Single-page database with folder-path tags

This example shows how to build a “single page database” from a folder of child pages. The key feature is that you can store files in subfolders and use their **path segments** as additional tags (e.g., a Pokémon placed under `electric/` automatically gains the `electric` tag).

## Structure

Example content tree:

* `index.md` (database page)
* `pokemon/`

  * `electric/pikachu.md`
  * `bulbasaur.md`

## Database page (`index.md`)

The database page declares which pages are considered children and defines default attributes applied to each child.

```yaml
---
title: Pokemon Database

# Child pages live under this folder
children:
  - pokemon

# Defaults applied to children
children_attrs:
  publish: false        # do not render child pages, but make their data available
  types: []             # default list for "types"

  # NOTE:
  # There are no defaults for `name`, `weight`, or `abilities`.
  # If a child page omits them and a template tries to read them, it will error.
---
```

### Notes

* `children`: points to the folder(s) containing entries.
* `children_attrs`: supplies defaults for missing fields across all children.
* Any field without a default must be defined in every child entry that the template expects.

## Child entries

### Example: `electric/pikachu.md`

`types` is omitted because the template will derive tags from the folder path (e.g., `electric`) and merge them with `types`.

```yaml
---
name: Pikachu
abilities:
  - name: static
    hidden: false
  - name: lightning-rod
    hidden: true
weight: 60
---
```

### Example: `bulbasaur.md`

This entry explicitly sets `types`, overriding the default empty list.

```yaml
---
name: Bulbasaur
abilities:
  - name: overgrow
    hidden: false
  - name: chlorophyll
    hidden: true
weight: 60
types:
  - grass
  - poison
---
```

## Rendering template

This template renders a table and computes each Pokémon’s final `Types` list by combining:

1. `item.types` (front matter)
2. `item.path_segments` (derived from its folder path)

Then it removes duplicates and prints a comma-separated list.

```html
<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Abilities</th>
      <th>Types</th>
      <th>Weight</th>
    </tr>
  </thead>
  <tbody>
    {%- for item in page.children %}
    <tr>
      <td>{{ item.name }}</td>
      <td>
        {%- for ability in item.abilities -%}
        <span class="ability{% if ability.hidden %} hidden{% endif %}">
          {%- if not loop.first %}, {% endif -%}
          {{ ability.name -}}
        </span>
        {%- endfor -%}
      </td>
      <td>
        {%- set types = [] | concat(with=item.types) | concat(with=item.path_segments) | pop(value="pokemon")  -%}
        {{ types | unique | join(sep=", ") }}
      </td>
      <td>{{ item.weight }}</td>
    </tr>
    {%- endfor %}
  </tbody>
</table>
```

## Behavior summary

* Putting a file under a folder (e.g., `electric/pikachu.md`) implicitly adds `electric` via `item.path_segments`.
* Explicit `types` defined in the entry are merged with the path tags.
* Duplicate tags are removed before display.
* Missing required fields (`name`, `weight`, `abilities`) will fail at render time if the template references them.
