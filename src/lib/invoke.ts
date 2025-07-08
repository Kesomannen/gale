import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { pushToast } from './toast';
import { toSentenceCase } from 'js-convert-case';

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

export async function invoke<T = void>(cmd: string, args?: any): Promise<T> {
	try {
		return await tauriInvoke<T>(cmd, args);
	} catch (error: any) {
		let errStr = error as string;
		let name = `Failed to ${toSentenceCase(cmd).toLowerCase()}`;
		let message = errStr[0].toUpperCase() + errStr.slice(1);

		if (!['.', '?', '!'].includes(message[message.length - 1])) {
			message += '.';
		}

		pushError({ name, message });
		throw error;
	}
}

function pushError(error: Error) {
	let msg = `${error.name}: ${error.message}`;
	tauriInvoke('log_err', { msg });

	pushToast({
		type: 'error',
		...error
	});
}
