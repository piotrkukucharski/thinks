import React, {Context, createContext} from 'react'
import ReactDOM from 'react-dom/client'
import './index.css';
import {ReactQueryDevtools} from '@tanstack/react-query-devtools'
import {createBrowserRouter, NavLink, RouterProvider} from "react-router-dom";
import ErrorPage from "./routes/error-page.tsx";
import RecordsPage from "./records/RecordsPage.tsx";
import {QueryClient, QueryClientProvider} from "@tanstack/react-query";
import {Outlet} from "react-router";
import NewNotePage from "./notes/NewNotePage.tsx";
import NotePage from "./notes/NotePage.tsx";

const queryClient = new QueryClient();

const router = createBrowserRouter([
    {
        path: "/",
        element: <Root/>,
        children: [
            {
                path: "r",
                element: <RecordsPage/>,
                errorElement: <ErrorPage/>,
            },
            {
                path: "n/new",
                element: <NewNotePage/>,
                errorElement: <ErrorPage/>,
            },
            {
                path: "n/:noteId",
                element: <NotePage/>,
                errorElement: <ErrorPage/>,
            }

        ],
    }
]);


export interface ApplicationContextStruct {
}

export const ApplicationContext: Context<ApplicationContextStruct> = createContext({});

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <QueryClientProvider client={queryClient}>
            <ApplicationContext.Provider value={{}}>
                <RouterProvider router={router}/>
            </ApplicationContext.Provider>
        </QueryClientProvider>
    </React.StrictMode>,
);

function Root() {
    return (
        <>
            <header>
                <nav>
                    <NavLink to={"/r"}>Records</NavLink>
                    <NavLink to={`/n/new`}>New Note</NavLink>
                </nav>
            </header>
            <main><Outlet/></main>
            <footer></footer>
            <ReactQueryDevtools initialIsOpen={false}/>
        </>
    );
}
