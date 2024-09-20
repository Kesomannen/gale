import { listen } from "@tauri-apps/api/event"

type LoadingBar = {
    id: string,
    title: string,
    text: string | null,
    progress: number | null,
}

type LoadingBarUpdate = {
    id: string,
    text: string | null,
    progress: number | null,
};

let loadingBarsInternal: LoadingBar[] = $state([]);

listen<{
    id: string,
    title: string,
}>('loading-bar-create', ({ payload: { id, title } }) => {
    loadingBarsInternal.push({
        id,
        title,
        text: null,
        progress: null,
    });
});

listen<LoadingBarUpdate>('loading-bar-update', ({ payload: { id, text, progress } }) => {
    const bar = loadingBarsInternal.find(bar => bar.id === id);
    if (bar === undefined) return;
    
    if (text !== null) bar.text = text;
    if (progress !== null) bar.progress = progress;
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