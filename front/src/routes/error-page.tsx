import {useRouteError} from "react-router-dom";
import {isRouteErrorResponse} from "react-router";

export default function ErrorPage() {
    const error = useRouteError();
    console.error(error);

    if(isRouteErrorResponse(error)){
        return (
            <div className={"error-page"}>
                <h1>{error.status}</h1>
                <p>
                    <i>{error.statusText || error.status}</i>
                </p>
            </div>
        );
    }
    return <></>;

}