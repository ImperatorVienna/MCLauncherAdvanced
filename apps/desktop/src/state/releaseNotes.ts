import { useEffect, useState } from "react";
import { useLocale } from "../i18n";
import { invokeNative, isDesktop } from "../platform/desktop";

export function useReleaseNotes(version: string | undefined) {
    const language = useLocale();
    const key = `${version ?? ""}/${language}`;
    const [result,setResult] = useState<{key:string;body:string|null;failed:boolean}>();
    useEffect(()=>{
        let active=true;
        if(version && isDesktop()) {
            void invokeNative<string|null>("launcher_release_notes",{version,language})
                .then(body=>{if(active)setResult({key,body,failed:false});})
                .catch(()=>{if(active)setResult({key,body:null,failed:true});});
        }
        return()=>{active=false;};
    },[version,language,key]);
    return {body:result?.key===key ? result.body : null,
        failed:result?.key===key && result.failed,
        loading:!!version && isDesktop() && result?.key!==key};
}
