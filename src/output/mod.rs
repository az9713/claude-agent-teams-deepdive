pub mod text;
pub mod json;
pub mod csv;
pub mod markdown;
pub mod sarif;
pub mod github_actions;

use crate::error::Result;
use crate::model::ScanResult;

pub trait OutputFormatter {
    fn format(&self, result: &ScanResult) -> Result<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Markdown,
    Count,
    Sarif,
    GithubActions,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "count" => Ok(OutputFormat::Count),
            "sarif" => Ok(OutputFormat::Sarif),
            "github-actions" | "github_actions" | "ga" => Ok(OutputFormat::GithubActions),
            other => Err(format!("Unknown output format: {}", other)),
        }
    }
}

pub fn format_output(result: &ScanResult, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Text => {
            let formatter = text::TextFormatter { show_summary: true };
            formatter.format(result)
        }
        OutputFormat::Count => Ok(format!("{}", result.stats.total_todos)),
        OutputFormat::Json => {
            let formatter = json::JsonFormatter;
            formatter.format(result)
        }
        OutputFormat::Csv => {
            let formatter = csv::CsvFormatter;
            formatter.format(result)
        }
        OutputFormat::Markdown => {
            let formatter = markdown::MarkdownFormatter;
            formatter.format(result)
        }
        OutputFormat::Sarif => {
            let formatter = sarif::SarifFormatter;
            formatter.format(result)
        }
        OutputFormat::GithubActions => {
            let formatter = github_actions::GithubActionsFormatter;
            formatter.format(result)
        }
    }
}
