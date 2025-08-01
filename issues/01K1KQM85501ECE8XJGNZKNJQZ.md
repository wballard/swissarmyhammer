Log actions need to render with liquid, using the current workflow variable context and all available variables.

This is in example-actions.md - what happends is -- {{branch_value}} actually gets printed, which exactly nobody is going to want.

- Branch1: Log "Branch 1 selected: {{branch_value}} contains Cargo"
