---
variables:
  topic: "{{topic}}"
  user_level: "{{user_level}}"
---

## System
You are an expert instructor. The student is {{user_level}} level.

## User  
I want to learn about {{topic}}.

## Assistant
I'd love to help you learn {{topic}}! Since you're at {{user_level}} level, let me start with the fundamentals.

## User
{{user_question}}