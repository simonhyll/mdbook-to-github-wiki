use std::io::{BufReader, Read, Write};

pub struct Builder {
    name: String,
    source: String,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            name: "wiki".to_string(),
            source: "book".to_string(),
        }
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn set_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    pub fn run(self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(format!("{}", self.name))?;
        let mut sidebar_md = std::fs::File::create(format!("{}/_Sidebar.md", self.name))?;
        let mut lines: Vec<String> = vec!["[Home](home)\n".to_string()];

        let summary_md = std::fs::File::open(format!("{}/src/SUMMARY.md", self.source))?;
        let mut reader = BufReader::new(summary_md);
        let mut summary_contents = "".to_string();
        let _ = reader.read_to_string(&mut summary_contents);
        let mut summary_lines: Vec<String> = summary_contents
            .split("\n")
            .filter(|n| n.contains("(") && n.contains("["))
            .map(|n| n.to_string())
            .collect();

        #[derive(Debug)]
        enum StringOrVec {
            String(String),
            Vec(Vec<StringOrVec>),
        }

        fn create_branch(lines: &mut Vec<String>, current_level: i32) -> Vec<StringOrVec> {
            let mut branch: Vec<StringOrVec> = vec![];
            while lines.len() > 0 {
                let value = lines.remove(0);

                let mut x: Vec<String> = value.split("").map(|n| n.to_string()).collect();
                let mut spaces: Vec<String> = vec![];
                if x[0] == "" {
                    x.remove(0);
                }
                while x[0] == " " {
                    spaces.push(x.remove(0));
                }

                let new_level = (spaces.len() / 2) as i32;

                if current_level == new_level {
                    branch.push(StringOrVec::String(value));
                } else if current_level < new_level {
                    lines.insert(0, value);
                    branch.push(StringOrVec::Vec(create_branch(lines, new_level)));
                } else if current_level > new_level {
                    lines.insert(0, value);
                    return branch;
                }
            }
            branch
        }
        let tree = create_branch(&mut summary_lines, 0);

        fn add_lines(lines: &mut Vec<String>, tree: &Vec<StringOrVec>, source: String) {
            for entry in tree {
                match entry {
                    StringOrVec::String(s) => {
                        lines.push(format!(
                            "{}",
                            s.replace("./", "").replace("/", "-").replace(".md", "")
                        ));
                    }
                    StringOrVec::Vec(v) => {
                        add_lines(lines, v, source.to_string());
                    }
                }
            }
        }
        add_lines(&mut lines, &tree, self.source.clone());

        lines.push("".to_string());
        let contents = lines.join("\n");

        sidebar_md.write_all(contents.as_bytes())?;

        let _ = std::fs::copy("README.md", format!("{}/home.md", self.name))?;

        let mut progression: Vec<String> = vec![];
        for n in lines.iter().filter(|n| n.contains("- ")) {
            let filename: String = n.split("(").collect::<Vec<&str>>()[1].replace(")", ".md");
            progression.push(filename);
        }

        for entry in walkdir::WalkDir::new(format!("{}/src", self.source))
            .min_depth(1)
            .into_iter()
            .filter(|n| n.is_ok())
            .map(|n| n.unwrap())
        {
            if entry.file_name() == "SUMMARY.md" || entry.path().is_dir() {
                continue;
            }
            let mut target = format!("{}", entry.path().to_str().unwrap());
            target = target
                .strip_prefix(format!("{}/src{}", self.source, std::path::MAIN_SEPARATOR).as_str())
                .unwrap()
                .to_string();
            target = target.replace("\\", "-");
            target = target.replace("/", "-");
            target = format!("{}/{}", self.name, target);

            let mut data = std::fs::read(entry.path())?;

            let i = progression
                .iter()
                .position(|n| n.eq(&target.replace(format!("{}/", &self.name).as_str(), "")));
            if i.is_some() {
                let index = i.unwrap();
                let p;
                if index == 0 {
                    p = None;
                } else {
                    p = progression.get(index - 1);
                }
                let n = progression.get(index + 1);
                let previous;
                let next;
                // Next is something, previous is not
                if n.is_some() && p.is_none() {
                    next = n.unwrap().replace(".md", "");
                    data.splice(
                        0..0,
                        format!(
                            r#"<table>
<tr>
<td><a href="home">◀</a></td>
<td width="9999" align="center"></td>
<td><a href="{next}">▶</a></td>
</tr>
</table>

"#
                        )
                        .bytes(),
                    );
                }
                // Next is nothing, previous is something
                else if n.is_none() && p.is_some() {
                    previous = p.unwrap().replace(".md", "");
                    data.splice(
                        0..0,
                        format!(
                            r#"<table>
<tr>
<td><a href="{previous}">◀</a></td>
<td width="9999" align="center"></td>
</tr>
</table>

"#
                        )
                        .bytes(),
                    );
                }
                // Both next and previous are something
                else if n.is_some() && p.is_some() {
                    previous = p.unwrap().replace(".md", "");
                    next = n.unwrap().replace(".md", "");
                    data.splice(
                        0..0,
                        format!(
                            r#"<table>
<tr>
<td><a href="{previous}">◀</a></td>
<td width="9999" align="center"></td>
<td><a href="{next}">▶</a></td>
</tr>
</table>

"#
                        )
                        .bytes(),
                    );
                }
            }

            let _ = std::fs::write(&target, data);
        }
        Ok(())
    }
}
