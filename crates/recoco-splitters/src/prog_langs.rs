// ReCoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// Original code from CocoIndex is copyrighted by CocoIndex
// SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for ReCoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the ReCoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

//! Programming language detection and tree-sitter support.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};
use unicase::UniCase;

/// Tree-sitter language information for syntax-aware parsing.
pub struct TreeSitterLanguageInfo {
    pub tree_sitter_lang: tree_sitter::Language,
    pub terminal_node_kind_ids: HashSet<u16>,
}

impl TreeSitterLanguageInfo {
    #[allow(dead_code)]
    fn new(
        lang_fn: impl Into<tree_sitter::Language>,
        terminal_node_kinds: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        let tree_sitter_lang: tree_sitter::Language = lang_fn.into();
        let terminal_node_kind_ids = terminal_node_kinds
            .into_iter()
            .filter_map(|kind| {
                let id = tree_sitter_lang.id_for_node_kind(kind, true);
                if id != 0 {
                    Some(id)
                } else {
                    // Node kind not found - this is a configuration issue
                    None
                }
            })
            .collect();
        Self {
            tree_sitter_lang,
            terminal_node_kind_ids,
        }
    }
}

/// Information about a programming language.
pub struct ProgrammingLanguageInfo {
    /// The main name of the language.
    /// It's expected to be consistent with the language names listed at:
    /// <https://github.com/Goldziher/tree-sitter-language-pack/tree/main?tab=readme-ov-file#available-languages>
    pub name: Arc<str>,

    /// Optional tree-sitter language info for syntax-aware parsing.
    pub treesitter_info: Option<TreeSitterLanguageInfo>,
}

static LANGUAGE_INFO_BY_NAME: LazyLock<
    HashMap<UniCase<&'static str>, Arc<ProgrammingLanguageInfo>>,
> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Adds a language to the global map of languages.
    // `name` is the main name of the language, used to set the `name` field of the `ProgrammingLanguageInfo`.
    // `aliases` are the other names of the language, which can be language names or file extensions (e.g. `.js`, `.py`).
    let mut add = |name: &'static str,
                   aliases: &[&'static str],
                   treesitter_info: Option<TreeSitterLanguageInfo>| {
        let config = Arc::new(ProgrammingLanguageInfo {
            name: Arc::from(name),
            treesitter_info,
        });
        for name in std::iter::once(name).chain(aliases.iter().copied()) {
            if map.insert(name.into(), config.clone()).is_some() {
                panic!("Language `{name}` already exists");
            }
        }
    };

    // Languages sorted alphabetically by name
    add("actionscript", &[".as"], None);
    add("ada", &[".ada", ".adb", ".ads"], None);
    add("agda", &[".agda"], None);
    add("apex", &[".cls", ".trigger"], None);
    add("arduino", &[".ino"], None);
    add("asm", &[".asm", ".a51", ".i", ".nas", ".nasm", ".s"], None);
    add("astro", &[".astro"], None);
    add("bash", &[".sh", ".bash"], None);
    add("beancount", &[".beancount"], None);
    add("bibtex", &[".bib", ".bibtex"], None);
    add("bicep", &[".bicep", ".bicepparam"], None);
    add("bitbake", &[".bb", ".bbappend", ".bbclass"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "c")] {
            add(
                "c",
                &[".c", ".cats", ".h.in", ".idc"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_c::LANGUAGE, [])),
            );
        } else {
            add("c", &[".c", ".cats", ".h.in", ".idc"], None);
        }
    }
    add("cairo", &[".cairo"], None);
    add("capnp", &[".capnp"], None);
    add("chatito", &[".chatito"], None);
    add("clarity", &[".clar"], None);
    add(
        "clojure",
        &[
            ".clj", ".boot", ".cl2", ".cljc", ".cljs", ".cljs.hl", ".cljscm", ".cljx", ".hic",
        ],
        None,
    );
    add("cmake", &[".cmake", ".cmake.in"], None);
    add(
        "commonlisp",
        &[
            ".lisp", ".asd", ".cl", ".l", ".lsp", ".ny", ".podsl", ".sexp",
        ],
        None,
    );
    cfg_if::cfg_if! {
        if #[cfg(feature = "cpp")] {
            add(
                "cpp",
                &[
                    ".cpp", ".h", ".c++", ".cc", ".cp", ".cppm", ".cxx", ".h++", ".hh", ".hpp", ".hxx",
                    ".inl", ".ipp", ".ixx", ".tcc", ".tpp", ".txx", "c++",
                ],
                Some(TreeSitterLanguageInfo::new(tree_sitter_cpp::LANGUAGE, [])),
            );
        } else {
            add(
                "cpp",
                &[
                    ".cpp", ".h", ".c++", ".cc", ".cp", ".cppm", ".cxx", ".h++", ".hh", ".hpp", ".hxx",
                    ".inl", ".ipp", ".ixx", ".tcc", ".tpp", ".txx", "c++",
                ],
                None,
            );
        }
    }
    add("cpon", &[".cpon"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "c-sharp")] {
            add(
                "csharp",
                &[".cs", ".cake", ".cs.pp", ".csx", ".linq", "cs", "c#"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_c_sharp::LANGUAGE,
                    [],
                )),
            );
        } else {
            add(
                "csharp",
                &[".cs", ".cake", ".cs.pp", ".csx", ".linq", "cs", "c#"],
                None,
            );
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "css")] {
            add(
                "css",
                &[".css", ".scss"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_css::LANGUAGE, [])),
            );
        } else {
            add("css", &[".css", ".scss"], None);
        }
    }
    add("csv", &[".csv"], None);
    add("cuda", &[".cu", ".cuh"], None);
    add("d", &[".d", ".di"], None);
    add("dart", &[".dart"], None);
    add("dockerfile", &[".dockerfile", ".containerfile"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "xml")] {
            add(
                "dtd",
                &[".dtd"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_xml::LANGUAGE_DTD,
                    [],
                )),
            );
        } else {
            add("dtd", &[".dtd"], None);
        }
    }
    add("elisp", &[".el"], None);
    add("elixir", &[".ex", ".exs"], None);
    add("elm", &[".elm"], None);
    add("embeddedtemplate", &[".ets"], None);
    add(
        "erlang",
        &[
            ".erl", ".app", ".app.src", ".escript", ".hrl", ".xrl", ".yrl",
        ],
        None,
    );
    add("fennel", &[".fnl"], None);
    add("firrtl", &[".fir"], None);
    add("fish", &[".fish"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "fortran")] {
            add(
                "fortran",
                &[".f", ".f90", ".f95", ".f03", "f", "f90", "f95", "f03"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_fortran::LANGUAGE,
                    [],
                )),
            );
        } else {
            add(
                "fortran",
                &[".f", ".f90", ".f95", ".f03", "f", "f90", "f95", "f03"],
                None,
            );
        }
    }
    add("fsharp", &[".fs", ".fsi", ".fsx"], None);
    add("func", &[".func"], None);
    add("gdscript", &[".gd"], None);
    add("gitattributes", &[".gitattributes"], None);
    add("gitignore", &[".gitignore"], None);
    add("gleam", &[".gleam"], None);
    add("glsl", &[".glsl", ".vert", ".frag"], None);
    add("gn", &[".gn", ".gni"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "go")] {
            add(
                "go",
                &[".go", "golang"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_go::LANGUAGE, [])),
            );
        } else {
            add("go", &[".go", "golang"], None);
        }
    }
    add("gomod", &["go.mod"], None);
    add("gosum", &["go.sum"], None);
    add("graphql", &[".graphql", ".gql"], None);
    add(
        "groovy",
        &[".groovy", ".grt", ".gtpl", ".gvy", ".gradle"],
        None,
    );
    add("hack", &[".hack"], None);
    add("hare", &[".ha"], None);
    add("haskell", &[".hs", ".hs-boot", ".hsc"], None);
    add("haxe", &[".hx"], None);
    add("hcl", &[".hcl", ".tf"], None);
    add("heex", &[".heex"], None);
    add("hlsl", &[".hlsl"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "html")] {
            add(
                "html",
                &[".html", ".htm", ".hta", ".html.hl", ".xht", ".xhtml"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_html::LANGUAGE, [])),
            );
        } else {
            add("html", &[".html", ".htm", ".hta", ".html.hl", ".xht", ".xhtml"], None);
        }
    }
    add("hyprlang", &[".hl"], None);
    add("ini", &[".ini", ".cfg"], None);
    add("ispc", &[".ispc"], None);
    add("janet", &[".janet"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "java")] {
            add(
                "java",
                &[".java", ".jav", ".jsh"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_java::LANGUAGE, [])),
            );
        } else {
            add("java", &[".java", ".jav", ".jsh"], None);
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "javascript")] {
            add(
                "javascript",
        &[
            ".js",
            "._js",
            ".bones",
            ".cjs",
            ".es",
            ".es6",
            ".gs",
            ".jake",
            ".javascript",
            ".jsb",
            ".jscad",
            ".jsfl",
            ".jslib",
            ".jsm",
            ".jspre",
            ".jss",
            ".jsx",
            ".mjs",
            ".njs",
            ".pac",
            ".sjs",
            ".ssjs",
            ".xsjs",
            ".xsjslib",
            "js",
        ],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_javascript::LANGUAGE,
                    [],
                )),
            );
        } else {
            add("javascript", &[
                ".js",
                "._js",
                ".bones",
                ".cjs",
                ".es",
                ".es6",
                ".gs",
                ".jake",
                ".javascript",
                ".jsb",
                ".jscad",
                ".jsfl",
                ".jslib",
                ".jsm",
                ".jspre",
                ".jss",
                ".jsx",
                ".mjs",
                ".njs",
                ".pac",
                ".sjs",
                ".ssjs",
                ".xsjs",
                ".xsjslib",
                "js",
            ], None);
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "json")] {
            add(
                "json",
                &[
                    ".json",
                    ".4DForm",
                    ".4DProject",
                    ".avsc",
            ".geojson",
            ".gltf",
            ".har",
            ".ice",
            ".JSON-tmLanguage",
            ".json.example",
            ".jsonl",
            ".mcmeta",
            ".sarif",
            ".tact",
            ".tfstate",
            ".tfstate.backup",
            ".topojson",
            ".webapp",
            ".webmanifest",
            ".yy",
            ".yyp",
        ],
                Some(TreeSitterLanguageInfo::new(tree_sitter_json::LANGUAGE, [])),
    );
        } else {
            add("json", &[
                ".json",
                ".4DForm",
                ".4DProject",
                ".avsc",
                ".geojson",
                ".gltf",
                ".har",
                ".ice",
                ".JSON-tmLanguage",
                ".json.example",
                ".jsonl",
                ".mcmeta",
                ".sarif",
                ".tact",
                ".tfstate",
                ".tfstate.backup",
                ".topojson",
                ".webapp",
                ".webmanifest",
                ".yy",
                ".yyp",
            ], None);
        }
    }
    add("jsonnet", &[".jsonnet"], None);
    add("julia", &[".jl"], None);
    add("kdl", &[".kdl"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "kotlin")] {
            add(
                "kotlin",
                &[".kt", ".ktm", ".kts"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_kotlin_ng::LANGUAGE,
                    [],
                )),
            );
        } else {
            add("kotlin", &[".kt", ".ktm", ".kts"], None);
        }
    }
    add("latex", &[".tex"], None);
    add("linkerscript", &[".ld"], None);
    add("llvm", &[".ll"], None);
    add(
        "lua",
        &[
            ".lua",
            ".nse",
            ".p8",
            ".pd_lua",
            ".rbxs",
            ".rockspec",
            ".wlua",
        ],
        None,
    );
    add("luau", &[".luau"], None);
    add("magik", &[".magik"], None);
    add(
        "make",
        &[".mak", ".make", ".makefile", ".mk", ".mkfile"],
        None,
    );
    cfg_if::cfg_if! {
        if #[cfg(feature = "markdown")] {
            add(
                "markdown",
                &[
                    ".md",
                    ".livemd",
                    ".markdown",
                    ".mdown",
                    ".mdwn",
                    ".mdx",
                    ".mkd",
                    ".mkdn",
                    ".mkdown",
                    ".ronn",
                    ".scd",
                    ".workbook",
                    "md",
                ],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_md::LANGUAGE,
                    ["inline", "indented_code_block", "fenced_code_block"],
                )),
            );
        } else {
            add("markdown", &[".md", ".livemd", ".markdown", ".mdown", ".mdwn", ".mdx", ".mkd", ".mkdn", ".mkdown", ".ronn", ".scd", ".workbook", "md"], None);
        }
    }
    add("mermaid", &[".mmd"], None);
    add("meson", &["meson.build"], None);
    add("netlinx", &[".axi"], None);
    add(
        "nim",
        &[".nim", ".nim.cfg", ".nimble", ".nimrod", ".nims"],
        None,
    );
    add("ninja", &[".ninja"], None);
    add("nix", &[".nix"], None);
    add("nqc", &[".nqc"], None);
    cfg_if::cfg_if! {
    if #[cfg(feature = "pascal")] {
        add(
            "pascal",
            &[
                ".pas", ".dfm", ".dpr", ".lpr", ".pascal", "pas", "dpr", "delphi",
            ],
            Some(TreeSitterLanguageInfo::new(
                tree_sitter_pascal::LANGUAGE,
                [],
            )),
        );
        } else {
            add("pascal", &[".pas", ".dfm", ".dpr", ".lpr", ".pascal", "pas", "dpr", "delphi"], None);
        }
    }
    add("pem", &[".pem"], None);
    add(
        "perl",
        &[
            ".pl", ".al", ".cgi", ".fcgi", ".perl", ".ph", ".plx", ".pm", ".psgi", ".t",
        ],
        None,
    );
    add("pgn", &[".pgn"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "php")] {
            add(
                "php",
                &[".php"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_php::LANGUAGE_PHP,
                    [],
                )),
            );
        } else {
            add("php", &[".php"], None);
        }
    }
    add("po", &[".po"], None);
    add("pony", &[".pony"], None);
    add("powershell", &[".ps1"], None);
    add("prisma", &[".prisma"], None);
    add("properties", &[".properties"], None);
    add("proto", &[".proto"], None);
    add("psv", &[".psv"], None);
    add("puppet", &[".pp"], None);
    add("purescript", &[".purs"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "python")] {
            add(
                "python",
                &[".py", ".pyw", ".pyi", ".pyx", ".pxd", ".pxi"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_python::LANGUAGE,
                    [],
                )),
            );
        } else {
            add("python", &[".py", ".pyw", ".pyi", ".pyx", ".pxd", ".pxi"], None);
        }
    }
    add("qmljs", &[".qml"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "r")] {
            add(
                "r",
                &[".r"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_r::LANGUAGE, [])),
            );
        } else {
            add("r", &[".r"], None);
        }
    }
    add("racket", &[".rkt"], None);
    add("rbs", &[".rbs"], None);
    add("re2c", &[".re"], None);
    add("rego", &[".rego"], None);
    add("requirements", &["requirements.txt"], None);
    add("ron", &[".ron"], None);
    add("rst", &[".rst"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "ruby")] {
            add(
                "ruby",
                &[".rb"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_ruby::LANGUAGE, [])),
            );
        } else {
            add("ruby", &[".rb"], None);
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "rust")] {
            add(
                "rust",
                &[".rs", "rs"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_rust::LANGUAGE, [])),
            );
        } else {
            add("rust", &[".rs", "rs"], None);
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "scala")] {
            add(
                "scala",
                &[".scala"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_scala::LANGUAGE, [])),
            );
        } else {
            add("scala", &[".scala"], None);
        }
    }
    add("scheme", &[".ss"], None);
    add("slang", &[".slang"], None);
    add("smali", &[".smali"], None);
    add("smithy", &[".smithy"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "solidity")] {
            add(
                "solidity",
                &[".sol"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_solidity::LANGUAGE,
                    [],
                )),
            );
        } else {
            add("solidity", &[".sol"], None);
        }
    }
    add("sparql", &[".sparql"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "sql")] {
            add(
                "sql",
                &[".sql"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_sequel::LANGUAGE,
                    [],
                )),
            );
        } else {
            add("sql", &[".sql"], None);
        }
    }
    add("squirrel", &[".nut"], None);
    add("starlark", &[".star", ".bzl"], None);
    add("svelte", &[".svelte"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "swift")] {
            add(
                "swift",
                &[".swift"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_swift::LANGUAGE, [])),
            );
        } else {
            add("swift", &[".swift"], None);
        }
    }
    add("tablegen", &[".td"], None);
    add("tcl", &[".tcl"], None);
    add("thrift", &[".thrift"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "toml")] {
            add(
                "toml",
                &[".toml"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_toml_ng::LANGUAGE,
                    [],
                )),
            );
        } else {
            add("toml", &[".toml"], None);
        }
    }
    add("tsv", &[".tsv"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "typescript")] {
            add(
                "tsx",
                &[".tsx"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_typescript::LANGUAGE_TSX,
                    [],
                )),
            );
        } else {
            add("tsx", &[".tsx"], None);
        }
    }
    add("twig", &[".twig"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "typescript")] {
            add(
                "typescript",
                &[".ts", "ts"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
                    [],
                )),
            );
        } else {
            add("typescript", &[".ts", "ts"], None);
        }
    }
    add("typst", &[".typ"], None);
    add("udev", &[".rules"], None);
    add("ungrammar", &[".ungram"], None);
    add("uxntal", &[".tal"], None);
    add("verilog", &[".vh"], None);
    add("vhdl", &[".vhd", ".vhdl"], None);
    add("vim", &[".vim"], None);
    add("vue", &[".vue"], None);
    add("wast", &[".wast"], None);
    add("wat", &[".wat"], None);
    add("wgsl", &[".wgsl"], None);
    add("xcompose", &[".xcompose"], None);
    cfg_if::cfg_if! {
        if #[cfg(feature = "xml")] {
            add(
                "xml",
                &[".xml"],
                Some(TreeSitterLanguageInfo::new(
                    tree_sitter_xml::LANGUAGE_XML,
                    [],
                )),
            );
        } else {
            add("xml", &[".xml"], None);
        }
    }
    cfg_if::cfg_if! {
        if #[cfg(feature = "yaml")] {
            add(
                "yaml",
                &[".yaml", ".yml"],
                Some(TreeSitterLanguageInfo::new(tree_sitter_yaml::LANGUAGE, [])),
            );
        } else {
            add("yaml", &[".yaml", ".yml"], None);
        }
    }
    add("yuck", &[".yuck"], None);
    add("zig", &[".zig"], None);

    map
});

/// Get programming language info by name or file extension.
///
/// The lookup is case-insensitive and supports both language names
/// (e.g., "rust", "python") and file extensions (e.g., ".rs", ".py").
pub fn get_language_info(name: &str) -> Option<&ProgrammingLanguageInfo> {
    LANGUAGE_INFO_BY_NAME
        .get(&UniCase::new(name))
        .map(|info| info.as_ref())
}

/// Detect programming language from a filename.
///
/// Returns the language name if the file extension is recognized.
pub fn detect_language(filename: &str) -> Option<&str> {
    let last_dot = filename.rfind('.')?;
    let extension = &filename[last_dot..];
    get_language_info(extension).map(|info| info.name.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(
        feature = "all",
        feature = "c",
        feature = "c-sharp",
        feature = "cpp",
        feature = "css",
        feature = "fortran",
        feature = "go",
        feature = "html",
        feature = "java",
        feature = "javascript",
        feature = "json",
        feature = "kotlin",
        feature = "markdown",
        feature = "pascal",
        feature = "php",
        feature = "python",
        feature = "r",
        feature = "ruby",
        feature = "rust",
        feature = "scala",
        feature = "solidity",
        feature = "sql",
        feature = "swift",
        feature = "toml",
        feature = "typescript",
        feature = "xml",
        feature = "yaml"
    ))]
    #[test]
    fn test_get_language_info() {
        let rust_info = get_language_info(".rs").unwrap();
        assert_eq!(rust_info.name.as_ref(), "rust");
        assert!(rust_info.treesitter_info.is_some());

        let py_info = get_language_info(".py").unwrap();
        assert_eq!(py_info.name.as_ref(), "python");

        // Case insensitive
        let rust_upper = get_language_info(".RS").unwrap();
        assert_eq!(rust_upper.name.as_ref(), "rust");

        // Unknown extension
        assert!(get_language_info(".unknown").is_none());
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("test.rs"), Some("rust"));
        assert_eq!(detect_language("main.py"), Some("python"));
        assert_eq!(detect_language("app.js"), Some("javascript"));
        assert_eq!(detect_language("noextension"), None);
        assert_eq!(detect_language("unknown.xyz"), None);
    }
}
