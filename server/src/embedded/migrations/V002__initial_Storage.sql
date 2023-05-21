create table storage
(
    id                   blob    not null on conflict fail
        constraint storage_pk
            primary key,
    mime_type            text    not null on conflict fail,
    body                 blob    not null on conflict fail,
    size_after_compress  integer not null on conflict fail,
    size_before_compress integer not null on conflict fail,
    hash_before_compress text    not null on conflict fail,
    compression_strategy text    not null on conflict fail,
    filename             text    not null on conflict fail,
    created_at           integer not null on conflict fail
);

create index storage_created_at_index
    on storage (created_at desc);

create index storage_mime_type_created_at_index
    on storage (mime_type, created_at);

create index storage_mime_type_index
    on storage (mime_type);

create index storage_size_after_compress_index
    on storage (size_after_compress desc);

