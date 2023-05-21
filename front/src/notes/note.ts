import {Record} from "../records/record.ts";
import {SerializedEditorState} from "lexical";

export type Note = Record & {
    mime_type: "note",
    body: SerializedEditorState,
};


export const isNote = (record: Record): record is Note => record.mime_type==='note';
