use std::path::PathBuf;
use chrono::NaiveDate;
use crate::module::import::{ImportModule, OrderTimeType};
use super::Context;


pub async fn list(context: &mut Context<'_>) {
    if let Err(e) = context.server_manager.waiting_for_start().await {
        eprintln!("Cannot establish connection to server. {}", e);
        return
    }

    let mut import_module = ImportModule::new(context.server_manager);
    let r = match import_module.list().await {
        Err(e) => {
            eprintln!("Error occrred in requesting. {}", e.to_string());
            return
        },
        Ok(r) => r
    };
    for item in &r.result {
        println!("-{:3}| {:50} | {}", item.id, item.file_name.as_ref().unwrap_or(&"".to_string()), item.partition_time)
    }
    if r.result.len() > 0 {
        println!("---");
    }
    println!("Total {} result(s).", r.total);
}

pub async fn add(context: &mut Context<'_>, files: &Vec<PathBuf>, remove: bool) {
    if files.len() > 0 {
        if let Err(e) = context.server_manager.waiting_for_start().await {
            eprintln!("Cannot establish connection to server. {}", e);
            return
        }

        let mut import_module = ImportModule::new(context.server_manager);
        for file in files {
            if let Err(e) = import_module.add(file, remove).await {
                println!("{} add failed. {}", file.to_str().unwrap(), e.to_string());
            }else{
                println!("{} added.", file.to_str().unwrap());
            }
        }
    }
}

pub async fn batch(context: &mut Context<'_>, partition_time: Option<NaiveDate>, create_time: Option<OrderTimeType>, order_time: Option<OrderTimeType>, analyse_source: bool) {
    if partition_time.is_some() || create_time.is_some() || order_time.is_some() || analyse_source {
        if let Err(e) = context.server_manager.waiting_for_start().await {
            eprintln!("Cannot establish connection to server. {}", e);
            return
        }

        let mut import_module = ImportModule::new(context.server_manager);
        let r = match import_module.batch(partition_time, create_time, order_time, analyse_source).await {
            Err(e) => {
                eprintln!("Error occrred in requesting. {}", e.to_string());
                return
            },
            Ok(r) => r
        };

        if r.len() > 0 {
            for res in &r {
                let reason: String = res.warnings.iter().map(|e| e.message.as_str()).collect();
                println!("-{:3}| {}", res.id, reason);
            }
            println!("---");
            println!("Some items batch failed.");
        }else{
            println!("Batch Succeed.");
        }
    }
}

pub async fn save(context: &mut Context<'_>) {
    if let Err(e) = context.server_manager.waiting_for_start().await {
        eprintln!("Cannot establish connection to server. {}", e);
        return
    }

    let mut import_module = ImportModule::new(context.server_manager);
    let r = match import_module.save().await {
        Err(e) => {
            eprintln!("Error occrred in requesting. {}", e.to_string());
            return
        },
        Ok(r) => r
    };
    for e in &r.errors {
        let mut v = Vec::new();
        if e.file_not_ready { v.push("File not ready.") }
        if e.not_existed_clone_image_id.is_some() { v.push("Preference clone image not exist.") }
        if e.not_existed_collection_id.is_some() { v.push("Preference collection not exist.") }
        if e.not_existed_book_ids.is_some() { v.push("Preference book not exist.") }
        if e.not_existed_folder_ids.is_some() { v.push("Preference folder not exist.") }
        let reason = v.join(" ");
        
        println!("-{:3}| {}", e.import_id, reason);
    }
    if (&r.errors).len() > 0 {
        println!("---");
        println!("{} item(s) saved. {} item(s) save failed.", r.total, r.errors.len());
    }else{
        println!("{} item(s) saved.", r.total);
    }
}