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

{% case detail_level %}
{% when "basic" %}
Please provide basic help and information about {{topic}}.
{% when "detailed" %}
Please provide comprehensive, detailed help and information about {{topic}}.
{% else %}
Please provide help and information about {{topic}}.
{% endcase %}