<div align="center">

# mdbook-to-github-wiki

Turns an `mdbook` book into a github wiki
</div>

## Getting started

```rust
// build.rs
fn main() -> Result<(), std::io::Error> {
    let _ = mdbook_to_github_wiki::Builder::new()
        .set_name(".github/wiki")
        .run();
    Ok(())
}
```
