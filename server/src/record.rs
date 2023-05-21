pub mod record {
    use serde::{Serialize};
    use uuid::Uuid;


    #[derive(Debug, Serialize)]
    pub struct ErrNoId {
        pub id: Uuid,
        pub err: String,
    }
}


pub mod http {
    use actix_web::{HttpResponse, ResponseError};
    use actix_web::body::BoxBody;
    use actix_web::http::StatusCode;

    use crate::record::record::ErrNoId;

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

}

pub mod service {
    use chrono::{DateTime, Utc};
    use r2d2::{Pool};
    use r2d2_sqlite::SqliteConnectionManager;
    use uuid::Uuid;
    use serde::{Deserialize, Serialize};

    use crate::record::queries::{delete_record, insert_record, select_record, select_records, WriteRecord};
    use crate::record::record::{ErrNoId};

    #[derive(Deserialize, Serialize)]
    pub struct RequestRecord {
        pub id: Uuid,
        pub mime_type: String,
        pub body: serde_json::Value,
    }

    #[derive(Deserialize, Serialize)]
    pub struct ResponseRecord {
        pub id: Uuid,
        pub mime_type: String,
        pub body: serde_json::Value,
        pub updated_at: DateTime<Utc>,
    }

    pub fn all_records(pool: &Pool<SqliteConnectionManager>) -> Vec<ResponseRecord> {
        select_records(pool).into_iter().map(|read_record| ResponseRecord {
            id: read_record.id,
            mime_type: read_record.mime_type,
            body: read_record.body,
            updated_at: read_record.updated_at,
        }).collect()
    }

    pub fn get_record(record_id: Uuid, pool: &Pool<SqliteConnectionManager>) -> Result<ResponseRecord, ErrNoId> {
        let read_record = select_record(record_id, &pool);
        if read_record.is_ok() {
            let record = read_record.unwrap();
            return Ok(ResponseRecord {
                id: record.id,
                mime_type: record.mime_type,
                body: record.body,
                updated_at: record.updated_at,
            });
        }
        Err(read_record.err().unwrap())
    }

    pub fn add_record(record: RequestRecord, pool: &Pool<SqliteConnectionManager>) {
        let current = Utc::now();
        insert_record(
            WriteRecord {
                id: record.id,
                mime_type: record.mime_type,
                body: record.body,
                created_at: current,
            }
            , &pool)
    }

    pub fn remove_record(record_id: Uuid, pool: &Pool<SqliteConnectionManager>) {
        delete_record(record_id, &pool)
    }
}

pub mod queries {
    use std::str::FromStr;
    use serde::{Deserialize, Serialize};

    use chrono::{DateTime, TimeZone, Utc};
    use r2d2::{Pool};
    use r2d2_sqlite::SqliteConnectionManager;
    use uuid::Uuid;

    use crate::record::record::{ErrNoId};

    #[derive(Deserialize, Serialize)]
    pub struct ReadRecord {
        pub id: Uuid,
        pub mime_type: String,
        pub body: serde_json::Value,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct WriteRecord {
        pub id: Uuid,
        pub mime_type: String,
        pub body: serde_json::Value,
        pub created_at: DateTime<Utc>,
    }

    pub fn select_records(pool: &Pool<SqliteConnectionManager>) -> Vec<ReadRecord> {
        let connection = pool.get().unwrap();
        let mut stmt = connection.prepare("SELECT rr.id, rr.mime_type, rr.body, rr.updated_at FROM records_read rr ORDER BY updated_at DESC").unwrap();

        let result_of_records = stmt.query_map([], |row| Ok(ReadRecord {
            id: Uuid::from_str(&row.get_unwrap::<_, String>(0)).unwrap(),
            mime_type: row.get_unwrap::<_, String>(1),
            body: row.get_unwrap::<_, serde_json::Value>(2),
            updated_at: Utc.timestamp_millis_opt(row.get_unwrap::<_, i64>(3)).unwrap(),
        }));

        let mut records: Vec<ReadRecord> = Vec::new();

        for result_of_record in result_of_records.unwrap() {
            records.push(result_of_record.unwrap());
        }

        records
    }

    pub fn select_record(record_id: Uuid, pool: &Pool<SqliteConnectionManager>) -> Result<ReadRecord, ErrNoId> {
        let connection = pool.get().unwrap();
        let mut stmt = connection.prepare("SELECT rr.id, rr.mime_type, rr.body, rr.updated_at FROM records_read rr WHERE id = ?1 LIMIT 1").unwrap();

        let result_of_record = stmt.query_row([record_id.to_string().as_str()], |row| Ok(ReadRecord {
            id: Uuid::from_str(&row.get_unwrap::<_, String>(0)).unwrap(),
            mime_type: row.get_unwrap::<_, String>(1),
            body: row.get_unwrap::<_, serde_json::Value>(2),
            updated_at: Utc.timestamp_millis_opt(row.get_unwrap::<_, i64>(3)).unwrap(),
        }));

        match result_of_record {
            Ok(v) => Ok(v),
            Err(r2d2_sqlite::rusqlite::Error::QueryReturnedNoRows) => Err(ErrNoId {
                id: record_id,
                err: String::from(format!("Record '{}' not found", record_id)),
            }),
            Err(e) => panic!("Error: {}", e)
        }
    }

    pub fn insert_record(record: WriteRecord, pool: &Pool<SqliteConnectionManager>) {
        let connection = pool.get().unwrap();
        let mut stmt = connection.prepare("INSERT INTO records_write (id, mime_type, body, created_at) VALUES (?1,?2,?3,?4)").unwrap();
        stmt.execute([
            record.id.to_string().as_str(),
            record.mime_type.as_str(),
            &record.body.to_string(),
            &record.created_at.timestamp().to_string(),
        ]).unwrap();
    }

    pub fn delete_record(record_id: Uuid, pool: &Pool<SqliteConnectionManager>) {
        let connection = pool.get().unwrap();
        let mut stmt = connection.prepare("DELETE FROM records_write WHERE id = ?1").unwrap();
        stmt.execute([
            record_id.to_string().as_str(),
        ]).unwrap();
    }

    #[cfg(test)]
    mod tests {
        use chrono::{TimeZone, Utc};
        use uuid::Uuid;

        use crate::record::queries::{delete_record, insert_record, select_record, select_records, WriteRecord};
        use crate::tests::{init_pool, initialize_db};

        #[test]
        fn test_insert_record() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("0cdc6d0f-bbbc-455d-914d-61f211fcece2").unwrap();
            let json_str_body = "{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}";
            let json_body = serde_json::from_str(json_str_body).unwrap();

            let requested_record = WriteRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: json_body,
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            };

            insert_record(requested_record, &pool);

            let result = select_record(id, &pool);
            assert!(result.is_ok());
            let record = result.unwrap();
            assert_eq!(record.id, id);
        }

        #[test]
        fn test_select_records() {
            initialize_db();
            let pool = init_pool();

            let inserted_json_str_body = "{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1083367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}";
            let inserted_json_body: serde_json::Value = serde_json::from_str(inserted_json_str_body).unwrap();


            insert_record(WriteRecord {
                id: Uuid::parse_str("4ac25667-609c-4edb-b44c-ac41b6ab2458").unwrap(),
                mime_type: String::from("note/lexical"),
                body: inserted_json_body.clone(),
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);
            insert_record(WriteRecord {
                id: Uuid::parse_str("5b0fc422-8d76-47c9-895b-6e5f057b27ff").unwrap(),
                mime_type: String::from("note/lexical"),
                body: inserted_json_body.clone(),
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);
            insert_record(WriteRecord {
                id: Uuid::parse_str("386e609e-d3df-438e-9451-1af8928b5da3").unwrap(),
                mime_type: String::from("note/lexical"),
                body: inserted_json_body.clone(),
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);

            let records = select_records(&pool);
            assert!(records.len() >= 3);
        }

        #[test]
        fn test_select_record() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("08766606-db6a-4ceb-8400-f4d2d542ab07").unwrap();

            let inserted_json_str_body = "{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1083367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}";
            let inserted_json_body: serde_json::Value = serde_json::from_str(inserted_json_str_body).unwrap();

            let inserted_record = WriteRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: inserted_json_body,
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            };

            insert_record(inserted_record, &pool);


            let result = select_record(id, &pool);
            assert!(result.is_ok());
            let record = result.unwrap();
            assert_eq!(record.id, id);
        }

        #[test]
        fn test_delete_record() {
            initialize_db();
            let pool = init_pool();
            let id = Uuid::parse_str("c26f6038-a956-459c-8856-710e7dfc0867").unwrap();
            let json_str_body = "{\"editorState\":{\"root\":{\"children\":[],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}},\"lastSaved\":1683367373153,\"source\":\"Playground\",\"version\":\"0.10.0\"}";
            let json_body: serde_json::Value = serde_json::from_str(json_str_body).unwrap();

            insert_record(WriteRecord {
                id,
                mime_type: String::from("note/lexical"),
                body: json_body,
                created_at: Utc.timestamp_millis_opt(1).unwrap(),
            }, &pool);
            delete_record(id, &pool);

            let requested_record = select_record(id, &pool);
            assert!(requested_record.is_err());
        }
    }
}

