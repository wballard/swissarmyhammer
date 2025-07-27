in the cli, validate.rs has validation logic in the cli. This is misplaced.

Workflows and Prompts need to be self validating with a Validatable trait that returns Vec<ValidationIssue
>

validate.rs in the cli just needs ot delegate to the trait and format results

there is a lot of dead code in validate.rs marked, remove it all.

There is goofy -- incorrect -- validation logic in validate.rs about alphanumeric characters. The parsers in swissarmyhammer should be the only opinion on parse and character validity.