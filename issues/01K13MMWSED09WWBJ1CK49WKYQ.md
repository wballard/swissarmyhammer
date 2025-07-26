Using Ulid::new() is apparently wrong. Look at https://docs.rs/ulid/latest/ulid/struct.Generator.html

Make sure all Ulid allocated are monotonic.