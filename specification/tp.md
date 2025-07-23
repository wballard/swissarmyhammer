Acquire the functionality from https://github.com/glamp/turboprop.

This will add a new sub-command `search`, with `index` and `query` (rename from `search` in the original).

There is no need for a `watch` command.

`index` will take a glob -- so an LLM can ask to index all files, all source files etc.

Use mistral.rs for the models and embedding.

Continue with DuckDB for storing and searching vectors.  Store this in .swissarmyhammer. Make sure it is in .gitignore, this is not cool to check in. No need for file limits of multiple databases.

When using DuckDB, open and close the file on demand -- use DuckDb itself as the file lock coordination mechanism if multiple processes want to modify the database -- indexing a file is the best example

Integrate `search` and `index` as mcp commands.


Indexing should be smart with MD5 content hashing to avoid re-embedding files that have not changed.

Use nomic for the model https://huggingface.co/nomic-ai/nomic-embed-code