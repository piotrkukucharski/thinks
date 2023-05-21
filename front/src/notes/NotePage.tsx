import {useQuery} from "@tanstack/react-query";
import {useParams} from "react-router";
import {
    isUuid,
    getRecord,
    NotFoundRecordByIdError,
} from "../records/record.ts";
import Composer from "./editor/Composer.tsx";
import {isNote} from "./note.ts";


export default function NotePage(): JSX.Element {
    const {noteId} = useParams();

    if (!isUuid(noteId)) {
        throw new Error();
    }


    const {isLoading, isError, data, error} = useQuery({
        queryKey: ['records', noteId],
        queryFn: ({queryKey}) => getRecord(queryKey[1])
    })

    if (isLoading) {
        return <>LOADING</>
    }
    if (isError || data === undefined) {
        if (error instanceof NotFoundRecordByIdError) {
            return <pre>Error Not Found {noteId}</pre>
        }
        return <pre>{JSON.stringify(error)}</pre>
    }

    if (!isNote(data)) {
        throw new Error();
    }

    return (
        <Composer body={data.body} noteId={noteId}/>
    );
}
