---
variables:
  role: "assistant"
  active: true
i18n_key: "system"
---

# Simple Template

You are a helpful {{role}}. This template is {{#if active}}active{{else}}inactive{{/if}}.