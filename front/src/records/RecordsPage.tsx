import {deleteRecord, getRecords, Record} from "./record.ts";
import {useQuery} from "@tanstack/react-query";
import {Link, useNavigate} from "react-router-dom";
import {isNote} from "../notes/note.ts";
import Viewer from "../notes/Viewer.tsx";
import { Masonry } from "masonic";
import React from "react";
import styled from "styled-components";

const MasonryItemBox = styled.div`
    padding: 5px;
    margin: 5px;
    border: black solid 1px;
`;

const MasonryItem = ({index,data,width}:{index:number, data: Record, width: number})=>{
    const navigate = useNavigate();
    return (
        <MasonryItemBox>
            <button onClick={()=>deleteRecord(data.id).then(()=>navigate("/r"))}>Delete</button>
            <Link to={`/n/${data.id}`}>Go To</Link>
            {isNote(data) &&
                <>
                    <Viewer width={width-20} noteId={data.id} body={data.body}/>
                </>
            }
        </MasonryItemBox>
    );
};

const MasonryCss: React.CSSProperties = {

};

export default function RecordsPage(): JSX.Element {

    const {isLoading, isError, data, error} = useQuery({queryKey: ['records'], queryFn: getRecords})

    if (isLoading) {
        return <>LOADING</>
    }
    if (isError) {
        return <>{error}</>
    }

    return <Masonry
        maxColumnCount={3}
        items={data}
        render={MasonryItem}
        style={MasonryCss}
    />
}

