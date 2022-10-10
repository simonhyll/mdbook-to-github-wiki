use mdbook_to_github_wiki::Builder;

fn main() {
    let _ = Builder::new().set_name(".github/wiki").run();
}
