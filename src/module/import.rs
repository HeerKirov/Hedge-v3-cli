use std::{path::PathBuf, error::Error};
use chrono::NaiveDate;
use clap::ValueEnum;
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;
use super::server::{ServerManager, ListResult, IdWithWarning};


pub struct ImportModule<'t> {
    server_manager: &'t ServerManager
}

impl <'t> ImportModule <'t> {
    pub fn new(server_manager: &ServerManager) -> ImportModule {
        ImportModule { server_manager }
    }
    pub async fn list(&mut self) -> Result<ListResult<ImportImageRes>, Box<dyn Error>> {
        self.server_manager.req(Method::GET, "/api/imports").await
    }
    pub async fn add(&mut self, file: &PathBuf, remove: bool) -> Result<IdWithWarning, Box<dyn Error>> {
        let body = json!({
            "filepath": file.to_str().unwrap(),
            "mobileImport": remove
        });
        self.server_manager.req_with_body(Method::POST, "/api/imports/import", body).await
    }
    pub async fn batch(&mut self, partition_time: Option<NaiveDate>, create_time: Option<OrderTimeType>, order_time: Option<OrderTimeType>, analyse_source: bool) -> Result<Vec<IdWithWarning>, Box<dyn Error>> {
        let body = json!({
            "setCreateTimeBy": create_time.map(|f| f.to_json_code()),
            "setOrderTimeBy": order_time.map(|f| f.to_json_code()),
            "analyseSource": analyse_source,
            "partitionTime": partition_time.map(|p| p.format("%Y-%m-%d").to_string())
        });
        self.server_manager.req_with_body(Method::POST, "/api/imports/batch-update", body).await
    }
    pub async fn save(&mut self) -> Result<ImportSaveRes, Box<dyn Error>> {
        let body = json!({
            "target": Option::<Vec<i32>>::None
        });
        self.server_manager.req_with_body(Method::POST, "/api/imports/save", body).await
    }
}

#[derive(Clone, ValueEnum)]
pub enum OrderTimeType {
    CreateTime,
    UpdateTime,
    ImportTime
}

impl OrderTimeType {
    fn to_json_code(&self) -> &'static str {
        match self {
            Self::CreateTime => "CREATE_TIME",
            Self::UpdateTime => "UPDATE_TIME",
            Self::ImportTime => "IMPORT_TIME"
        }
    }
}

#[derive(Deserialize)]
pub struct ImportImageRes {
    pub id: i32,
    pub file: String,
    #[serde(rename = "thumbnailFile")]
    pub thumbnail_file: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "sourceSite")]
    pub source_site: Option<String>,
    #[serde(rename = "sourceId")]
    pub source_id: Option<i64>,
    #[serde(rename = "sourcePart")]
    pub source_part: Option<i32>,
    pub tagme: Vec<String>,
    #[serde(rename = "partitionTime")]
    pub partition_time: String,
    #[serde(rename = "orderTime")]
    pub order_time: String
}

#[derive(Deserialize)]
pub struct ImportSaveRes {
    pub total: i32,
    pub errors: Vec<ImportSaveErrorItem>
}

#[derive(Deserialize)]
pub struct ImportSaveErrorItem {
    #[serde(rename = "importId")]
    pub import_id: i32,
    #[serde(rename = "fileNotReady")]
    pub file_not_ready: bool,
    #[serde(rename = "notExistedCollectionId")]
    pub not_existed_collection_id: Option<i32>,
    #[serde(rename = "notExistedCloneImageId")]
    pub not_existed_clone_image_id: Option<i32>,
    #[serde(rename = "notExistedBookIds")]
    pub not_existed_book_ids: Option<Vec<i32>>,
    #[serde(rename = "notExistedFolderIds")]
    pub not_existed_folder_ids: Option<Vec<i32>>
}