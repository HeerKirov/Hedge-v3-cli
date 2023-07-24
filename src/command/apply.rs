use std::{path::PathBuf, io::stdin, error::Error};
use serde::Deserialize;
use crate::{module::bulk::{BulkModule, SourceDataBulkForm, TagBulkForm, TopicBulkForm, AuthorBulkForm}, utils::error::ApplicationError};
use super::Context;

pub enum ApplyInputType {
    Directory(PathBuf),
    File(PathBuf),
    Input
}

pub async fn apply(context: &mut Context<'_>, input: &Vec<ApplyInputType>, verbose: bool) {
    if let Err(e) = context.server_manager.maintaining_for_start().await {
        eprintln!("Cannot establish connection to server. {}", e);
        return
    }

    let file = match read_input(input) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Apply input read error. {}", e);
            return
        }
    };

    let mut bulk_module = BulkModule::new(&context.server_manager);

    if let Some(source_data) = file.source_data {
        match bulk_module.source_data_bulk_update(&source_data).await {
            Err(e) => eprintln!("Bulk source-data failed. {}", e),
            Ok(result) => {
                println!("Bulk source-data: {} succeed, {} failed, {} errors", result.success, result.failed, result.errors.len());
                if verbose && !result.errors.is_empty() {
                    println!("---");
                    for e in result.errors {
                        println!("Error {}-{}: [{}]{}", e.target.source_site, e.target.source_id, e.error.code, e.error.message);
                    }
                    println!("");
                }
            }
        }
    }
    if let Some(tags) = file.tags {
        match bulk_module.tag_bulk_update(&tags).await  {
            Err(e) => eprintln!("Bulk tags failed. {}", e),
            Ok(result) => {
                println!("Bulk tags: {} succeed, {} failed, {} errors", result.success, result.failed, result.errors.len());
                if verbose && !result.errors.is_empty() {
                    println!("---");
                    for e in result.errors {
                        println!("Error {}: [{}]{}", e.target, e.error.code, e.error.message);
                    }
                    println!("");
                }
            }
        }
    }
    if let Some(topics) = file.topics {
        match bulk_module.topic_bulk_update(&topics).await  {
            Err(e) => eprintln!("Bulk topics failed. {}", e),
            Ok(result) => {
                println!("Bulk topics: {} succeed, {} failed, {} errors", result.success, result.failed, result.errors.len());
                if verbose && !result.errors.is_empty() {
                    println!("---");
                    for e in result.errors {
                        println!("Error {}: [{}]{}", e.target, e.error.code, e.error.message);
                    }
                    println!("");
                }
            }
        }
    }
    if let Some(authors) = file.authors {
        match bulk_module.author_bulk_update(&authors).await  {
            Err(e) => eprintln!("Bulk authors failed. {}", e),
            Ok(result) => {
                println!("Bulk authors: {} succeed, {} failed, {} errors", result.success, result.failed, result.errors.len());
                if verbose && !result.errors.is_empty() {
                    println!("---");
                    for e in result.errors {
                        println!("Error {}: [{}]{}", e.target, e.error.code, e.error.message);
                    }
                    println!("");
                }
            }
        }
    }
}

fn read_input(input: &Vec<ApplyInputType>) -> Result<ApplyFile, Box<dyn Error>> {
    let mut files: Vec<ApplyFile> = Vec::new();
    for i in input {
        match i {
            ApplyInputType::Directory(d) => files.append(&mut read_from_directory(d)?),
            ApplyInputType::File(f) => files.push(read_from_file(f)?),
            ApplyInputType::Input => files.push(read_from_input()?)
        }
    }
    if files.len() == 0 {
        Result::Err(Box::new(ApplicationError::new("apply files is empty.")))
    }else if files.len() == 1 {
        Result::Ok(files.pop().unwrap())
    }else{
        Result::Ok(reduce_files(files))
    }
}

fn read_from_directory(d: &PathBuf) -> Result<Vec<ApplyFile>, Box<dyn Error>> {
    let mut files: Vec<ApplyFile> = Vec::new();
    for item in std::fs::read_dir(d)? {
        if let Ok(entry) = item {
            if entry.file_type()?.is_file() {
                let file_name = entry.file_name();
                let extension_str = PathBuf::from(&file_name).extension().unwrap().to_str().unwrap().to_lowercase();
                let extension = extension_str.as_str();
                if extension == "json" || extension == "yaml" || extension == "toml" {
                    let mut f = d.clone();
                    f.push(&file_name);
                    files.push(read_from_file(&f)?)
                }
            }
        }
    }
    Result::Ok(files)
}

fn read_from_file(f: &PathBuf) -> Result<ApplyFile, Box<dyn Error>> {
    let text = std::fs::read_to_string(f)?;
    let extension_str = f.extension().unwrap().to_str().unwrap().to_lowercase();
    let extension = extension_str.as_str();
    match extension {
        "json" => {
            let json: ApplyFile = serde_json::from_str(&text)?;
            Result::Ok(json)
        },
        "yaml" => {
            let yaml: ApplyFile = serde_yaml::from_str(&text)?;
            Result::Ok(yaml)
        },
        "toml" => {
            let toml: ApplyFile = toml::from_str(&text)?;
            Result::Ok(toml)
        },
        _ => Result::Err(Box::new(ApplicationError::new(&format!("Unsupported file type {}.", extension))))
    }
}

fn read_from_input() -> Result<ApplyFile, Box<dyn Error>> {
    let lines: Vec<String> = stdin().lines().map(|f| f.unwrap()).collect();
    let stdin = lines.join("\n");

    match serde_json::from_str(&stdin) {
        Err(e) => if e.is_data() {
            return Result::Err(Box::new(e));
        },
        Ok(json) => return Result::Ok(json)
    }

    if let Ok(json) = serde_yaml::from_str(&stdin) {
        return Result::Ok(json)
    }

    match toml::from_str(&stdin) {
        Err(e) => return Result::Err(Box::new(e)),
        Ok(json) => return Result::Ok(json)
    }
}

fn reduce_files(mut files: Vec<ApplyFile>) -> ApplyFile {
    let mut ret_source_data: Vec<SourceDataBulkForm> = Vec::new(); 
    let mut ret_tags: Vec<TagBulkForm> = Vec::new(); 
    let mut ret_topics: Vec<TopicBulkForm> = Vec::new(); 
    let mut ret_authors: Vec<AuthorBulkForm> = Vec::new();

    while let Some(f) = files.pop() {
        if let Some(mut source_data) = f.source_data {
            ret_source_data.append(&mut source_data)
        }
        if let Some(mut tags) = f.tags {
            ret_tags.append(&mut tags)
        }
        if let Some(mut topics) = f.topics {
            ret_topics.append(&mut topics)
        }
        if let Some(mut authors) = f.authors {
            ret_authors.append(&mut authors)
        }
    }

    ApplyFile { 
        source_data: if ret_source_data.is_empty() { Option::None }else{ Option::Some(ret_source_data) }, 
        tags: if ret_tags.is_empty() { Option::None }else{ Option::Some(ret_tags) }, 
        topics: if ret_topics.is_empty() { Option::None }else{ Option::Some(ret_topics) }, 
        authors: if ret_authors.is_empty() { Option::None }else{ Option::Some(ret_authors) }
    }
}

#[derive(Deserialize)]
pub struct ApplyFile {
    #[serde(alias = "sourceData")]
    pub source_data: Option<Vec<SourceDataBulkForm>>,
    pub tags: Option<Vec<TagBulkForm>>,
    pub topics: Option<Vec<TopicBulkForm>>,
    pub authors: Option<Vec<AuthorBulkForm>>,
    //TODO 添加settings
}