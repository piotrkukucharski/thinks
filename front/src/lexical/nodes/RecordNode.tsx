/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 */

import type {
  DOMConversionMap,
  DOMConversionOutput,
  DOMExportOutput,
  EditorConfig,
  ElementFormatType,
  LexicalEditor,
  LexicalNode,
  NodeKey,
  Spread,
} from 'lexical';

import {BlockWithAlignableContents} from '@lexical/react/LexicalBlockWithAlignableContents';
import {
  DecoratorBlockNode,
  SerializedDecoratorBlockNode,
} from '@lexical/react/LexicalDecoratorBlockNode';
import * as React from 'react';
import {getRecord, getRecords, uuid} from "../../records/record.ts";
import {useQuery} from "@tanstack/react-query";
import {generateHtmlFromNote, isNote} from "../../notes/note.ts";

type RecordComponentProps = Readonly<{
  className: Readonly<{
    base: string;
    focus: string;
  }>;
  format: ElementFormatType | null;
  nodeKey: NodeKey;
  recordId: uuid;
}>;

function HtmlComponent({
  className,
  format,
  nodeKey,
  recordId,
}: RecordComponentProps) {
  const {isLoading, isError, data, error} = useQuery({
    queryKey: ['records', recordId],
    queryFn: ({queryKey}) => getRecord(queryKey[1])
  })

  if(isLoading){
    return (
        <BlockWithAlignableContents
            className={className}
            format={format}
            nodeKey={nodeKey}>
          <></>
        </BlockWithAlignableContents>
    );
  }

  if (isError || data===undefined){
    console.error(error);
    return (
        <BlockWithAlignableContents
            className={className}
            format={format}
            nodeKey={nodeKey}>
          <p>Error during fetch record "{recordId}"</p>
        </BlockWithAlignableContents>
    );
  }

  if(isNote(data)){
    // @TODO
    const html = generateHtmlFromNote(data,560);
    return (
        <BlockWithAlignableContents
            className={className}
            format={format}
            nodeKey={nodeKey}>
          <div dangerouslySetInnerHTML={{__html:html}}/>
        </BlockWithAlignableContents>
    );
  }

  return (
      <BlockWithAlignableContents
          className={className}
          format={format}
          nodeKey={nodeKey}>
        <pre>{JSON.stringify(data)}</pre>
      </BlockWithAlignableContents>
  );
}

export type SerializedRecordNode = Spread<
  {
    recordId: uuid;
  },
  SerializedDecoratorBlockNode
>;

function convertRecordElement(
  domNode: HTMLElement,
): null | DOMConversionOutput {
  const recordId = domNode.getAttribute('data-lexical-record');
  if (recordId) {
    const node = $createRecordNode(recordId);
    return {node};
  }
  return null;
}

export class RecordNode extends DecoratorBlockNode {
  __id: uuid;

  static getType(): string {
    return 'record';
  }

  static clone(node: RecordNode): RecordNode {
    return new RecordNode(node.__id, node.__format, node.__key);
  }

  static importJSON(serializedNode: SerializedRecordNode): RecordNode {
    const node = $createRecordNode(serializedNode.recordId);
    node.setFormat(serializedNode.format);
    return node;
  }

  exportJSON(): SerializedRecordNode {
    return {
      ...super.exportJSON(),
      type: this.getType(),
      version: 1,
      recordId: this.__id,
    };
  }

  constructor(id: string, format?: ElementFormatType, key?: NodeKey) {
    super(format, key);
    this.__id = id;
  }

  exportDOM(): DOMExportOutput {
    const element = document.createElement('div');
    element.setAttribute('data-lexical-record', this.__id);
    // element.setAttribute('width', '560');
    // element.setAttribute('height', '315');
    // element.setAttribute(
    //   'src',
    //   `https://www.youtube-nocookie.com/embed/${this.__id}`,
    // );
    // element.setAttribute('frameborder', '0');
    // element.setAttribute(
    //   'allow',
    //   'accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture',
    // );
    // element.setAttribute('allowfullscreen', 'true');
    // element.setAttribute('title', 'YouTube video');
    return {element};
  }

  static importDOM(): DOMConversionMap | null {
    return {
      iframe: (domNode: HTMLElement) => {
        if (!domNode.hasAttribute('data-lexical-record')) {
          return null;
        }
        return {
          conversion: convertRecordElement,
          priority: 1,
        };
      },
    };
  }

  updateDOM(): false {
    return false;
  }

  getId(): string {
    return this.__id;
  }

  getTextContent(
    _includeInert?: boolean | undefined,
    _includeDirectionless?: false | undefined,
  ): string {
    return `/r/${this.__id}`;
  }

  decorate(_editor: LexicalEditor, config: EditorConfig): JSX.Element {
    const embedBlockTheme = config.theme.embedBlock || {};
    const className = {
      base: embedBlockTheme.base || '',
      focus: embedBlockTheme.focus || '',
    };
    return (
      <HtmlComponent
        className={className}
        format={this.__format}
        nodeKey={this.getKey()}
        recordId={this.__id}
      />
    );
  }
}

export function $createRecordNode(recordId: uuid): RecordNode {
  return new RecordNode(recordId);
}

export function $isRecordNode(
  node: RecordNode | LexicalNode | null | undefined,
): node is RecordNode {
  return node instanceof RecordNode;
}
