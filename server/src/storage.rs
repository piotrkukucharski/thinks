use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::storage::storage::service::Service;

pub mod storage {
    use serde::{Serialize};
    use actix_web::http::StatusCode;
    use actix_web::{HttpResponse, ResponseError};
    use actix_web::body::BoxBody;
    use uuid::Uuid;

    pub mod service {
        use std::path::Path;
        use serde::{Deserialize, Serialize};
        use uuid::Uuid;
        use chrono::{DateTime, Utc};
        use mime_guess::MimeGuess;
        use r2d2::{Pool};
        use r2d2_sqlite::SqliteConnectionManager;
        use crate::storage::storage::query::{CompressionStrategy, delete, insert, DbRow, select, select_without_body, DbRowWithoutBody};
        use crate::storage::storage::ErrNoId;
        use blake2::{Blake2b512, Digest};

        #[derive(Deserialize, Serialize)]
        pub struct RequestReadBlob {
            pub id: Uuid,
            pub filename: String,
        }

        #[derive(Deserialize, Serialize)]
        pub struct RequestDeleteBlob {
            pub id: Uuid,
            pub filename: String,
        }

        #[derive(Deserialize, Serialize)]
        pub struct RequestUploadBlob {
            pub id: Uuid,
            pub body: Vec<u8>,
            pub path: String,
        }

        #[derive(Deserialize, Serialize)]
        pub struct ResponseReadBlob {
            pub id: Uuid,
            pub body: Vec<u8>,
            pub mime_type: String,
            pub size: usize,
            pub created_at: DateTime<Utc>,
            pub filename: String,
        }

        impl ResponseReadBlob {
            fn from_row(row: DbRow) -> Self {
                Self {
                    id: row.id,
                    body: row.body,
                    mime_type: row.mime_type.to_string(),
                    size: row.size_before_compress,
                    created_at: row.created_at,
                    filename: row.filename,
                }
            }
        }

        impl ResponseReadMetaDataBlob {
            fn from_row(row: DbRowWithoutBody) -> Self {
                Self {
                    id: row.id,
                    mime_type: row.mime_type.to_string(),
                    size: row.size_before_compress,
                    created_at: row.created_at,
                }
            }
        }

        #[derive(Deserialize, Serialize)]
        pub struct ResponseReadMetaDataBlob {
            pub id: Uuid,
            pub mime_type: String,
            pub size: usize,
            pub created_at: DateTime<Utc>,
        }

        pub struct Service {
            pub pool: Pool<SqliteConnectionManager>,
        }

        impl Service {
            pub fn new(pool: &Pool<SqliteConnectionManager>) -> Self {
                Self { pool: pool.clone() }
            }
            pub fn read(&self, request: RequestReadBlob) -> Result<ResponseReadBlob, ErrNoId> {
                match select(request.id, request.filename, &self.pool) {
                    Ok(v) => Ok(ResponseReadBlob::from_row(v)),
                    Err(err_no_id) => Err(err_no_id)
                }
            }
            pub fn read_meta_data(&self, request: RequestReadBlob) -> Result<ResponseReadMetaDataBlob, ErrNoId> {
                match select_without_body(request.id, request.filename, &self.pool) {
                    Ok(v) => Ok(ResponseReadMetaDataBlob::from_row(v)),
                    Err(err_no_id) => Err(err_no_id)
                }
            }
            pub fn delete(&self, request: RequestDeleteBlob) {
                delete(request.id, request.filename, &self.pool)
            }
            pub fn upload(&self, request: RequestUploadBlob) {
                let mut hasher = Blake2b512::new();
                let body = request.body;
                hasher.update(&body);
                let output = hasher.finalize();
                let hash = hex::encode(output);
                let size = &body.len() * std::mem::size_of::<u8>();
                let filename = Path::new(&request.path).file_name().unwrap().to_str().unwrap().to_string();

                let row = DbRow {
                    id: request.id,
                    mime_type: MimeGuess::from_path(&request.path).first_or_octet_stream(),
                    body,
                    size_after_compress: size,
                    size_before_compress: size,
                    hash_before_compress: hash,
                    compression_strategy: CompressionStrategy::Uncompressed,
                    created_at: Utc::now(),
                    filename,
                };

                insert(row, &self.pool)
            }
        }

        impl Clone for Service {
            fn clone(&self) -> Self {
                Service::new(&self.pool)
            }
        }

        #[cfg(test)]
        mod tests {
            use std::str::FromStr;
            use r2d2::Pool;
            use r2d2_sqlite::SqliteConnectionManager;
            use uuid::Uuid;
            use crate::storage::storage::service::{RequestDeleteBlob, RequestReadBlob, RequestUploadBlob, Service};
            use crate::tests::{init_pool, initialize_db};

            fn init_service(pool: &Pool<SqliteConnectionManager>) -> Service {
                Service::new(&pool)
            }

            #[test]
            fn test_service_new() {
                initialize_db();
                let pool = init_pool();
                Service::new(&pool);
                assert!(true);
            }

            #[test]
            fn test_service_upload() {
                initialize_db();
                let pool = init_pool();
                let service = init_service(&pool);
                let id = Uuid::from_str("84f48191-5daa-427b-af2f-5f7c523e9745").unwrap();
                service.upload(RequestUploadBlob {
                    id,
                    body: "TEST".as_bytes().to_vec(),
                    path: "/tmp/file.txt".to_string(),
                });
            }

            #[test]
            fn test_service_read() {
                initialize_db();
                let pool = init_pool();
                let service = init_service(&pool);
                let id = Uuid::from_str("23885056-5dc3-4e17-9d25-1ad6ce459dca").unwrap();
                let filename = "file.txt";
                service.upload(RequestUploadBlob {
                    id,
                    body: "TEST".as_bytes().to_vec(),
                    path: "/tmp/file.txt".to_string(),
                });
                let response = service.read(RequestReadBlob { id, filename: filename.to_string() });
                assert!(response.is_ok());
            }

            #[test]
            fn test_service_read_when_exist_but_filename_is_difference_then_saved() {
                initialize_db();
                let pool = init_pool();
                let service = init_service(&pool);
                let id = Uuid::from_str("85b6809a-f2c2-4c52-84be-1d772413d3ae").unwrap();
                service.upload(RequestUploadBlob {
                    id,
                    body: "TEST".as_bytes().to_vec(),
                    path: "/tmp/file.txt".to_string(),
                });
                let filename = "file1.txt";
                let response = service.read(RequestReadBlob { id, filename: filename.to_string() });
                assert!(response.is_err());
            }

            #[test]
            fn test_service_read_when_exist_but_id_is_difference_then_saved() {
                initialize_db();
                let pool = init_pool();
                let service = init_service(&pool);
                let id1 = Uuid::from_str("c9484687-1b16-41ff-9eac-455cb173b783").unwrap();
                service.upload(RequestUploadBlob {
                    id: id1,
                    body: "TEST".as_bytes().to_vec(),
                    path: "/tmp/file.txt".to_string(),
                });
                let id2 = Uuid::from_str("80d29c34-e174-48c5-b060-eaf878f66725").unwrap();
                let filename = "file.txt";
                let response = service.read(RequestReadBlob { id: id2, filename: filename.to_string() });
                assert!(response.is_err());
            }

            #[test]
            fn test_service_delete() {
                initialize_db();
                let pool = init_pool();
                let service = init_service(&pool);
                let id = Uuid::from_str("ffbb2557-1d7a-44ec-b949-cd2a886551e1").unwrap();
                service.upload(RequestUploadBlob {
                    id,
                    body: "TEST".as_bytes().to_vec(),
                    path: "/tmp/file.txt".to_string(),
                });
                service.delete(RequestDeleteBlob { id, filename: "file.txt".to_string() });
            }
        }
    }


    #[derive(Debug, Serialize)]
    pub struct ErrNoId {
        pub id: Uuid,
        pub err: String,
    }

    impl ResponseError for ErrNoId {
        fn status_code(&self) -> StatusCode {
            StatusCode::NOT_FOUND
        }

        fn error_response(&self) -> HttpResponse<BoxBody> {
            let body = serde_json::to_string(&self).unwrap();
            let res = HttpResponse::new(self.status_code());
            res.set_body(BoxBody::new(body))
        }
    }

    // Implement Display for ErrNoId
    impl std::fmt::Display for ErrNoId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    mod query {
        use std::fmt;
        use std::fmt::{Formatter};
        use std::str::FromStr;
        use chrono::{DateTime, TimeZone, Utc};
        use mime_guess::Mime;
        use r2d2::{Pool};
        use r2d2_sqlite::SqliteConnectionManager;
        use uuid::Uuid;
        use crate::storage::storage::ErrNoId;

        pub enum CompressionStrategy {
            Lz4,
            Uncompressed,
        }

        impl fmt::Display for CompressionStrategy {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    CompressionStrategy::Lz4 => write!(f, "lz4"),
                    CompressionStrategy::Uncompressed => write!(f, "uncompressed"),
                }
            }
        }

        impl std::str::FromStr for CompressionStrategy {
            type Err = ();

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                match input {
                    "lz4" => Ok(CompressionStrategy::Lz4),
                    "uncompressed" => Ok(CompressionStrategy::Uncompressed),
                    _ => Err(())
                }
            }
        }

        pub struct DbRow {
            pub id: Uuid,
            pub mime_type: Mime,
            pub body: Vec<u8>,
            pub size_after_compress: usize,
            pub size_before_compress: usize,
            pub hash_before_compress: String,
            pub compression_strategy: CompressionStrategy,
            pub created_at: DateTime<Utc>,
            pub filename: String,
        }

        pub struct DbRowWithoutBody {
            pub id: Uuid,
            pub mime_type: Mime,
            pub size_after_compress: usize,
            pub size_before_compress: usize,
            pub hash_before_compress: String,
            pub compression_strategy: CompressionStrategy,
            pub created_at: DateTime<Utc>,
            pub filename: String,
        }

        pub fn insert(row: DbRow, pool: &Pool<SqliteConnectionManager>) {
            let connection = pool.get().unwrap();
            let mut stmt = connection.prepare("INSERT INTO storage (id, mime_type, body, size_after_compress, size_before_compress, hash_before_compress, compression_strategy, created_at, filename) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,?8,?9)").unwrap();
            stmt.execute([
                row.id.to_string().as_str(),
                &row.mime_type.to_string(),
                hex::encode(&row.body).as_str(),
                &row.size_after_compress.to_string(),
                &row.size_before_compress.to_string(),
                &row.hash_before_compress.to_string(),
                &row.compression_strategy.to_string(),
                &row.created_at.timestamp().to_string(),
                &row.filename.to_string(),
            ]).unwrap();
        }

        pub fn delete(id: Uuid, filename: String, pool: &Pool<SqliteConnectionManager>) {
            let connection = pool.get().unwrap();
            let mut stmt = connection.prepare("DELETE FROM storage WHERE id = ?1 AND filename = ?2").unwrap();
            stmt.execute([
                id.to_string().as_str(),
                filename.as_str(),
            ]).unwrap();
        }

        pub fn select(id: Uuid, filename: String, pool: &Pool<SqliteConnectionManager>) -> Result<DbRow, ErrNoId> {
            let connection = pool.get().unwrap();
            let mut stmt = connection.prepare(
                "SELECT id, mime_type, body, size_after_compress, size_before_compress, hash_before_compress, compression_strategy, created_at, filename FROM storage WHERE id = ?1 AND filename = ?2"
            ).unwrap();

            let result_of_blob = stmt.query_row([id.to_string().as_str(), filename.as_str()], |row| Ok(DbRow {
                id: Uuid::from_str(&row.get_unwrap::<_, String>(0)).unwrap(),
                mime_type: Mime::from_str(&row.get_unwrap::<_, String>(1)).unwrap(),
                body: hex::decode(row.get_unwrap::<_, String>(2)).unwrap(),
                size_after_compress: row.get_unwrap::<_, usize>(3),
                size_before_compress: row.get_unwrap::<_, usize>(4),
                hash_before_compress: row.get_unwrap::<_, String>(5),
                compression_strategy: CompressionStrategy::from_str(&row.get_unwrap::<_, String>(6)).unwrap(),
                created_at: Utc.timestamp_millis_opt(row.get_unwrap::<_, i64>(7)).unwrap(),
                filename: row.get_unwrap::<_, String>(8),
            }));

            match result_of_blob {
                Ok(v) => Ok(v),
                Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Err(ErrNoId {
                    id: id,
                    err: String::from(format!("Blob '{}' not found", id)),
                }),
                Err(e) => panic!("Error: {}", e)
            }
        }

        pub fn select_without_body(id: Uuid, filename: String, pool: &Pool<SqliteConnectionManager>) -> Result<DbRowWithoutBody, ErrNoId> {
            let connection = pool.get().unwrap();
            let mut stmt = connection.prepare("SELECT id, mime_type, size_after_compress, size_before_compress, hash_before_compress, compression_strategy, created_at, filename FROM storage WHERE id = ?1 AND filename = ?2").unwrap();

            let result_of_blob = stmt.query_row([id.to_string().as_str(), filename.as_str()], |row| Ok(DbRowWithoutBody {
                id: Uuid::from_str(&row.get_unwrap::<_, String>(0)).unwrap(),
                mime_type: Mime::from_str(&row.get_unwrap::<_, String>(1)).unwrap(),
                size_after_compress: row.get_unwrap::<_, usize>(2),
                size_before_compress: row.get_unwrap::<_, usize>(3),
                hash_before_compress: row.get_unwrap::<_, String>(4),
                compression_strategy: CompressionStrategy::from_str(&row.get_unwrap::<_, String>(5)).unwrap(),
                created_at: Utc.timestamp_millis_opt(row.get_unwrap::<_, i64>(6)).unwrap(),
                filename: row.get_unwrap::<_, String>(7),
            }));

            match result_of_blob {
                Ok(v) => Ok(v),
                Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Err(ErrNoId {
                    id,
                    err: String::from(format!("Blob '{}' not found", id)),
                }),
                Err(e) => panic!("Error: {}", e)
            }
        }

        #[cfg(test)]
        mod tests {
            use std::str::FromStr;
            use blake2::{Blake2b512, Digest};
            use chrono::{TimeZone, Utc};
            use mime_guess::mime;
            use crate::storage::storage::query::{CompressionStrategy, DbRow, delete, insert, select, select_without_body};
            use crate::tests::{init_pool, initialize_db};
            use uuid::Uuid;

            fn create_fixture_row(id: Uuid) -> DbRow {
                let mut hasher = Blake2b512::new();
                let body = "TEST".as_bytes().to_vec();
                hasher.update(&body);
                let output = hasher.finalize();
                let hex = hex::encode(output);
                let size = &body.len() * std::mem::size_of::<u8>();
                let file_name = "test01.txt";
                DbRow {
                    id,
                    mime_type: mime::TEXT_PLAIN,
                    body,
                    size_after_compress: size,
                    size_before_compress: size,
                    hash_before_compress: hex,
                    compression_strategy: CompressionStrategy::Uncompressed,
                    created_at: Utc.timestamp_millis_opt(1).unwrap(),
                    filename: file_name.to_string(),
                }
            }

            #[test]
            fn test_insert() {
                initialize_db();
                let pool = init_pool();
                let id = Uuid::from_str("6ac3f044-000d-4e3f-af0c-98c0005c0695").unwrap();
                insert(create_fixture_row(id), &pool);
            }

            #[test]
            fn test_select() {
                initialize_db();
                let pool = init_pool();
                let id = Uuid::from_str("a1be75d3-3de6-4d38-a182-396a7350387d").unwrap();
                insert(create_fixture_row(id), &pool);
                let result = select(id, "test01.txt".to_string(), &pool);
                assert!(result.is_ok());
            }

            #[test]
            fn test_select_when_not_exist() {
                initialize_db();
                let pool = init_pool();
                let id = Uuid::from_str("0dab7ced-38b1-4080-9768-71acb24385ae").unwrap();
                let result = select(id, "test01.txt".to_string(), &pool);
                assert!(result.is_err());
            }

            #[test]
            fn test_select_without_body() {
                initialize_db();
                let pool = init_pool();
                let id = Uuid::from_str("e1987ed0-a0f1-403b-97b5-4754e9e86834").unwrap();
                insert(create_fixture_row(id), &pool);
                let result = select_without_body(id, "test01.txt".to_string(), &pool);
                assert!(result.is_ok());
            }

            #[test]
            fn test_select_without_body_when_not_exist() {
                initialize_db();
                let pool = init_pool();
                let id = Uuid::from_str("52209560-bd34-45f4-bca3-bfa2d1808ae7").unwrap();
                let result = select_without_body(id, "test01.txt".to_string(), &pool);
                assert!(result.is_err());
            }

            #[test]
            fn test_delete() {
                initialize_db();
                let pool = init_pool();
                let id = Uuid::from_str("e0027c7d-4a1a-47cb-a098-ec612a3f3b87").unwrap();
                insert(create_fixture_row(id), &pool);
                assert!(select_without_body(id, "test01.txt".to_string(), &pool).is_ok());
                delete(id,"test01.txt".to_string(), &pool);
                assert!(select_without_body(id, "test01.txt".to_string(), &pool).is_err());
            }
        }
    }
}

pub fn create_service(pool: &Pool<SqliteConnectionManager>) -> Service {
    storage::service::Service::new(pool)
}