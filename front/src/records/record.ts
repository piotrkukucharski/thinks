import {v4 as uuidV4, validate as uuidValidate, version as uuidVersion} from 'uuid';
import {createContext} from "react";

export type uuid = string;

export const isUuid = (value: unknown): value is uuid => typeof value === 'string' && uuidValidate(value) && (uuidVersion(value) === 4);

export type Record = {
    readonly id: uuid,
    readonly mime_type: string,
    readonly body: object,
    readonly updated_at: typeof Date,
}

export type RecordToBeSaved = {
    readonly id: uuid,
    readonly mime_type: string,
    readonly body: object,
}

export class NotFoundRecordByIdError extends Error {
    constructor(id: uuid) {
        super(`Not Found Record by id:${id.toString()}`);
        this.name = "NotFoundRecordError";
    }
}

export type RecordEditorContextValue = {
    readonly recordId: uuid,
};

export const isRecordEditorContextValue= (value: RecordEditorContextValue|null): value is RecordEditorContextValue => value !== null;


export const generateUuid = (): uuid => uuidV4();

export const getRecords = (): Promise<Record[]> => {
    return fetch(`/api/records`, {
        method: "GET",
    }).then(r => r.json());
};

export const getRecord = (id: uuid): Promise<Record> => {
    if(!isUuid(id)){
        return Promise.reject(new NotFoundRecordByIdError(id));
    }
    return fetch(`/api/records/${id.toString()}`, {
        method: "GET",
    }).then(r => {
        if (r.status === 404) {
            return Promise.reject(new NotFoundRecordByIdError(id));
        }
        return r.json();
    });
}


export const isExistRecord = (id: uuid): Promise<boolean> => {
    return getRecord(id).catch((e: NotFoundRecordByIdError) => false).then(r => true);
}
export const deleteRecord = (id: uuid): Promise<boolean> => {
    return fetch(`/api/records/${id.toString()}`, {
        method: "DELETE",
    }).then(r => r.status === 200);
}
export const saveRecord = (record: RecordToBeSaved): Promise<boolean> => {
    return fetch(`/api/records/${record.id.toString()}`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(record),
    }).then(r => r.status === 201)
}