import {Record} from "../records/record.ts";
import {SerializedEditorState} from "lexical";
import {createHeadlessEditor} from "@lexical/headless";
import Nodes from "../lexical/nodes/Nodes.ts";
import PlaygroundEditorTheme from "../lexical/themes/PlaygroundEditorTheme.ts";
import {ImageNode} from "../lexical/nodes/ImageNode.tsx";
import {$generateHtmlFromNodes} from "@lexical/html";

export type Note = Record & {
    mime_type: "note",
    body: SerializedEditorState,
};


export const isNote = (record: Record): record is Note => record.mime_type==='note';

export const generateHtmlFromNote = (record: Note, width: number): string => {
    const editor = createHeadlessEditor({
        namespace: `note-${record.id}`,
        nodes: [...Nodes],
        theme: PlaygroundEditorTheme,
        editable: false,
    });

    const editorState = editor.parseEditorState(record.body);
    editor.setEditorState(editorState);
    editor.registerNodeTransform(ImageNode,(imageNote)=>{
        imageNote.__width = width-20;
    });

    let html = '';
    editor.getEditorState().read(()=>{
        html = $generateHtmlFromNodes(editor);
    });

    return html;
};