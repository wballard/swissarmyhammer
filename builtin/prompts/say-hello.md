---
name: Say Hello
title: Say Hello
description: A simple greeting prompt that can be customized with name and language
category: basic
tags:
  - greeting
  - hello
  - example
arguments:
  - name: name
    description: The name of the person to greet
    required: false
    default: Friend
  - name: language
    description: The language to greet in
    required: false
    default: English
---

DO NOT run any tools to perform this task:

{% if language == "English" %}
Please respond with: "Hello, {{ name }}! Greetings from Swiss Army Hammer! The workflow system is working correctly."
{% else %}
Please greet {{ name }} in {{ language }} and provide an English translation. Make it warm and friendly.
{% endif %}