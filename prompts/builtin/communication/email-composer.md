---
title: Smart Email Composer
description: Compose emails with dynamic content using capture blocks
arguments:
  - name: recipient_name
    description: Name of the email recipient
    required: true
  - name: sender_name
    description: Name of the sender
    required: true
  - name: email_type
    description: Type of email (welcome, followup, reminder, thank_you)
    required: true
  - name: context
    description: Additional context for the email
    default: ""
  - name: formal
    description: Use formal tone
    default: "false"
  - name: include_signature
    description: Include email signature
    default: "true"
  - name: time_of_day
    description: Current time of day (morning, afternoon, evening)
    default: "morning"
---

{% comment %} Capture the greeting based on time and formality {% endcomment %}
{% capture greeting %}
{% if formal == "true" %}
  {% case time_of_day %}
    {% when "morning" %}Good morning
    {% when "afternoon" %}Good afternoon
    {% when "evening" %}Good evening
    {% else %}Dear
  {% endcase %}
{% else %}
  {% case time_of_day %}
    {% when "morning" %}Hi
    {% when "afternoon" %}Hello
    {% when "evening" %}Hey
    {% else %}Hi
  {% endcase %}
{% endif %}
{% endcapture %}

{% comment %} Capture the subject line {% endcomment %}
{% capture subject_line %}
{% case email_type %}
  {% when "welcome" %}
    Welcome to our community{% if recipient_name %}, {{ recipient_name }}{% endif %}!
  {% when "followup" %}
    Following up on our conversation
  {% when "reminder" %}
    Friendly reminder: {{ context | default: "Action required" }}
  {% when "thank_you" %}
    Thank you{% if context %} for {{ context }}{% endif %}
  {% else %}
    {{ email_type | capitalize }} - {{ context | default: "Important message" }}
{% endcase %}
{% endcapture %}

{% comment %} Capture the main body content {% endcomment %}
{% capture body_content %}
{% case email_type %}
  {% when "welcome" %}
    {% if formal == "true" %}
    It is our pleasure to welcome you to our community. We are delighted to have you join us and look forward to a productive relationship.
    
    As a new member, you will have access to:
    - Exclusive resources and materials
    - Regular updates and newsletters
    - Community support and networking opportunities
    {% else %}
    We're thrilled to have you join our community! 🎉
    
    Here's what you can expect:
    - Awesome resources to help you get started
    - Regular updates on what's happening
    - A supportive community ready to help
    {% endif %}
    
  {% when "followup" %}
    {% if formal == "true" %}
    I trust this message finds you well. I am writing to follow up on our recent discussion{% if context %} regarding {{ context }}{% endif %}.
    
    I wanted to ensure that all your questions were addressed and to offer any additional assistance you might require.
    {% else %}
    I hope you're doing well! I wanted to follow up on our chat{% if context %} about {{ context }}{% endif %}.
    
    Just checking if you have any questions or if there's anything else I can help with.
    {% endif %}
    
  {% when "reminder" %}
    {% if formal == "true" %}
    This is a courtesy reminder{% if context %} regarding: {{ context }}{% endif %}.
    
    Please take a moment to review this matter at your earliest convenience.
    {% else %}
    Just a quick reminder{% if context %} about: {{ context }}{% endif %}.
    
    When you get a chance, please take a look. Thanks!
    {% endif %}
    
  {% when "thank_you" %}
    {% if formal == "true" %}
    I would like to express my sincere gratitude{% if context %} for {{ context }}{% endif %}.
    
    Your contribution has been invaluable, and we greatly appreciate your efforts.
    {% else %}
    I wanted to say a big thank you{% if context %} for {{ context }}{% endif %}!
    
    Your help means a lot, and I really appreciate it.
    {% endif %}
    
  {% else %}
    {{ context | default: "I hope this message finds you well." }}
{% endcase %}
{% endcapture %}

{% comment %} Capture the closing {% endcomment %}
{% capture closing %}
{% if formal == "true" %}
  {% case email_type %}
    {% when "welcome" %}Warmest regards
    {% when "followup" %}Best regards
    {% when "reminder" %}Kind regards
    {% when "thank_you" %}With sincere appreciation
    {% else %}Sincerely
  {% endcase %}
{% else %}
  {% case email_type %}
    {% when "welcome" %}Cheers
    {% when "followup" %}Best
    {% when "reminder" %}Thanks
    {% when "thank_you" %}Many thanks
    {% else %}Take care
  {% endcase %}
{% endif %}
{% endcapture %}

{% comment %} Capture the signature if needed {% endcomment %}
{% capture signature %}
{% if include_signature == "true" %}
{{ sender_name }}
{% if formal == "true" %}
[Your Title]
[Company Name]
[Contact Information]
{% endif %}
{% endif %}
{% endcapture %}

---

**Subject:** {{ subject_line | strip }}

---

{{ greeting | strip }} {{ recipient_name }},

{{ body_content | strip }}

{{ closing | strip }},
{{ signature | strip }}

---

## Email Metadata

- **Type**: {{ email_type | capitalize }}
- **Tone**: {% if formal == "true" %}Formal{% else %}Casual{% endif %}
- **Word Count**: {{ body_content | strip | split: " " | size }}
- **Greeting Style**: {{ greeting | strip }}
- **Closing Style**: {{ closing | strip }}

{% comment %} Create a text-only version using captures {% endcomment %}
{% capture text_version %}
{{ subject_line | strip | upcase }}

{{ greeting | strip }} {{ recipient_name }},

{{ body_content | strip | remove: "  " }}

{{ closing | strip }},
{{ signature | strip }}
{% endcapture %}

## Plain Text Version

```
{{ text_version | strip }}
```