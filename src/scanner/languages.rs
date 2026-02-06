use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Language {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub line_comments: &'static [&'static str],
    pub block_comment_start: Option<&'static str>,
    pub block_comment_end: Option<&'static str>,
}

static RUST: Language = Language {
    name: "Rust",
    extensions: &["rs"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static GO: Language = Language {
    name: "Go",
    extensions: &["go"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static PYTHON: Language = Language {
    name: "Python",
    extensions: &["py", "pyi"],
    line_comments: &["#"],
    block_comment_start: None,
    block_comment_end: None,
};

static JAVASCRIPT: Language = Language {
    name: "JavaScript",
    extensions: &["js", "jsx", "mjs", "cjs"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static TYPESCRIPT: Language = Language {
    name: "TypeScript",
    extensions: &["ts", "tsx"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static JAVA: Language = Language {
    name: "Java",
    extensions: &["java"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static C_LANG: Language = Language {
    name: "C",
    extensions: &["c", "h"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static CPP: Language = Language {
    name: "C++",
    extensions: &["cpp", "cxx", "cc", "hpp", "hxx", "hh"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static CSHARP: Language = Language {
    name: "C#",
    extensions: &["cs"],
    line_comments: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

static RUBY: Language = Language {
    name: "Ruby",
    extensions: &["rb"],
    line_comments: &["#"],
    block_comment_start: None,
    block_comment_end: None,
};

static ALL_LANGUAGES: &[&Language] = &[
    &RUST,
    &GO,
    &PYTHON,
    &JAVASCRIPT,
    &TYPESCRIPT,
    &JAVA,
    &C_LANG,
    &CPP,
    &CSHARP,
    &RUBY,
];

pub struct LanguageDatabase {
    by_extension: HashMap<&'static str, &'static Language>,
}

impl LanguageDatabase {
    pub fn new() -> Self {
        let mut by_extension = HashMap::new();
        for lang in ALL_LANGUAGES {
            for ext in lang.extensions {
                by_extension.insert(*ext, *lang);
            }
        }
        LanguageDatabase { by_extension }
    }

    pub fn from_extension(&self, ext: &str) -> Option<&'static Language> {
        self.by_extension.get(ext).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_extension() {
        let db = LanguageDatabase::new();
        let lang = db.from_extension("rs").unwrap();
        assert_eq!(lang.name, "Rust");
        assert_eq!(lang.line_comments, &["//"]);
        assert_eq!(lang.block_comment_start, Some("/*"));
        assert_eq!(lang.block_comment_end, Some("*/"));
    }

    #[test]
    fn test_python_extensions() {
        let db = LanguageDatabase::new();
        let lang = db.from_extension("py").unwrap();
        assert_eq!(lang.name, "Python");
        assert_eq!(lang.line_comments, &["#"]);
        assert_eq!(lang.block_comment_start, None);

        let lang2 = db.from_extension("pyi").unwrap();
        assert_eq!(lang2.name, "Python");
    }

    #[test]
    fn test_javascript_extensions() {
        let db = LanguageDatabase::new();
        for ext in &["js", "jsx", "mjs", "cjs"] {
            let lang = db.from_extension(ext).unwrap();
            assert_eq!(lang.name, "JavaScript");
        }
    }

    #[test]
    fn test_typescript_extensions() {
        let db = LanguageDatabase::new();
        for ext in &["ts", "tsx"] {
            let lang = db.from_extension(ext).unwrap();
            assert_eq!(lang.name, "TypeScript");
        }
    }

    #[test]
    fn test_cpp_extensions() {
        let db = LanguageDatabase::new();
        for ext in &["cpp", "cxx", "cc", "hpp", "hxx", "hh"] {
            let lang = db.from_extension(ext).unwrap();
            assert_eq!(lang.name, "C++");
        }
    }

    #[test]
    fn test_c_extensions() {
        let db = LanguageDatabase::new();
        for ext in &["c", "h"] {
            let lang = db.from_extension(ext).unwrap();
            assert_eq!(lang.name, "C");
        }
    }

    #[test]
    fn test_go_extension() {
        let db = LanguageDatabase::new();
        let lang = db.from_extension("go").unwrap();
        assert_eq!(lang.name, "Go");
    }

    #[test]
    fn test_java_extension() {
        let db = LanguageDatabase::new();
        let lang = db.from_extension("java").unwrap();
        assert_eq!(lang.name, "Java");
    }

    #[test]
    fn test_csharp_extension() {
        let db = LanguageDatabase::new();
        let lang = db.from_extension("cs").unwrap();
        assert_eq!(lang.name, "C#");
    }

    #[test]
    fn test_ruby_extension() {
        let db = LanguageDatabase::new();
        let lang = db.from_extension("rb").unwrap();
        assert_eq!(lang.name, "Ruby");
        assert_eq!(lang.line_comments, &["#"]);
        assert_eq!(lang.block_comment_start, None);
    }

    #[test]
    fn test_unknown_extension() {
        let db = LanguageDatabase::new();
        assert!(db.from_extension("xyz").is_none());
        assert!(db.from_extension("").is_none());
    }

    #[test]
    fn test_all_ten_languages_registered() {
        let db = LanguageDatabase::new();
        let unique_names: std::collections::HashSet<&str> = db
            .by_extension
            .values()
            .map(|lang| lang.name)
            .collect();
        assert_eq!(unique_names.len(), 10);
    }
}
