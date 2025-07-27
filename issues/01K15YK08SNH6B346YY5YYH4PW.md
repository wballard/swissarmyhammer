```
1. ./swissarmyhammer/src/workflow/cache.rs (score: 0.748)
   pub fn is_expired(&self, ttl: Duration) -> bool {
           self.cached_at.elapsed() > ttl
```

search query results need file and line numbers in a clickable

file:line like ./source.rs:23

you'll need to update the table and indexes to have line number for the extracted functions