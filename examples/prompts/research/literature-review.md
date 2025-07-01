---
title: Literature Review Assistant
description: Help conduct systematic literature reviews on academic topics
arguments:
  - name: topic
    description: The research topic or question
    required: true
  - name: field
    description: Academic field or discipline
    required: false
    default: "general"
  - name: timeframe
    description: Time period for literature (e.g., "last 5 years", "2010-2023")
    required: false
    default: "last 10 years"
  - name: scope
    description: Scope of review (broad overview, specific aspect, methodology, etc.)
    required: false
    default: "comprehensive overview"
---

# Literature Review: {{topic}}

Please help me conduct a systematic literature review on {{topic}} in the {{field}} field.

## Review Parameters
- **Topic**: {{topic}}
- **Field**: {{field}}
- **Timeframe**: {{timeframe}}
- **Scope**: {{scope}}

## Literature Review Structure

### 1. Research Question Definition
Help me refine the research question:
- What specific aspects of {{topic}} should be explored?
- What are the key variables or concepts to investigate?
- What is the scope and boundaries of this review?

### 2. Search Strategy
Develop a comprehensive search strategy:

#### Keywords and Search Terms
- Primary keywords related to {{topic}}
- Synonyms and alternative terms
- Boolean operators for complex searches
- Field-specific terminology

#### Database Recommendations
Suggest relevant academic databases for {{field}}:
- Primary databases (e.g., PubMed, IEEE Xplore, JSTOR)
- Secondary databases and repositories
- Grey literature sources
- Conference proceedings

#### Inclusion/Exclusion Criteria
Define criteria for:
- Publication type (peer-reviewed articles, books, etc.)
- Language requirements
- Geographic scope
- Methodological approaches
- Quality thresholds

### 3. Literature Analysis Framework

#### Thematic Organization
Suggest how to organize findings:
- Major themes or categories
- Theoretical frameworks
- Methodological approaches
- Chronological developments
- Geographic or demographic patterns

#### Critical Analysis Points
For each source, evaluate:
- Research methodology and design
- Sample size and characteristics
- Key findings and conclusions
- Limitations and biases
- Contribution to the field
- Relevance to research question

### 4. Synthesis and Gaps
Help identify:
- Consensus areas in the literature
- Contradictory findings or debates
- Methodological strengths and weaknesses
- Research gaps and limitations
- Emerging trends or future directions

### 5. Review Structure Template

Provide an outline for:

**Introduction**
- Background and context
- Research question and objectives
- Review methodology

**Main Body** (organized thematically)
- Theme 1: [Key findings and analysis]
- Theme 2: [Key findings and analysis]
- Theme 3: [Key findings and analysis]

**Discussion**
- Synthesis of findings
- Implications for theory and practice
- Limitations of current research

**Conclusion**
- Summary of key insights
- Research gaps and future directions
- Recommendations

### 6. Citation Management
Recommend:
- Reference management tools (Zotero, Mendeley, EndNote)
- Citation style for {{field}}
- Organization strategies for large bibliographies

### 7. Quality Assessment
Provide criteria for evaluating:
- Study design and methodology
- Sample representativeness
- Statistical analysis quality
- Reporting standards
- Potential conflicts of interest

Please provide a detailed framework for conducting this literature review on {{topic}}, including specific search strategies, analysis methods, and organizational approaches suitable for the {{field}} field.