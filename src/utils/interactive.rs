use crate::project::Project;
use dialoguer::{Confirm, MultiSelect};
use colored::Colorize;

pub fn project_selection(projects: Vec<Project>) -> anyhow::Result<Vec<Project>> {
    if projects.is_empty() {
        return Ok(vec![]);
    }
    
    let items: Vec<String> = projects.iter()
        .map(|p| format!("{} [{}] ({})", 
            p.name().cyan(),
            format!("{:?}", p.project_type).yellow(),
            humansize::format_size(p.metadata.estimated_size.unwrap_or(0), humansize::BINARY).green()
        ))
        .collect();
    
    let selections = MultiSelect::new()
        .with_prompt("Select projects to clean")
        .items(&items)
        .interact()?;
    
    Ok(selections.into_iter()
        .map(|i| projects[i].clone())
        .collect())
}

pub fn confirm_clean(projects: &[Project], total_size: u64) -> anyhow::Result<bool> {
    println!("\n{}", "Summary:".bold());
    println!("  Projects to clean: {}", projects.len().to_string().cyan());
    println!("  Estimated space to free: {}", 
        humansize::format_size(total_size, humansize::BINARY).green());
    
    Confirm::new()
        .with_prompt("Proceed with cleaning?")
        .default(true)
        .interact()
        .map_err(Into::into)
}