use std::collections::HashMap;
use crate::module::{source_data::SourceDataModule, download::DownloadModule};
use super::Context;


pub async fn query(context: &mut Context<'_>, hql: &str, offset: u32, limit: u32) {
    context.server_manager.waiting_for_start().await;
    let mut source_data_module = SourceDataModule::new(context.server_manager);
    let r = match source_data_module.query(Option::Some(hql), Option::None, Option::None, Option::Some(offset), Option::Some(limit)).await {
        Err(e) => {
            eprintln!("Error occrred in requesting. {}", e.to_string());
            return
        },
        Ok(r) => r
    };
    let site_max_len = (&r.result).iter().map(|f| f.site_name.len()).max().unwrap_or(20);
    let id_max_len = (&r.result).iter().map(|f| f.source_id.to_string().len()).max().unwrap_or(10);
    for item in &r.result {
        let mut v = Vec::new();
        if item.tag_count > 0 { v.push(format!("{} tag(s)", item.tag_count)) }
        if item.book_count > 0 { v.push(format!("{} book(s)", item.book_count)) }
        if item.relation_count > 0 { v.push(format!("{} relation(s)", item.relation_count)) }
        let description = v.join(", ");
        println!("- {:>site_max_len$} | {:>id_max_len$} | ({}) {}", item.site_name, item.source_id, item.status, description, site_max_len = site_max_len, id_max_len = id_max_len);
    }
    if r.result.len() > 0 {
        println!("---");
    }
    println!("Total {} result(s), current {} to {}.", r.total, offset + 1, offset + r.result.len() as u32);
}

pub async fn download(context: &mut Context<'_>) {
    // TODO 需要一个持续发送signal的副线程，最好做成callback调用
    context.server_manager.waiting_for_start().await;
    let mut source_data_module = SourceDataModule::new(context.server_manager);
    // TODO 最终的site参数可配置
    let r = match source_data_module.query(Option::None, Option::Some(vec!["NOT_EDITED", "ERROR"]), Option::Some(vec!["sankakucomplex"]), Option::None, Option::Some(1000)).await {
        Err(e) => {
            eprintln!("Error occrred in requesting. {}", e.to_string());
            return
        },
        Ok(r) => r
    };
    if r.result.len() <= 0 {
        println!("Total {} result(s) found.", r.total);
        return
    }

    println!("Total {} result(s) found. Current processing {} result(s).", r.total, r.result.len());
    println!("---");
    
    let download_module = DownloadModule::new(&context.config);

    let result_count = r.result.len();
    let result_count_str_len = result_count.to_string().len();
    let mut index = 1;
    let mut success = 0;
    let mut failed = 0;
    for item in &r.result {
        //TODO 根据此种类的site是否需要附加数据的配置项，决定是否要get detail
        let additional_info = HashMap::new();
        let dn = download_module.download(&item.site, item.source_id, &additional_info).await;

        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        print!("{} | {:>rc_len$}/{} \x1b[1;33m| {:16} | {:>12} |\x1b[0m", date, index, result_count, item.site, item.source_id, rc_len = result_count_str_len);

        match dn {
            Ok((result, info)) => {
                let form = result.to_update_form();

                match source_data_module.update(&item.site, item.source_id, &form).await {
                    Ok(()) => {
                        println!("\x1b[1;32m Success (in {:.2}s, retry {} time(s))\x1b[0m", (info.time_cost as f64) / 1000.0, info.retry_count);
                        success += 1;
                    },
                    Err(e) => {
                        println!("\x1b[1;31m Success (in {:.2}s, retry {} time(s)), But update failed: {}\x1b[0m", (info.time_cost as f64) / 1000.0, info.retry_count, e);
                        failed += 1;
                    }
                }

                if index < result_count {
                    download_module.wait(info.time_cost).await;
                }
            },
            Err(e) => {
                println!("\x1b[1;31m Failed: {}\x1b[0m", e);
                failed += 1;

                if index < result_count {
                    download_module.wait(0).await;
                }
            }
        }
        index += 1;
    }

    println!("---");
    println!("Processing complated. Success {} item(s), failed {} item(s).", success, failed);
}

pub async fn connect(context: &mut Context<'_>) {
    context.server_manager.waiting_for_start().await;
    // let source_data_module = SourceDataModule::new(context.server_manager);
    println!("TO BE IMPLEMENTED");
}