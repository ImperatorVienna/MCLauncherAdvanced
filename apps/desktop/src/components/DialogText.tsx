import { useContext, useLayoutEffect, useRef, type ReactNode } from "react";
import { Navigation } from "./Control";

/** Shared selectable text area and scroll target for document and log dialogs. */
export function DialogText({id,label,text,children,documentText=false}:{id:string;label:string;text?:string;children?:ReactNode;documentText?:boolean}) {
    const manager=useContext(Navigation),content=useRef<HTMLDivElement>(null);
    useLayoutEffect(()=>{
        const element=content.current!;
        const unregister=manager.register(id,{element,scope:"modal",group:"content"});
        manager.modalScroll=element;
        return()=>{unregister();if(manager.modalScroll===element)manager.modalScroll=null;};
    },[manager,id]);
    return <div ref={content} id={id} className={`log-content${documentText ? " legal-content" : ""}`} tabIndex={0} aria-label={label}>{children ?? text}</div>;
}
