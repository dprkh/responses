---
variables:
  user_count: "{{user_count}}"
  locale_key: "system"
---

# {{i18n "system.title"}}

{{> shared/status_header.md active_users=user_count system_status=status}}

## Current Status
{{i18n "current_tasks" count=task_count tasks=(plural task_count "task")}}

## Performance Metrics
- Users: {{plural user_count "user"}}
- Progress: {{format_number completion style="percent"}}
- Load: {{format_number cpu_usage style="decimal"}}

{{> shared/footer.md version=system_version}}