import { invoke, isTauri as forTauri } from '@tauri-apps/api/core';
import { client, wrapResponse } from './client';
import { browser } from '$app/environment';
// import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export const isTauri = browser && forTauri();

export async function getRefreshTokenFromTauri() {
	return (await invoke('get_refresh_token')) as string | null;
}

export async function saveRefreshTokenTauri(token: string) {
	const response = await invoke('set_refresh_token', { token });

	return response;
}

export async function getAccessTokenWithTauri() {
	try {
		const refreshToken = await getRefreshTokenFromTauri();
		if (refreshToken) {
			return wrapResponse(await client.auth_refresh_token.query(refreshToken));
		}
	} catch (e) {
		console.error(e);
	}
}

// export function createTauriListeners() {
// 	if (!isTauri) {
// 		return () => {};
// 	}
// 	console.log('setup listeners.');

// 	const unsubscribers: UnlistenFn[] = [];
// 	listen<string[]>('qr_results', async (results) => {
// 		const accountDetails = await client.query(['account.preview', results.payload]);
// 		accountDetails.map(createAccount);
// 	}).then((unsub) => unsubscribers.push(unsub));

// 	return () => {
// 		for (const unsub of unsubscribers) {
// 			unsub();
// 		}
// 	};
// }
