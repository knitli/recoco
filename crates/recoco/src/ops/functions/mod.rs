#[cfg(feature = "function-detect-lang")]
pub mod detect_program_lang;
#[cfg(feature = "function-embed")]
pub mod embed_text;
#[cfg(feature = "function-extract-llm")]
pub mod extract_by_llm;
#[cfg(feature = "function-json")]
pub mod parse_json;
#[cfg(feature = "function-split")]
pub mod split_by_separators;
#[cfg(feature = "function-split")]
pub mod split_recursively;

#[cfg(test)]
mod test_utils;
