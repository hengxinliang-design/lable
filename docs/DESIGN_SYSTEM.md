# Label Platform Design System

## Coral Reef Color Palette

The product design palette uses the Coral Reef summer color set.

| Token | Hex | Usage |
|---|---:|---|
| `--color-deep-teal` | `#01666A` | Primary navigation, headers, dense controls, key structural accents |
| `--color-lagoon` | `#159CBA` | Primary actions, links, active states, charts |
| `--color-shell` | `#FAF5F2` | App background, quiet surfaces, page bands |
| `--color-coral-light` | `#FEC7C2` | Soft warnings, selected-row backgrounds, empty-state accents |
| `--color-coral` | `#FF714F` | High-emphasis CTAs, alerts, important status highlights |

## Product Color Roles

Core roles:
- Primary: `#01666A`
- Secondary: `#159CBA`
- Background: `#FAF5F2`
- Soft accent: `#FEC7C2`
- Action accent: `#FF714F`

Recommended UI mapping:
- Header and sidebar: deep teal.
- Primary buttons and active tabs: lagoon.
- Page background: shell.
- Secondary panels and gentle highlights: coral light.
- Destructive or high-attention actions: coral.
- Body text should remain near-black for readability.

## Accessibility Notes

- Use white text on `#01666A` and `#159CBA`.
- Use dark text on `#FAF5F2` and `#FEC7C2`.
- Use white or near-black text on `#FF714F` depending on size and emphasis; prefer dark text for small labels.
- Do not use coral light as text color on shell background.

## Implementation Tokens

```css
:root {
  --color-deep-teal: #01666A;
  --color-lagoon: #159CBA;
  --color-shell: #FAF5F2;
  --color-coral-light: #FEC7C2;
  --color-coral: #FF714F;
}
```
