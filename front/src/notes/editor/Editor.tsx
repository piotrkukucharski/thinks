import {AutoFocusPlugin} from '@lexical/react/LexicalAutoFocusPlugin';
import {CheckListPlugin} from '@lexical/react/LexicalCheckListPlugin';
import {ClearEditorPlugin} from '@lexical/react/LexicalClearEditorPlugin';
import LexicalErrorBoundary from '@lexical/react/LexicalErrorBoundary';
import {HashtagPlugin} from '@lexical/react/LexicalHashtagPlugin';
import {HistoryPlugin} from '@lexical/react/LexicalHistoryPlugin';
import {HorizontalRulePlugin} from '@lexical/react/LexicalHorizontalRulePlugin';
import {ListPlugin} from '@lexical/react/LexicalListPlugin';
import {RichTextPlugin} from '@lexical/react/LexicalRichTextPlugin';
import {TabIndentationPlugin} from '@lexical/react/LexicalTabIndentationPlugin';
import {useEffect, useState} from 'react';
import {CAN_USE_DOM} from '../../lexical/utils/canUseDOM.ts';

import {useSharedHistoryContext} from '../../lexical/context/SharedHistoryContext.tsx';
import TableCellNodes from '../../lexical/nodes/TableCellNodes.ts';
import ActionsPlugin from '../../lexical/plugins/ActionsPlugin';
import AutoEmbedPlugin from '../../lexical/plugins/AutoEmbedPlugin';
import AutoLinkPlugin from '../../lexical/plugins/AutoLinkPlugin';
import CodeActionMenuPlugin from '../../lexical/plugins/CodeActionMenuPlugin';
import CodeHighlightPlugin from '../../lexical/plugins/CodeHighlightPlugin';
import CollapsiblePlugin from '../../lexical/plugins/CollapsiblePlugin';
import ComponentPickerPlugin from '../../lexical/plugins/ComponentPickerPlugin';
import DragDropPaste from '../../lexical/plugins/DragDropPastePlugin';
import DraggableBlockPlugin from '../../lexical/plugins/DraggableBlockPlugin';
import EmojiPickerPlugin from '../../lexical/plugins/EmojiPickerPlugin';
import EmojisPlugin from '../../lexical/plugins/EmojisPlugin';
import EquationsPlugin from '../../lexical/plugins/EquationsPlugin';
import FloatingLinkEditorPlugin from '../../lexical/plugins/FloatingLinkEditorPlugin';
import FloatingTextFormatToolbarPlugin from '../../lexical/plugins/FloatingTextFormatToolbarPlugin';
import ImagesPlugin from '../../lexical/plugins/ImagesPlugin';
import KeywordsPlugin from '../../lexical/plugins/KeywordsPlugin';
import LinkPlugin from '../../lexical/plugins/LinkPlugin';
import ListMaxIndentLevelPlugin from '../../lexical/plugins/ListMaxIndentLevelPlugin';
import MarkdownShortcutPlugin from '../../lexical/plugins/MarkdownShortcutPlugin';
import MentionsPlugin from '../../lexical/plugins/MentionsPlugin';
import TabFocusPlugin from '../../lexical/plugins/TabFocusPlugin';
import TableCellActionMenuPlugin from '../../lexical/plugins/TableActionMenuPlugin';
import {TablePlugin} from '../../lexical/plugins/TablePlugin.tsx';
import ToolbarPlugin from '../../lexical/plugins/ToolbarPlugin';
import YouTubePlugin from '../../lexical/plugins/YouTubePlugin';
import PlaygroundEditorTheme from '../../lexical/themes/PlaygroundEditorTheme.ts';
import ContentEditable from '../../lexical/ui/ContentEditable.tsx';
import Placeholder from '../../lexical/ui/Placeholder.tsx';
import {uuid} from "../../records/record.ts";

export default function bEditor(params: { noteId: uuid,namespace: string}): JSX.Element {
    const {historyState} = useSharedHistoryContext();
    const text = '';
    const placeholder = <Placeholder>{text}</Placeholder>;
    const [floatingAnchorElem, setFloatingAnchorElem] =
        useState<HTMLDivElement | null>(null);
    const [isSmallWidthViewport, setIsSmallWidthViewport] =
        useState<boolean>(false);

    const onRef = (_floatingAnchorElem: HTMLDivElement) => {
        if (_floatingAnchorElem !== null) {
            setFloatingAnchorElem(_floatingAnchorElem);
        }
    };

    const cellEditorConfig = {
        namespace: params.namespace,
        nodes: [...TableCellNodes],
        onError: (error: Error) => {
            throw error;
        },
        theme: PlaygroundEditorTheme,
    };

    useEffect(() => {
        const updateViewPortWidth = () => {
            const isNextSmallWidthViewport =
                CAN_USE_DOM && window.matchMedia('(max-width: 1025px)').matches;

            if (isNextSmallWidthViewport !== isSmallWidthViewport) {
                setIsSmallWidthViewport(isNextSmallWidthViewport);
            }
        };
        updateViewPortWidth();
        window.addEventListener('resize', updateViewPortWidth);

        return () => {
            window.removeEventListener('resize', updateViewPortWidth);
        };
    }, [isSmallWidthViewport]);

    return (
        <>
            <ToolbarPlugin/>
            <div
                className={`editor-container`}>
                <DragDropPaste/>
                <AutoFocusPlugin/>
                <ClearEditorPlugin/>
                <ComponentPickerPlugin/>
                <EmojiPickerPlugin/>
                <AutoEmbedPlugin/>
                <MentionsPlugin/>
                <EmojisPlugin/>
                <HashtagPlugin/>
                <KeywordsPlugin/>
                <AutoLinkPlugin/>
                <HistoryPlugin externalHistoryState={historyState}/>
                <RichTextPlugin
                    contentEditable={
                        <div className="editor-scroller">
                            <div className="editor" ref={onRef}>
                                <ContentEditable/>
                            </div>
                        </div>
                    }
                    placeholder={placeholder}
                    ErrorBoundary={LexicalErrorBoundary}
                />
                <MarkdownShortcutPlugin/>
                <CodeHighlightPlugin/>
                <ListPlugin/>
                <CheckListPlugin/>
                <ListMaxIndentLevelPlugin maxDepth={7}/>
                <TablePlugin cellEditorConfig={cellEditorConfig}>
                    <AutoFocusPlugin/>
                    <RichTextPlugin
                        contentEditable={
                            <ContentEditable className="TableNode__contentEditable"/>
                        }
                        placeholder={null}
                        ErrorBoundary={LexicalErrorBoundary}
                    />
                    <MentionsPlugin/>
                    <HistoryPlugin/>
                    <ImagesPlugin captionsEnabled={false}/>
                    <LinkPlugin/>
                    <FloatingTextFormatToolbarPlugin/>
                </TablePlugin>
                <ImagesPlugin/>
                <LinkPlugin/>
                <YouTubePlugin/>
                <HorizontalRulePlugin/>
                <EquationsPlugin/>
                <TabFocusPlugin/>
                <TabIndentationPlugin/>
                <CollapsiblePlugin/>
                {floatingAnchorElem && !isSmallWidthViewport && (
                    <>
                        <DraggableBlockPlugin anchorElem={floatingAnchorElem}/>
                        <CodeActionMenuPlugin anchorElem={floatingAnchorElem}/>
                        <FloatingLinkEditorPlugin anchorElem={floatingAnchorElem}/>
                        <TableCellActionMenuPlugin
                            anchorElem={floatingAnchorElem}
                            cellMerge={true}
                        />
                        <FloatingTextFormatToolbarPlugin
                            anchorElem={floatingAnchorElem}
                        />
                    </>
                )}
                <ActionsPlugin noteId={params.noteId} />
            </div>
        </>
    );
}
