import {SerializedEditorState} from 'lexical';
import Nodes from "../lexical/nodes/Nodes.ts";
import {uuid} from "../records/record.ts";
import PlaygroundEditorTheme from "../lexical/themes/PlaygroundEditorTheme.ts";
import React, {useEffect, useState} from "react";
import {$generateHtmlFromNodes} from "@lexical/html";
import {createHeadlessEditor} from '@lexical/headless';
import styled, {IStyledComponent} from "styled-components";
import {ImageNode} from "../lexical/nodes/ImageNode.tsx";


const ViewerStyledComponent:IStyledComponent<"web", "div", never> = styled.div`
    width: ${props=> props.$inputWidth}px;
    max-height: ${props=> props.$inputMaxHeight}vh;
    overflow: scroll;
`;

export default function Viewer(params: { width:number, noteId: uuid, body: SerializedEditorState }): JSX.Element {
    const [html,setHtml] = useState<string>("<div></div>");
    const [maxHeight,setMaxHeight] = useState<number>(50);
    useEffect(()=>{
        const editor = createHeadlessEditor({
            namespace: `note-${params.noteId}`,
            nodes: [...Nodes],
            theme: PlaygroundEditorTheme,
            editable: false,
        });
        const editorState = editor.parseEditorState(params.body);
        editor.setEditorState(editorState);
        editor.registerNodeTransform(ImageNode,(imageNote)=>{
            imageNote.__width = params.width-20;
        });

        editor.update(()=>{
            setHtml($generateHtmlFromNodes(editor));
        });
    },[]);

    useEffect(()=>{
        setMaxHeight(Math.floor(Math.random() * (80 - 40) + 40));
    }, [])

    // @ts-ignore
    return <ViewerStyledComponent $inputMaxHeight={maxHeight} $inputWidth={params.width} dangerouslySetInnerHTML={{__html:html}}/>
}


