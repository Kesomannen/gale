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

type InvokeError = {
	message: string;
	detail: string;
};

export async function invoke<T = void>(cmd: string, args?: any): Promise<T> {
	try {
		return await tauriInvoke<T>(cmd, args);
	} catch (anyError: any) {
		let error = anyError as InvokeError;
		tauriInvoke('log_err', { msg: `${cmd} failed: ${error.detail}` });

		let name = `Failed to ${toSentenceCase(cmd).toLowerCase()}`;
		let displayMessage = error.message[0].toUpperCase() + error.message.slice(1);

		if (!['.', '?', '!'].includes(displayMessage[displayMessage.length - 1])) {
			displayMessage += '.';
		}

		pushToast({
			type: 'error',
			name,
			message: displayMessage
		});

		throw error.detail;
	}
}
