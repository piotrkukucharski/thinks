import Nodes from "../../lexical/nodes/Nodes.ts";
import PlaygroundEditorTheme from "../../lexical/themes/PlaygroundEditorTheme.ts";
import {InitialConfigType, LexicalComposer} from "@lexical/react/LexicalComposer";
import {SharedHistoryContext} from "../../lexical/context/SharedHistoryContext.tsx";
import {TableContext} from "../../lexical/plugins/TablePlugin.tsx";
import {SharedAutocompleteContext} from "../../lexical/context/SharedAutocompleteContext.tsx";
import Editor from "./Editor.tsx";
import {uuid} from "../../records/record.ts";

export default function bComposer(params: { noteId: uuid, body: object | null }): JSX.Element {
    const initialConfig: InitialConfigType = {
        editorState: params.body === null ? null : JSON.stringify(params.body),
        namespace: `note-${params.noteId}`,
        nodes: [...Nodes],
        onError: (error: Error) => {
            throw error;
        },
        theme: PlaygroundEditorTheme,
    };

    return (
        <LexicalComposer initialConfig={initialConfig}>
            <SharedHistoryContext>
                <TableContext>
                    <SharedAutocompleteContext>
                        <div className="editor-shell">
                            <Editor noteId={params.noteId} namespace={`note-${params.noteId}`}/>
                        </div>
                    </SharedAutocompleteContext>
                </TableContext>
            </SharedHistoryContext>
        </LexicalComposer>
    );
}
