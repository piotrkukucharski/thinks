/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 */

import type {LexicalEditor} from 'lexical';

import {useLexicalComposerContext} from '@lexical/react/LexicalComposerContext';
import {mergeRegister} from '@lexical/utils';
import {
    $getRoot, $isParagraphNode,
} from 'lexical';
import {useEffect, useState} from 'react';

import useModal from '../../hooks/useModal.tsx';
import Button from '../../ui/Button.tsx';
import {deleteRecord, saveRecord, uuid} from "../../../records/record.ts";
import { useNavigate } from "react-router-dom";

export default function ActionsPlugin({noteId}: {noteId: uuid}): JSX.Element {
    const navigate = useNavigate();
    const [editor] = useLexicalComposerContext();
    const [isEditable, setIsEditable] = useState(() => editor.isEditable());
    const [isEditorEmpty, setIsEditorEmpty] = useState(true);
    const [modal, showModal] = useModal();

    useEffect(() => {
        return mergeRegister(
            editor.registerEditableListener((editable) => {
                setIsEditable(editable);
            }),
        );
    }, [editor]);
    useEffect(() => {
        return editor.registerUpdateListener(
            ({dirtyElements, prevEditorState, tags}) => {
                // If we are in read only mode, send the editor state
                // to server and ask for validation if possible.
                editor.getEditorState().read(() => {
                    const root = $getRoot();
                    const children = root.getChildren();

                    if (children.length > 1) {
                        setIsEditorEmpty(false);
                    } else {
                        if ($isParagraphNode(children[0])) {
                            const paragraphChildren = children[0].getChildren();
                            setIsEditorEmpty(paragraphChildren.length === 0);
                        } else {
                            setIsEditorEmpty(false);
                        }
                    }
                });
            },
        );
    }, [editor, isEditable]);
    return (
        <div className="actions">
            <button
                className="action-button save"
                disabled={isEditorEmpty}
                onClick={() => {
                    saveRecord({body: editor.getEditorState().toJSON(), id: noteId, mime_type: "note"}).then(r=>{
                        if(r){
                            navigate(`/n/${noteId}`);
                        }else{
                            alert("CAN'T SAVE");
                        }
                    })
                }}
                title="Export"
                aria-label="Export editor state to JSON">
                <i className="export"/>
            </button>
            <button
                className="action-button delete"
                onClick={() => {
                    showModal('Delete Note', (onClose) => (
                        <ShowDeleteDialog editor={editor} onClose={onClose}/>
                    ));
                }}
                title="Clear"
                aria-label="Clear editor contents">
                <i className="clear"/>
            </button>
            {modal}
        </div>
    );
}

function ShowDeleteDialog({
                             editor,
                             onClose,
                         }: {
    editor: LexicalEditor;
    onClose: () => void;
}): JSX.Element {
    return (
        <>
            Are you sure you want to clear the editor?
            <div className="Modal__content">
                <Button
                    onClick={() => {
                        editor.setEditable(false);
                        editor.focus();
                        onClose();
                    }}>
                    Delete
                </Button>{' '}
                <Button
                    onClick={() => {
                        editor.focus();
                        onClose();
                    }}>
                    Cancel
                </Button>
            </div>
        </>
    );
}
