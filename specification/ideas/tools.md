tools:

- Task
  - Bash
    - use https://github.com/QwenLM/qwen-code/blob/main/packages/core/src/tools/shell.ts for inspiration and description
  - Outline
    - use https://github.com/cline/cline/blob/main/src/core/tools/listCodeDefinitionNamesTool.ts for inspiration and description
      - we're expanding on this by going beyond top level symbols
    - languages
      - rust
      - typescript, javascript
      - dart
      - python
    - uses tree sitter to parse sources into a structured yaml
    - specify which files to parse by glob
    - include just the types, methods, and member variables, not the bodies
    - include the documentation comments
    - include the source line
    - honor gitignore
    - yaml is nested to reflect the source tree file structure
      ```yaml
      src:
        utils:
          math.ts:
            children:
              - name: "Calculator"
                kind: "class"
                line: 3
                children:
                  - name: "result"
                    kind: "property"
                    type: "number"
                    line: 5
                  - name: "add"
                    kind: "method"
                    signature: "(a: number, b: number) => number"
                    line: 8
                    doc: "Adds two numbers and returns the result."
              - name: "Operation"
                kind: "enum"
                line: 15
              - name: "multiply"
                kind: "function"
                signature: "(a: number, b: number) => number"
                line: 22
                doc: "Multiplies two numbers."
          string.ts:
            children:
              - name: "StringUtils"
                kind: "class"
                line: 2
                children:
                  - name: "capitalize"
                    kind: "method"
                    signature: "(s: string) => string"
                    line: 4
                    doc: "Capitalizes the first letter of a string."
      ```
  - File (this is a new tool group noun parallel to issues)
    - Read
      - use https://github.com/cline/cline/blob/main/src/core/tools/readTool.ts for inspiration and description
    - Edit
      - use https://github.com/cline/cline/blob/main/src/core/tools/editTool.ts for inspiration and description
    - Write
      - use https://github.com/cline/cline/blob/main/src/core/tools/writeTool.ts for inspiration and description
    - Glob
      - use https://github.com/QwenLM/qwen-code/blob/main/packages/core/src/tools/glob.ts for inspiration and description
    - Grep
      - use https://github.com/cline/cline/blob/main/src/core/tools/grepTool.ts for inspiration and description
      - implement the actual search with ripgrep
  - Todo (this is a new tool group parallel to issues, but works on a named todo file parameter)
    - use the todo list to keep track of work while
    - this is not the same as an issue, in that the todo list is ephemeral and never checked in
    - store the todo list in a yaml nested list format, you will very likely have multiline text for context
      ```yaml
      todo:
        - task: "Implement file read tool"
          context: "Use cline's readTool.ts for inspiration"
        - task: "Add glob support"
          context: "Refer to qwen-code glob.ts"
        - task: "Integrate ripgrep for grep"
          context: "Improve search performance"
        - task: "Write documentation"
          context: "Describe usage for each tool"
      ```
    - Next (todo list)
      - read the very next todo - only one at a time, this forces a FIFO and avoids context pollution with too much to do
    - Add (todo list, thing to do, additional context)
      - add a new item to the todo list
      - auto creates the todo list file if it does not yet exist
    - Delete (todo list, thing that was to do to now delete)
      - remove a done item from the todo list, leaving the work still to be done
  - WebFetch
    - use https://github.com/cline/cline/blob/main/src/core/tools/webFetchTool.ts for inspiration and description
    - use https://github.com/swissarmyhammer/markdowndown for the actual fetch and render
    - direct page read
  - WebSearch
    - use SearXNG search API https://docs.searxng.org/dev/search_api.html
    - https://searx.space/#help-html-grade -- public metasearch for SearXNG hosting to find A+, A+ instances to query with the API
    - fetch the returned URLs with markdowndown to provide content and context, arrange this in an organized markdown