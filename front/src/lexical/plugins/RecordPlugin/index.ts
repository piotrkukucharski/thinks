import {useLexicalComposerContext} from '@lexical/react/LexicalComposerContext';
import {$insertNodeToNearestRoot} from '@lexical/utils';
import {COMMAND_PRIORITY_EDITOR, createCommand, LexicalCommand} from 'lexical';
import {useEffect} from 'react';

import {$createRecordNode, RecordNode} from '../../nodes/RecordNode.tsx';

export const INSERT_RECORD_COMMAND: LexicalCommand<string> = createCommand(
  'INSERT_RECORD_COMMAND',
);

export default function RecordPlugin(): JSX.Element | null {
  const [editor] = useLexicalComposerContext();

  useEffect(() => {
    if (!editor.hasNodes([RecordNode])) {
      throw new Error('RecordPlugin: RecordNode not registered on editor');
    }

    return editor.registerCommand<string>(
      INSERT_RECORD_COMMAND,
      (payload) => {
        const youTubeNode = $createRecordNode(payload);
        $insertNodeToNearestRoot(youTubeNode);

        return true;
      },
      COMMAND_PRIORITY_EDITOR,
    );
  }, [editor]);

  return null;
}
