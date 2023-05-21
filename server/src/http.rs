use actix_web::{App, HttpResponse, HttpServer, Scope, web};
use actix_web::http::header::{ContentDisposition, ContentType};
use actix_web::middleware::Logger;
use env_logger::Env;
use futures::StreamExt;
use r2d2::{ManageConnection, Pool};
use r2d2_sqlite::SqliteConnectionManager;
use uuid::Uuid;

use crate::record::record::ErrNoId as err_no_id_for_record;
use crate::storage::storage::ErrNoId as err_no_id_for_storage;
use crate::record::service::{add_record, all_records, get_record, remove_record, RequestRecord};
use crate::storage::storage::service::{RequestDeleteBlob, RequestReadBlob, RequestUploadBlob};

mod record;
mod storage;


async fn get_record_handler(state: web::Data<StateApiRecordsScope>, path: web::Path<Uuid>) -> Result<HttpResponse, err_no_id_for_record>
{
    let record_id = path.into_inner();

    let result_record = get_record(record_id, &state.pool);
    match result_record {
        Ok(v) => Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&v).unwrap())
        ),
        Err(e) => Err(e),
    }
}

async fn get_records_handler(state: web::Data<StateApiRecordsScope>) -> HttpResponse
{
    let records = all_records(&state.pool);
    HttpResponse::Ok()
        .insert_header(ContentType::json())
        .json(records)
}

async fn delete_record_handler(state: web::Data<StateApiRecordsScope>, path: web::Path<Uuid>) -> HttpResponse {
    let record_id = path.into_inner();
    remove_record(record_id, &state.pool);
    HttpResponse::Ok().finish()
}

async fn post_record_handler(state: web::Data<StateApiRecordsScope>, path: web::Path<Uuid>, body: web::Json<RequestRecord>) -> HttpResponse {
    let record_id = path.into_inner();
    let record = body.into_inner();
    if record_id != record.id {
        return HttpResponse::BadRequest().finish();
    }
    add_record(record, &state.pool);

    return HttpResponse::Created().finish();
}

async fn get_view_file_handler(state: web::Data<StateApiStorageScope>, path: web::Path<(Uuid, String)>) -> Result<HttpResponse, err_no_id_for_storage> {
    let (id, filename) = path.into_inner();
    let blob = state.storage_service.read(RequestReadBlob { id, filename });
    match blob {
        Ok(v) => Ok(HttpResponse::Ok()
            .insert_header(("Content-Type", v.mime_type))
            .body(v.body)
        ),
        Err(err_no_id_for_storage) => Err(err_no_id_for_storage),
    }
}

async fn get_meta_file_handler(state: web::Data<StateApiStorageScope>, path: web::Path<(Uuid, String)>) -> Result<HttpResponse, err_no_id_for_storage> {
    let (id, filename) = path.into_inner();
    let blob = state.storage_service.read_meta_data(RequestReadBlob { id, filename });
    match blob {
        Ok(v) => Ok(HttpResponse::Ok()
            .insert_header(ContentType::json())
            .body(serde_json::to_string(&v).unwrap())
        ),
        Err(err_no_id_for_storage) => Err(err_no_id_for_storage),
    }
}

async fn get_download_file_handler(state: web::Data<StateApiStorageScope>, path: web::Path<(Uuid, String)>) -> Result<HttpResponse, err_no_id_for_storage> {
    let (id, filename) = path.into_inner();
    let blob = state.storage_service.read(RequestReadBlob { id, filename });
    match blob {
        Ok(v) => Ok(HttpResponse::Ok()
            .insert_header(("Content-Type", v.mime_type))
            .insert_header(ContentDisposition::attachment(v.filename))
            .body(v.body)
        ),
        Err(err_no_id_for_storage) => Err(err_no_id_for_storage),
    }
}

async fn delete_file_handler(state: web::Data<StateApiStorageScope>, path: web::Path<(Uuid, String)>) -> HttpResponse {
    let (id, filename) = path.into_inner();
    state.storage_service.delete(RequestDeleteBlob { id, filename });
    HttpResponse::Ok().finish()
}

async fn post_file_handler(state: web::Data<StateApiStorageScope>, path: web::Path<(Uuid, String)>, mut body: web::Payload) -> HttpResponse {
    let (id, filename) = path.into_inner();

    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }

    state.storage_service.upload(RequestUploadBlob {
        id,
        body: bytes.to_vec(),
        path: filename,
    });
    HttpResponse::Created().finish()
}


mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("/home/ptr/Repositories/think/server/src/embedded/migrations");
}

#[derive(Clone)]
struct StateApiRecordsScope {
    pool: Pool<SqliteConnectionManager>,
}

fn api_records_scope(pool: &Pool<SqliteConnectionManager>) -> Scope {
    web::scope("/api/records")
        .app_data(web::Data::new(StateApiRecordsScope {
            pool: pool.clone()
        }))
        .route("", web::get().to(get_records_handler))
        .route("/{record}", web::get().to(get_record_handler))
        .route("/{record}", web::delete().to(delete_record_handler))
        .route("/{record}", web::post().to(post_record_handler))
}

struct StateApiStorageScope {
    storage_service: storage::storage::service::Service,
}

fn api_storage_scope(pool: &Pool<SqliteConnectionManager>) -> Scope {
    let storage_service = storage::create_service(pool);

    web::scope("/api/file")
        .app_data(web::Data::new(StateApiStorageScope {
            storage_service,
        }))
        .route("/{file}/{filename}", web::get().to(get_view_file_handler))
        .route("/{file}/meta/{filename}", web::get().to(get_meta_file_handler))
        .route("/{file}/download/{filename}", web::get().to(get_download_file_handler))
        .route("/{file}/{filename}", web::delete().to(delete_file_handler))
        .route("/{file}/{filename}", web::post().to(post_file_handler))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default());

    let database_url = "/home/ptr/Repositories/think/server/tmp/data.db";
    let manager = SqliteConnectionManager::file(database_url);
    let mut connection = manager.connect().unwrap();
    embedded::migrations::runner().run(&mut connection).unwrap();

    let pool = Pool::new(manager).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(
                api_records_scope(&pool)
            ).service(
            api_storage_scope(&pool)
        )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::sync::Once;

    use super::*;

    static INIT: Once = Once::new();
    static DATABASE_URL: &str = "/home/ptr/Repositories/think/server/tmp/data_test.db";

    pub fn initialize_db() {
        INIT.call_once(|| {
            let database_path = std::path::Path::new(DATABASE_URL);
            if database_path.exists() {
                fs::remove_file(database_path).unwrap();
            }
            let manager = SqliteConnectionManager::file(DATABASE_URL);
            let mut connection = manager.connect().unwrap();
            embedded::migrations::runner().run(&mut connection).unwrap();
            let fixtures_records_100 = fs::read_to_string("/home/ptr/Repositories/think/server/src/fixtures/records_100.sql").unwrap();
            connection.execute_batch(fixtures_records_100.as_str()).unwrap();
            // let fixtures_records_10000 = fs::read_to_string("/home/ptr/Repositories/think/server/src/fixtures/records_10000.sql").unwrap();
            // connection.execute_batch(fixtures_records_10000.as_str()).unwrap();
            connection.close().unwrap();
        })
    }

    pub fn init_pool() -> Pool<SqliteConnectionManager> {
        let manager = SqliteConnectionManager::file(DATABASE_URL);
        Pool::new(manager).unwrap()
    }

    #[cfg(test)]
    mod tests_api_records_scope {
        use actix_web::{App, test};
        use actix_web::http::header::ContentType;
        use actix_web::http::StatusCode;
        use chrono::{TimeZone, Utc};
        use uuid::Uuid;
        use crate::api_records_scope;
        use crate::record::queries::{insert_record, ReadRecord, select_record, WriteRecord};
        use crate::record::service::{get_record, RequestRecord};
        use crate::tests::{init_pool, initialize_db};

        #[actix_web::test]
        async fn test_get_records_handler() {
            initialize_db();
            let pool = init_pool();
            let inserted_record_json_body: serde_json::Value = serde_json::from_str("{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}").unwrap();
            insert_record(WriteRecord {
                id: Uuid::parse_str("686d15bd-9f78-4e9a-ad3e-a517384302e9").unwrap(),
                mime_type: String::from("note/lexical"),
                body: inserted_record_json_body.clone(),
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);
            insert_record(WriteRecord {
                id: Uuid::parse_str("ac07da60-d4dd-493e-93cb-277b2b8b4a56").unwrap(),
                mime_type: String::from("note/lexical"),
                body: inserted_record_json_body.clone(),
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);
            insert_record(WriteRecord {
                id: Uuid::parse_str("dce1ceac-300c-473c-a329-b9f404476def").unwrap(),
                mime_type: String::from("note/lexical"),
                body: inserted_record_json_body.clone(),
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);

            let mut app = test::init_service(
                App::new()
                    .service(
                        api_records_scope(&pool)
                    )
            ).await;
            let req = test::TestRequest::get().uri("/api/records").to_request();

            let resp: Vec<ReadRecord> = test::call_and_read_body_json(&mut app, req).await;
            assert!(!resp.is_empty());
        }

        #[actix_web::test]
        async fn test_get_record_handler() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("1ae64856-01d1-42b3-b4bc-19fe3e6b2f60").unwrap();
            let inserted_record_json_body = serde_json::from_str("{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}").unwrap();
            let inserted_record = WriteRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: inserted_record_json_body,
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            };

            insert_record(inserted_record, &pool);

            let mut app = test::init_service(
                App::new()
                    .service(
                        api_records_scope(&pool)
                    )
            ).await;
            let req = test::TestRequest::get().uri(format!("/api/records/{}", id.to_string()).as_str()).to_request();

            let resp: ReadRecord = test::call_and_read_body_json(&mut app, req).await;
            assert_eq!(resp.id, id);
        }

        #[actix_web::test]
        async fn test_delete_record_handler() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("ea698120-4363-45a9-b561-59761f82b2c8").unwrap();
            let inserted_record_json_body = serde_json::from_str("{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}").unwrap();
            let inserted_record = WriteRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: inserted_record_json_body,
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            };

            insert_record(inserted_record, &pool);

            let mut app = test::init_service(
                App::new()
                    .service(
                        api_records_scope(&pool)
                    )
            ).await;
            let req = test::TestRequest::delete().uri(format!("/api/records/{}", id.to_string()).as_str()).to_request();

            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);

            let deleted_record = get_record(id, &pool);
            assert!(deleted_record.is_err());
        }

        #[actix_web::test]
        async fn test_post_record_handler_update_record() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("fd033743-8e6b-417f-9d76-de2dba47a682").unwrap();
            let inserted_record_json_body = serde_json::from_str("{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}").unwrap();
            let inserted_record = WriteRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: inserted_record_json_body,
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            };

            insert_record(inserted_record, &pool);

            let mut app = test::init_service(
                App::new()
                    .service(
                        api_records_scope(&pool)
                    )
            ).await;

            let payload_request_to_update = &RequestRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: serde_json::from_str("{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683467337123,\"source\":\"Playground\",\"version\":\"0.10.0\"}").unwrap(),
            };
            let payload_request_to_update_as_str = serde_json::to_string(payload_request_to_update).unwrap();

            let req = test::TestRequest::post()
                .uri(format!("/api/records/{}", id.to_string()).as_str())
                .insert_header(ContentType::json())
                .set_payload(payload_request_to_update_as_str)
                .to_request();

            let resp = test::call_service(&mut app, req).await;

            assert_eq!(resp.status(), StatusCode::CREATED);
            let updated_record = select_record(id, &pool).unwrap();
            assert_eq!(updated_record.body, payload_request_to_update.body);
        }

        #[actix_web::test]
        async fn test_post_record_handler_insert_record() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("b984d681-f2f6-4cb6-9bfd-ef32a6b8119a").unwrap();

            let mut app = test::init_service(
                App::new()
                    .service(
                        api_records_scope(&pool)
                    )
            ).await;

            let payload_request_to_update = &RequestRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: serde_json::from_str("{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683467337123,\"source\":\"Playground\",\"version\":\"0.10.0\"}").unwrap(),
            };
            let payload_request_to_update_as_str = serde_json::to_string(payload_request_to_update).unwrap();

            let req = test::TestRequest::post()
                .uri(format!("/api/records/{}", id.to_string()).as_str())
                .insert_header(ContentType::json())
                .set_payload(payload_request_to_update_as_str)
                .to_request();

            let resp = test::call_service(&mut app, req).await;

            assert_eq!(resp.status(), StatusCode::CREATED);
            let updated_record = select_record(id, &pool).unwrap();
            assert_eq!(updated_record.body, payload_request_to_update.body);
        }
    }

    #[cfg(test)]
    mod tests_api_storage_scope {
        use std::str::FromStr;
        use actix_web::{App, test};
        use actix_web::http::StatusCode;
        use uuid::Uuid;
        use crate::api_storage_scope;
        use crate::storage::{create_service};
        use crate::storage::storage::service::{RequestReadBlob, RequestUploadBlob, ResponseReadMetaDataBlob};
        use crate::tests::{init_pool, initialize_db};

        #[actix_web::test]
        async fn test_get_view_file_handler() {
            initialize_db();
            let pool = init_pool();
            let storage_service = create_service(&pool);
            let app = test::init_service(
                App::new()
                    .service(
                        api_storage_scope(&pool)
                    )
            ).await;
            storage_service.upload(RequestUploadBlob {
                id: Uuid::from_str("260fc36a-1295-48a1-906d-93d8e8465732").unwrap(),
                body: "TEST".as_bytes().to_vec(),
                path: "test.txt".to_string(),
            });
            let req = test::TestRequest::get().uri("/api/file/260fc36a-1295-48a1-906d-93d8e8465732/test.txt").to_request();
            let res = test::call_service(&app, req).await;
            assert!(res.status().is_success())
        }

        #[actix_web::test]
        async fn test_get_download_file_handler() {
            initialize_db();
            let pool = init_pool();
            let storage_service = create_service(&pool);
            let app = test::init_service(
                App::new()
                    .service(
                        api_storage_scope(&pool)
                    )
            ).await;
            storage_service.upload(RequestUploadBlob {
                id: Uuid::from_str("c8f3a893-7d6b-4ace-b2ea-32dbb30c7ff9").unwrap(),
                body: "TEST".as_bytes().to_vec(),
                path: "test.txt".to_string(),
            });
            let req = test::TestRequest::get().uri("/api/file/c8f3a893-7d6b-4ace-b2ea-32dbb30c7ff9/download/test.txt").to_request();
            let res = test::call_service(&app, req).await;
            assert!(res.status().is_success())
        }

        #[actix_web::test]
        async fn test_get_meta_file_handler() {
            initialize_db();
            let pool = init_pool();
            let storage_service = create_service(&pool);
            let app = test::init_service(
                App::new()
                    .service(
                        api_storage_scope(&pool)
                    )
            ).await;
            let id = Uuid::from_str("85d83734-2af0-41b9-9df4-b3131451e572").unwrap();
            storage_service.upload(RequestUploadBlob {
                id,
                body: "TEST".as_bytes().to_vec(),
                path: "test.txt".to_string(),
            });
            let req = test::TestRequest::get().uri("/api/file/85d83734-2af0-41b9-9df4-b3131451e572/meta/test.txt").to_request();
            let res: ResponseReadMetaDataBlob = test::call_and_read_body_json(&app, req).await;
            assert_eq!(res.id, id);
        }

        #[actix_web::test]
        async fn test_delete_file_handler() {
            initialize_db();
            let pool = init_pool();
            let storage_service = create_service(&pool);
            let app = test::init_service(
                App::new()
                    .service(
                        api_storage_scope(&pool)
                    )
            ).await;
            let id = Uuid::from_str("85d83734-2af0-41b9-9df4-b3131451e572").unwrap();
            storage_service.upload(RequestUploadBlob {
                id,
                body: "TEST".as_bytes().to_vec(),
                path: "test.txt".to_string(),
            });
            let req = test::TestRequest::delete().uri("/api/file/85d83734-2af0-41b9-9df4-b3131451e572/test.txt").to_request();
            let res = test::call_service(&app, req).await;
            assert!(res.status().is_success());
            let result = storage_service.read_meta_data(RequestReadBlob { id, filename: "test.txt".to_string() });
            assert!(result.is_err());
        }

        #[actix_web::test]
        async fn test_post_file_handler() {
            initialize_db();
            let pool = init_pool();
            let storage_service = create_service(&pool);
            let app = test::init_service(
                App::new()
                    .service(
                        api_storage_scope(&pool)
                    )
            ).await;
            let id = Uuid::from_str("4d1b9378-6c34-47d1-b27c-da9ab3e6b524").unwrap();
            let body = "TEST".as_bytes().to_vec();
            let request =
                test::TestRequest::post()
                    .uri("/api/file/4d1b9378-6c34-47d1-b27c-da9ab3e6b524/test.txt")
                    .insert_header(("Content-Type","text/plain;charset=UTF-8"))
                    .set_payload(body)
                    .to_request();
            let response = test::call_service(&app, request).await;
            assert!(response.status().eq(&StatusCode::CREATED));
            let meta_data = storage_service.read_meta_data(RequestReadBlob{ id, filename: "test.txt".to_string() });
            assert!(meta_data.is_ok());
        }
    }
}

