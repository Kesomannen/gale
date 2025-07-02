import { invoke } from '$lib/invoke';

export const openGaleLog = () => invoke('open_gale_log');
export const logErr = (msg: string) => invoke('log_err', { msg });
