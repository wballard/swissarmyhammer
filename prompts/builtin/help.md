---
name: help
title: Help Assistant
description: A prompt for providing helpful assistance and guidance to users
arguments:
  - name: topic
    description: The topic to get help about
    required: false
    default: general assistance
  - name: detail_level
    description: How detailed the help should be
    required: false
    default: normal
---

# Help for {{topic}}

Please provide help and information about {{topic}}.