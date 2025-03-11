import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { sentenceCase } from './util';
import { pushToast } from './toast';

type Error = {
	name: string;
	message: string;
};

listen<Error>('error', (evt) =>
	pushToast({
		type: 'error',
		...evt.payload
	})
);

export async function invokeCommand<T>(cmd: string, args?: any): Promise<T> {
	try {
		return await invoke<T>(cmd, args);
	} catch (error: any) {
		let errStr = error as string;
		let name = `Failed to ${sentenceCase(cmd).toLowerCase()}`;
		let message = errStr[0].toUpperCase() + errStr.slice(1);

		if (!message.endsWith('.') && !message.endsWith('?') && !message.endsWith('!')) {
			message += '.';
		}

		pushError({ name, message });
		throw error;
	}
}

function pushError(error: Error) {
	let msg = `${error.name}: ${error.message}`;
	invoke('log_err', { msg });

	pushToast({
		type: 'error',
		...error
	});
}
