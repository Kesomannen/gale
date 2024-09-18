import { listen } from "@tauri-apps/api/event"
import { message } from "@tauri-apps/plugin-dialog";

type LoadingBar = {
    id: string,
    title: string,
    message: string | null,
    progress: number | null,
}

type LoadingBarUpdate = {
    id: string,
    message: string,
    progress: number,
};

let loadingBarsInternal: LoadingBar[] = $state([]);

listen<LoadingBar>('loading-bar-create', ({ payload }) => {
    loadingBarsInternal.push(payload);
});

listen<LoadingBarUpdate>('loading-bar-update', ({ payload }) => {
    const bar = loadingBarsInternal.find(bar => bar.id === payload.id);
    if (bar === undefined) return;
    
    bar.message = payload.message;
    bar.progress = payload.progress;
});

listen<string>('loading-bar-close', ({ payload }) => {
    loadingBarsInternal = loadingBarsInternal.filter(({ id }) => id !== payload);   
});

let loadingBars = {
    get all() {
        return loadingBarsInternal;
    }
}

export default loadingBars;