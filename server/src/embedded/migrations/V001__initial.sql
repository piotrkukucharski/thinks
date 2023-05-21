create table records_write
(
    id         blob    not null on conflict fail,
    created_at integer not null on conflict fail,
    mime_type  text    not null on conflict fail,
    body       blob    not null on conflict fail,
    constraint records_write_pk
        primary key (id, created_at) on conflict fail
);

create unique index records_write_id_created_at_uindex
    on records_write (id asc, created_at desc);

create index records_write_mime_type_created_at_index
    on records_write (mime_type asc, created_at desc);

CREATE VIEW records_read AS
SELECT rw.id, rw.created_at AS updated_at, rw.mime_type, rw.body
FROM records_write rw
WHERE (rw.id, rw.created_at) IN (SELECT id, MAX(created_at) FROM records_write GROUP BY id);
