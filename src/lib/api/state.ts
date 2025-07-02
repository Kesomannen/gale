import { invoke } from '$lib/invoke';

export const isFirstRun = () => invoke<boolean>('is_first_run');
