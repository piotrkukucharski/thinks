import {useEffect, useState} from "react";
import {generateUuid, uuid} from "../records/record.ts";
import Composer from "./editor/Composer.tsx";

export default function NewNotePage(): JSX.Element {
    const [noteId, setNoteId] = useState<uuid|undefined>(undefined);
    useEffect(()=>setNoteId(generateUuid()),[]);
    if(noteId === undefined){
        return <></>
    }
    return <Composer noteId={noteId} body={null}/>
}
